//! Asynchronous background indexing daemon with incremental caching and low-priority throttling

use crate::error::SemanticError;
use crate::extractors::extract_content;
use crate::models::DocumentType;
use crate::search::HybridSearchEngine;
use crc32fast::Hasher;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::thread::{self, sleep, JoinHandle};
use std::time::Duration;
use walkdir::WalkDir;

const CACHE_MAGIC: &[u8; 4] = b"FMCF";
const CACHE_VERSION: u32 = 1;

/// Operating throttle mode balancing indexing speed against CPU load and battery consumption
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ThrottleMode {
    /// Zero artificial delay; maximum indexing throughput
    MaxPerformance,
    /// 5ms delay per file; keeps CPU temperature and fan quiet
    #[default]
    Balanced,
    /// 25ms delay per file; minimizes battery draw on laptops
    BatterySaver,
}

impl ThrottleMode {
    /// Delay between processing consecutive files
    pub fn delay(&self) -> Duration {
        match self {
            ThrottleMode::MaxPerformance => Duration::from_millis(0),
            ThrottleMode::Balanced => Duration::from_millis(5),
            ThrottleMode::BatterySaver => Duration::from_millis(25),
        }
    }

    /// Automatically detects system power status on Linux (/sys/class/power_supply)
    pub fn detect_system_power_mode() -> Self {
        let power_supply = Path::new("/sys/class/power_supply");
        if !power_supply.exists() {
            return ThrottleMode::Balanced;
        }

        if let Ok(entries) = std::fs::read_dir(power_supply) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("BAT") {
                    let status_path = entry.path().join("status");
                    if let Ok(status) = std::fs::read_to_string(status_path) {
                        if status.trim().eq_ignore_ascii_case("Discharging") {
                            return ThrottleMode::BatterySaver;
                        }
                    }
                }
            }
        }

        ThrottleMode::Balanced
    }
}

/// Fingerprint record of an indexed file on disk for fast incremental change detection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileFingerprint {
    pub mtime: u64,
    pub size: u64,
    pub crc32: u32,
}

/// Persistent cache of indexed file fingerprints
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileFingerprintCache {
    entries: HashMap<PathBuf, FileFingerprint>,
}

impl FileFingerprintCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Determines if a file has changed and needs re-indexing
    pub fn should_index(&self, path: &Path, mtime: u64, size: u64) -> bool {
        match self.entries.get(path) {
            Some(cached) => cached.mtime != mtime || cached.size != size,
            None => true,
        }
    }

    /// Updates cache with fresh fingerprint
    pub fn update(&mut self, path: PathBuf, mtime: u64, size: u64, crc32: u32) {
        self.entries
            .insert(path, FileFingerprint { mtime, size, crc32 });
    }

    /// Removes a file from the fingerprint cache
    pub fn remove(&mut self, path: &Path) -> bool {
        self.entries.remove(path).is_some()
    }

    /// Clears the fingerprint cache
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Serializes cache to disk with CRC32 verification
    pub fn save_to_file(&self, path: &Path) -> Result<(), SemanticError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let payload = bincode::serialize(&self.entries).map_err(|e| {
            SemanticError::Extraction(format!("Failed to serialize fingerprint cache: {e}"))
        })?;

        let mut hasher = Hasher::new();
        hasher.update(&payload);
        let checksum = hasher.finalize();

        let mut file = File::create(path)?;
        file.write_all(CACHE_MAGIC)?;
        file.write_all(&CACHE_VERSION.to_le_bytes())?;
        file.write_all(&checksum.to_le_bytes())?;
        file.write_all(&payload)?;
        file.flush()?;

        Ok(())
    }

    /// Deserializes cache from disk verifying magic and CRC32
    pub fn load_from_file(path: &Path) -> Result<Self, SemanticError> {
        let mut file = File::open(path)?;

        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;
        if &magic != CACHE_MAGIC {
            return Err(SemanticError::Extraction(
                "Invalid cache file magic".to_string(),
            ));
        }

        let mut ver_bytes = [0u8; 4];
        file.read_exact(&mut ver_bytes)?;
        let _version = u32::from_le_bytes(ver_bytes);

        let mut crc_bytes = [0u8; 4];
        file.read_exact(&mut crc_bytes)?;
        let expected_checksum = u32::from_le_bytes(crc_bytes);

        let mut payload = Vec::new();
        file.read_to_end(&mut payload)?;

        let mut hasher = Hasher::new();
        hasher.update(&payload);
        if hasher.finalize() != expected_checksum {
            return Err(SemanticError::Extraction(
                "Cache file CRC32 checksum mismatch".to_string(),
            ));
        }

        let entries = bincode::deserialize(&payload).map_err(|e| {
            SemanticError::Extraction(format!("Failed to deserialize fingerprint cache: {e}"))
        })?;

        Ok(Self { entries })
    }
}

/// Snapshot of live indexing progress
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct IndexerProgress {
    pub total_discovered: usize,
    pub total_processed: usize,
    pub indexed_chunks: usize,
    pub current_file: Option<PathBuf>,
    pub is_indexing: bool,
    pub is_paused: bool,
    pub throttle_mode: ThrottleMode,
}

/// Asynchronous background worker orchestrating incremental folder indexing
pub struct BackgroundIndexer {
    engine: Arc<RwLock<HybridSearchEngine>>,
    cache: Arc<RwLock<FileFingerprintCache>>,
    progress: Arc<RwLock<IndexerProgress>>,
    is_running: Arc<AtomicBool>,
    is_paused: Arc<AtomicBool>,
    throttle_mode: Arc<RwLock<ThrottleMode>>,
    worker_handle: Option<JoinHandle<()>>,
}

impl BackgroundIndexer {
    /// Creates a new `BackgroundIndexer` attached to a shared `HybridSearchEngine`
    pub fn new(engine: Arc<RwLock<HybridSearchEngine>>) -> Self {
        let throttle_mode = ThrottleMode::detect_system_power_mode();
        let initial_progress = IndexerProgress {
            throttle_mode,
            ..Default::default()
        };

        Self {
            engine,
            cache: Arc::new(RwLock::new(FileFingerprintCache::new())),
            progress: Arc::new(RwLock::new(initial_progress)),
            is_running: Arc::new(AtomicBool::new(false)),
            is_paused: Arc::new(AtomicBool::new(false)),
            throttle_mode: Arc::new(RwLock::new(throttle_mode)),
            worker_handle: None,
        }
    }

    /// Access to fingerprint cache
    pub fn cache(&self) -> Arc<RwLock<FileFingerprintCache>> {
        Arc::clone(&self.cache)
    }

    /// Current progress snapshot
    pub fn progress(&self) -> IndexerProgress {
        self.progress.read().clone()
    }

    /// Sets operating throttle mode
    pub fn set_throttle_mode(&self, mode: ThrottleMode) {
        *self.throttle_mode.write() = mode;
        self.progress.write().throttle_mode = mode;
    }

    /// Pauses background indexing
    pub fn pause(&self) {
        self.is_paused.store(true, Ordering::SeqCst);
        self.progress.write().is_paused = true;
    }

    /// Resumes background indexing
    pub fn resume(&self) {
        self.is_paused.store(false, Ordering::SeqCst);
        self.progress.write().is_paused = false;
    }

    /// Signals worker thread to stop
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        self.progress.write().is_indexing = false;
    }

    /// Whether indexer is currently actively scanning or processing
    pub fn is_indexing(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    /// Synchronously scans and indexes a directory with change skipping
    pub fn index_directory_sync(&self, root: &Path) -> Result<usize, SemanticError> {
        let mut total_indexed_chunks = 0;

        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            let doc_type = DocumentType::from_extension(ext);
            if doc_type == DocumentType::Unknown {
                continue;
            }

            let metadata = match std::fs::metadata(path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let size = metadata.len();
            let mtime = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            let should = self.cache.read().should_index(path, mtime, size);
            if !should {
                continue;
            }

            if let Ok(extracted) = extract_content(path) {
                if let Ok(chunks) = self.engine.write().index_document(&extracted) {
                    self.cache
                        .write()
                        .update(path.to_path_buf(), mtime, size, 0);
                    total_indexed_chunks += chunks;
                }
            }

            let delay = self.throttle_mode.read().delay();
            if !delay.is_zero() {
                sleep(delay);
            }
        }

        Ok(total_indexed_chunks)
    }

    /// Handles single-file creation or modification from inotify watcher
    pub fn handle_file_created_or_modified(&self, path: &Path) -> Result<bool, SemanticError> {
        if !path.is_file() {
            return Ok(false);
        }

        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let doc_type = DocumentType::from_extension(ext);
        if doc_type == DocumentType::Unknown {
            return Ok(false);
        }

        let metadata = std::fs::metadata(path)?;
        let size = metadata.len();
        let mtime = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        if !self.cache.read().should_index(path, mtime, size) {
            return Ok(false);
        }

        let extracted = extract_content(path)?;
        let _ = self.engine.write().index_document(&extracted)?;
        self.cache
            .write()
            .update(path.to_path_buf(), mtime, size, 0);
        Ok(true)
    }

    /// Handles file deletion from inotify watcher
    pub fn handle_file_deleted(&self, path: &Path) -> bool {
        let removed_chunks = self.engine.write().remove_document(path);
        let removed_cache = self.cache.write().remove(path);
        removed_chunks > 0 || removed_cache
    }

    /// Starts asynchronous background indexing thread across a list of directories
    pub fn start_background_indexing(&mut self, paths: Vec<PathBuf>) -> Receiver<IndexerProgress> {
        self.stop();

        self.is_running.store(true, Ordering::SeqCst);
        self.is_paused.store(false, Ordering::SeqCst);

        let (progress_tx, progress_rx) = channel::<IndexerProgress>();

        let engine = Arc::clone(&self.engine);
        let cache = Arc::clone(&self.cache);
        let progress = Arc::clone(&self.progress);
        let is_running = Arc::clone(&self.is_running);
        let is_paused = Arc::clone(&self.is_paused);
        let throttle_mode = Arc::clone(&self.throttle_mode);

        let handle = thread::spawn(move || {
            // Step 1: Discover all candidate files
            let mut candidates = Vec::new();

            for root in &paths {
                if !root.exists() {
                    continue;
                }
                for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
                    if !is_running.load(Ordering::SeqCst) {
                        break;
                    }
                    let p = entry.path();
                    if p.is_file() {
                        let ext = p.extension().and_then(|s| s.to_str()).unwrap_or_default();
                        if DocumentType::from_extension(ext) != DocumentType::Unknown {
                            candidates.push(p.to_path_buf());
                        }
                    }
                }
            }

            {
                let mut p = progress.write();
                p.total_discovered = candidates.len();
                p.total_processed = 0;
                p.is_indexing = true;
                p.is_paused = false;
                let _ = progress_tx.send(p.clone());
            }

            // Step 2: Incrementally process candidate files
            for path in candidates {
                if !is_running.load(Ordering::SeqCst) {
                    break;
                }

                while is_paused.load(Ordering::SeqCst) {
                    if !is_running.load(Ordering::SeqCst) {
                        break;
                    }
                    sleep(Duration::from_millis(100));
                }

                let metadata = match std::fs::metadata(&path) {
                    Ok(m) => m,
                    Err(_) => {
                        let mut p = progress.write();
                        p.total_processed += 1;
                        let _ = progress_tx.send(p.clone());
                        continue;
                    }
                };

                let size = metadata.len();
                let mtime = metadata
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);

                let needs_index = cache.read().should_index(&path, mtime, size);

                if needs_index {
                    {
                        let mut p = progress.write();
                        p.current_file = Some(path.clone());
                        let _ = progress_tx.send(p.clone());
                    }

                    if let Ok(extracted) = extract_content(&path) {
                        if let Ok(chunks) = engine.write().index_document(&extracted) {
                            cache.write().update(path.clone(), mtime, size, 0);
                            let mut p = progress.write();
                            p.indexed_chunks += chunks;
                        }
                    }
                }

                {
                    let mut p = progress.write();
                    p.total_processed += 1;
                    let _ = progress_tx.send(p.clone());
                }

                let delay = throttle_mode.read().delay();
                if !delay.is_zero() {
                    sleep(delay);
                }
            }

            {
                let mut p = progress.write();
                p.is_indexing = false;
                p.current_file = None;
                let _ = progress_tx.send(p.clone());
            }
            is_running.store(false, Ordering::SeqCst);
        });

        self.worker_handle = Some(handle);
        progress_rx
    }
}

impl Drop for BackgroundIndexer {
    fn drop(&mut self) {
        self.stop();
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}
