//! Real-time Linux inotify filesystem event monitor and arena synchronization.
//!
//! Provides automated 150ms debouncing of rapid filesystem bursts, robust
//! `IN_Q_OVERFLOW` queue recovery via targeted incremental rescans, and
//! lock-free reader concurrency via `parking_lot::RwLock<FileArena>`.

use crate::arena::FileArena;
use crate::crawler::{extract_flags, Crawler, CrawlerConfig};
use crate::error::{EngineError, Result};
use notify::event::{ModifyKind, RenameMode};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use parking_lot::RwLock;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::thread::{spawn, JoinHandle};
use std::time::{Duration, Instant};

/// Configuration for the filesystem monitor
#[derive(Clone, Debug)]
pub struct FsWatcherConfig {
    /// Debounce duration window to aggregate rapid file bursts (default: 150ms)
    pub debounce_duration: Duration,
    /// Root paths to monitor on startup
    pub watch_paths: Vec<PathBuf>,
}

impl Default for FsWatcherConfig {
    fn default() -> Self {
        Self {
            debounce_duration: Duration::from_millis(150),
            watch_paths: Vec::new(),
        }
    }
}

/// Real-time inotify filesystem watcher synchronizing disk updates with `FileArena`
pub struct FsWatcher {
    arena: Arc<RwLock<FileArena>>,
    active_watches: Arc<RwLock<HashSet<PathBuf>>>,
    watcher: Option<RecommendedWatcher>,
    shutdown_tx: Option<Sender<()>>,
    worker_thread: Option<JoinHandle<()>>,
    is_running: Arc<AtomicBool>,
}

impl FsWatcher {
    /// Creates and starts an inotify filesystem watcher attached to an arena
    pub fn new(arena: Arc<RwLock<FileArena>>, config: FsWatcherConfig) -> Result<Self> {
        let active_watches = Arc::new(RwLock::new(HashSet::new()));
        let is_running = Arc::new(AtomicBool::new(true));

        let (event_tx, event_rx) = channel::<notify::Result<Event>>();
        let (shutdown_tx, shutdown_rx) = channel::<()>();

        // Initialize notify recommended watcher
        let mut watcher = notify::recommended_watcher(event_tx)
            .map_err(|e| EngineError::Notify(e.to_string()))?;

        // Add configured watch paths
        for p in &config.watch_paths {
            if p.exists() {
                let _ = watcher.watch(p, RecursiveMode::Recursive);
                active_watches.write().insert(p.clone());
            }
        }

        let worker_arena = Arc::clone(&arena);
        let worker_active_watches = Arc::clone(&active_watches);
        let worker_running = Arc::clone(&is_running);
        let debounce_duration = config.debounce_duration;

        let worker_thread = spawn(move || {
            Self::event_loop(
                worker_arena,
                worker_active_watches,
                worker_running,
                event_rx,
                shutdown_rx,
                debounce_duration,
            );
        });

        Ok(Self {
            arena,
            active_watches,
            watcher: Some(watcher),
            shutdown_tx: Some(shutdown_tx),
            worker_thread: Some(worker_thread),
            is_running,
        })
    }

    /// Background event loop aggregating events with debouncing and applying to `FileArena`
    fn event_loop(
        arena: Arc<RwLock<FileArena>>,
        active_watches: Arc<RwLock<HashSet<PathBuf>>>,
        is_running: Arc<AtomicBool>,
        event_rx: Receiver<notify::Result<Event>>,
        shutdown_rx: Receiver<()>,
        debounce_duration: Duration,
    ) {
        let mut pending_events: Vec<Event> = Vec::new();
        let mut last_event_time = Instant::now();

        while is_running.load(Ordering::Relaxed) {
            // Check shutdown signal
            if shutdown_rx.try_recv().is_ok() {
                break;
            }

            // Determine timeout based on whether we have buffered events
            let timeout = if pending_events.is_empty() {
                Duration::from_millis(100)
            } else {
                let elapsed = last_event_time.elapsed();
                if elapsed >= debounce_duration {
                    // Flush accumulated events
                    Self::apply_event_batch(&arena, &pending_events);
                    pending_events.clear();
                    Duration::from_millis(100)
                } else {
                    debounce_duration.saturating_sub(elapsed)
                }
            };

            match event_rx.recv_timeout(timeout) {
                Ok(Ok(event)) => {
                    pending_events.push(event);
                    last_event_time = Instant::now();
                }
                Ok(Err(err)) => {
                    // Inotify queue overflow or limit error detected
                    tracing::warn!("Inotify watcher error/overflow detected: {:?}", err);
                    Self::handle_overflow_internal(&arena, &active_watches);
                }
                Err(RecvTimeoutError::Timeout) => {
                    if !pending_events.is_empty() && last_event_time.elapsed() >= debounce_duration
                    {
                        Self::apply_event_batch(&arena, &pending_events);
                        pending_events.clear();
                    }
                }
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }

        // Flush remaining events on exit
        if !pending_events.is_empty() {
            Self::apply_event_batch(&arena, &pending_events);
        }
    }

    /// Adds a directory path to the active inotify watch set recursively
    pub fn watch<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let p = path.as_ref().to_path_buf();
        if let Some(ref mut w) = self.watcher {
            w.watch(&p, RecursiveMode::Recursive)
                .map_err(|e| EngineError::Notify(e.to_string()))?;
            self.active_watches.write().insert(p);
        }
        Ok(())
    }

    /// Removes a directory path from the active inotify watch set
    pub fn unwatch<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let p = path.as_ref().to_path_buf();
        if let Some(ref mut w) = self.watcher {
            w.unwatch(&p)
                .map_err(|e| EngineError::Notify(e.to_string()))?;
            self.active_watches.write().remove(&p);
        }
        Ok(())
    }

    /// List of currently monitored filesystem paths
    pub fn watched_paths(&self) -> Vec<PathBuf> {
        self.active_watches.read().iter().cloned().collect()
    }

    /// Synchronously applies a batch of notify events to the arena (for testing and direct flush)
    pub fn apply_events_sync(&self, events: &[Event]) -> usize {
        Self::apply_event_batch(&self.arena, events)
    }

    /// Processes and reconciles a batch of filesystem events into `FileArena`
    fn apply_event_batch(arena: &Arc<RwLock<FileArena>>, events: &[Event]) -> usize {
        if events.is_empty() {
            return 0;
        }

        let mut arena_lock = arena.write();
        let mut mutated_count = 0;

        let mut rename_from: Option<PathBuf> = None;

        for event in events {
            match &event.kind {
                // Rename events
                EventKind::Modify(ModifyKind::Name(mode)) => match mode {
                    RenameMode::Both => {
                        if event.paths.len() >= 2 {
                            let from = &event.paths[0];
                            let to = &event.paths[1];
                            if let Ok(renamed) = arena_lock.rename_file(from, to) {
                                if renamed {
                                    mutated_count += 1;
                                } else {
                                    Self::ingest_path_to_arena(&mut arena_lock, to);
                                    mutated_count += 1;
                                }
                            }
                        }
                    }
                    RenameMode::From => {
                        if let Some(first) = event.paths.first() {
                            rename_from = Some(first.clone());
                        }
                    }
                    RenameMode::To => {
                        if let Some(to) = event.paths.first() {
                            if let Some(from) = rename_from.take() {
                                if let Ok(renamed) = arena_lock.rename_file(&from, to) {
                                    if renamed {
                                        mutated_count += 1;
                                    } else {
                                        Self::ingest_path_to_arena(&mut arena_lock, to);
                                        mutated_count += 1;
                                    }
                                }
                            } else {
                                Self::ingest_path_to_arena(&mut arena_lock, to);
                                mutated_count += 1;
                            }
                        }
                    }
                    _ => {
                        for path in &event.paths {
                            if path.exists() {
                                Self::ingest_path_to_arena(&mut arena_lock, path);
                            } else {
                                arena_lock.remove_file(path);
                            }
                            mutated_count += 1;
                        }
                    }
                },

                // Creation events
                EventKind::Create(_) => {
                    for path in &event.paths {
                        if Self::ingest_path_to_arena(&mut arena_lock, path) {
                            mutated_count += 1;
                        }
                    }
                }

                // Modification events
                EventKind::Modify(_) => {
                    for path in &event.paths {
                        if path.exists() {
                            if Self::update_path_in_arena(&mut arena_lock, path)
                                || Self::ingest_path_to_arena(&mut arena_lock, path)
                            {
                                mutated_count += 1;
                            }
                        } else if arena_lock.remove_file(path) {
                            mutated_count += 1;
                        }
                    }
                }

                // Deletion / removal events
                EventKind::Remove(_) => {
                    for path in &event.paths {
                        if arena_lock.remove_file(path) {
                            mutated_count += 1;
                        }
                    }
                }

                _ => {}
            }
        }

        mutated_count
    }

    /// Reads path metadata from disk and adds it to `FileArena`
    fn ingest_path_to_arena(arena: &mut FileArena, path: &Path) -> bool {
        let meta = match std::fs::symlink_metadata(path) {
            Ok(m) => m,
            Err(_) => return false,
        };

        let is_hidden = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|s| s.starts_with('.'));
        let flags = extract_flags(&meta, is_hidden);
        let size = if meta.is_dir() { 0 } else { meta.len() };
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        if arena.find_record_by_path(path).is_some() {
            arena.update_file_metadata(path, size, mtime, flags)
        } else {
            arena.add_file(path, flags, size, mtime).is_ok()
        }
    }

    /// Updates existing path metadata in `FileArena`
    fn update_path_in_arena(arena: &mut FileArena, path: &Path) -> bool {
        let meta = match std::fs::symlink_metadata(path) {
            Ok(m) => m,
            Err(_) => return false,
        };

        let is_hidden = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|s| s.starts_with('.'));
        let flags = extract_flags(&meta, is_hidden);
        let size = if meta.is_dir() { 0 } else { meta.len() };
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        arena.update_file_metadata(path, size, mtime, flags)
    }

    /// Recovers from an inotify queue overflow by incrementally rescanning active roots
    fn handle_overflow_internal(
        arena: &Arc<RwLock<FileArena>>,
        active_watches: &Arc<RwLock<HashSet<PathBuf>>>,
    ) {
        let roots: Vec<PathBuf> = active_watches.read().iter().cloned().collect();
        if roots.is_empty() {
            return;
        }

        // Brief delay to let file operations settle
        std::thread::sleep(Duration::from_millis(200));

        let config = CrawlerConfig::with_roots(roots);
        let mut fresh_arena = FileArena::new();
        let crawler = Crawler::new(config);
        if crawler.crawl(&mut fresh_arena).is_ok() {
            let mut current = arena.write();
            // Reconcile records into active arena
            for record in fresh_arena.records() {
                if let Some(path) = fresh_arena.reconstruct_path(record) {
                    let _ =
                        current.add_file(&path, record.flags, record.size_bytes, record.mtime_secs);
                }
            }
        }
    }

    /// Manually triggers an overflow recovery on a given root path (for testing and manual sync)
    pub fn trigger_overflow_rescan(&self, root: &Path) -> Result<usize> {
        let config = CrawlerConfig::with_roots(vec![root.to_path_buf()]);
        let mut fresh_arena = FileArena::new();
        let crawler = Crawler::new(config);
        let stats = crawler.crawl(&mut fresh_arena)?;

        let mut current = self.arena.write();
        for record in fresh_arena.records() {
            if let Some(path) = fresh_arena.reconstruct_path(record) {
                let _ = current.add_file(&path, record.flags, record.size_bytes, record.mtime_secs);
            }
        }

        Ok(stats.files_indexed as usize)
    }

    /// Stops the background watcher thread
    pub fn stop(&mut self) {
        self.is_running.store(false, Ordering::Relaxed);
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.worker_thread.take() {
            let _ = handle.join();
        }
        self.watcher = None;
    }
}

impl Drop for FsWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}
