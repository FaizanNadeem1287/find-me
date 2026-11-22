//! Asynchronous Tokio communication bridge and live engine event streaming
//!
//! Connects `findme-core` (compact memory arena, parallel crawler, nucleo matcher,
//! inotify watcher) with `findme-ui` over asynchronous Tokio channels, non-blocking
//! tasks, and Iced event subscriptions while strictly maintaining 60 FPS responsiveness.

use crate::message::ColumnId;
use findme_core::{
    Crawler, CrawlerConfig, FileArena, FileFlags, FsWatcher, FsWatcherConfig, QueryParser,
    SearchConfig, SearchEngine, SearchResultItem, SortColumn, SortDirection,
};
use iced::futures::{SinkExt, Stream};
use iced::stream;
use iced::Subscription;
use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use tokio::sync::broadcast;

/// Engine lifecycle and background filesystem events
#[derive(Debug, Clone, PartialEq)]
pub enum EngineEvent {
    /// Background crawl or indexing started
    Started,
    /// Live crawling progress batch
    Progress {
        files_found: u64,
        dirs_scanned: u64,
        current_path: String,
        percent: f32,
    },
    /// Index rebuild or reload completed successfully
    IndexReady {
        total_files: u64,
        total_folders: u64,
        memory_bytes: u64,
        duration_ms: u64,
    },
    /// Inotify detected filesystem change event
    FsChange { path: PathBuf, kind: String },
    /// Background engine error occurred
    Error(String),
}

static EVENT_BUS: OnceLock<broadcast::Sender<EngineEvent>> = OnceLock::new();

/// Returns reference to the global engine broadcast channel
pub fn get_event_bus() -> &'static broadcast::Sender<EngineEvent> {
    EVENT_BUS.get_or_init(|| {
        let (tx, _rx) = broadcast::channel(500);
        tx
    })
}

/// Creates a continuous stream of background engine events for Iced subscriptions
pub fn engine_stream() -> impl Stream<Item = EngineEvent> {
    stream::channel(100, async move |mut output| {
        let mut rx = get_event_bus().subscribe();
        while let Ok(event) = rx.recv().await {
            if output.send(event).await.is_err() {
                break;
            }
        }
    })
}

/// Returns an Iced subscription listening for real-time engine events
pub fn engine_subscription() -> Subscription<EngineEvent> {
    Subscription::run(engine_stream)
}

/// Thread-safe asynchronous engine client managing background tasks and the memory arena
pub struct EngineBridge {
    arena: Arc<RwLock<FileArena>>,
    watcher: Arc<RwLock<Option<FsWatcher>>>,
    event_tx: broadcast::Sender<EngineEvent>,
    is_indexing: Arc<AtomicBool>,
}

impl Default for EngineBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for EngineBridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let count = self.arena.read().len();
        f.debug_struct("EngineBridge")
            .field("arena_records", &count)
            .field("is_indexing", &self.is_indexing())
            .finish()
    }
}

impl EngineBridge {
    /// Creates a new `EngineBridge` pre-populated with standard default sample files
    ///
    /// Ensures instant startup responsiveness without disk I/O while providing
    /// rich data for immediate querying, table virtualisation, and tests.
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(500);
        Self::with_channel(Self::default_sample_arena(), event_tx)
    }

    /// Creates an empty `EngineBridge` with 0 initial files
    pub fn empty() -> Self {
        let (event_tx, _) = broadcast::channel(500);
        Self::with_channel(FileArena::new(), event_tx)
    }

    /// Creates an `EngineBridge` wrapping a specific `FileArena`
    pub fn with_arena(arena: FileArena) -> Self {
        let (event_tx, _) = broadcast::channel(500);
        Self::with_channel(arena, event_tx)
    }

    /// Creates an `EngineBridge` with an arena and an explicit event channel
    pub fn with_channel(arena: FileArena, event_tx: broadcast::Sender<EngineEvent>) -> Self {
        Self {
            arena: Arc::new(RwLock::new(arena)),
            watcher: Arc::new(RwLock::new(None)),
            event_tx,
            is_indexing: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Returns shared reference to the inner `FileArena`
    pub fn arena(&self) -> Arc<RwLock<FileArena>> {
        Arc::clone(&self.arena)
    }

    /// Returns whether background indexing is actively running
    pub fn is_indexing(&self) -> bool {
        self.is_indexing.load(Ordering::SeqCst)
    }

    /// Subscribes to engine events broadcast channel for this engine bridge
    pub fn subscribe_events(&self) -> broadcast::Receiver<EngineEvent> {
        self.event_tx.subscribe()
    }

    /// Manually broadcasts an engine event to this bridge's listeners and the global UI bus
    pub fn send_event(&self, event: EngineEvent) -> usize {
        let _ = get_event_bus().send(event.clone());
        self.event_tx.send(event).unwrap_or(0)
    }

    /// Asynchronously executes a search query off-thread via `tokio::task::spawn_blocking`
    ///
    /// Guarantees that query parsing, AST evaluation, Rayon fuzzy matching, and path
    /// reconstruction never block the UI thread, maintaining strict 60 FPS performance.
    pub async fn search(
        &self,
        query: &str,
        max_results: usize,
        sort_column: ColumnId,
        sort_asc: bool,
    ) -> (Vec<SearchResultItem>, u64, usize) {
        let arena_clone = Arc::clone(&self.arena);
        let query_owned = query.to_string();

        let handle = tokio::task::spawn_blocking(move || {
            let start = std::time::Instant::now();
            let arena = arena_clone.read();

            let ast = QueryParser::parse(&query_owned);
            let sort_col = match sort_column {
                ColumnId::Name => SortColumn::Name,
                ColumnId::Path => SortColumn::Path,
                ColumnId::Type => SortColumn::Name,
                ColumnId::Size => SortColumn::Size,
                ColumnId::Modified => SortColumn::Modified,
            };
            let sort_dir = if sort_asc {
                SortDirection::Ascending
            } else {
                SortDirection::Descending
            };

            let config = SearchConfig {
                max_results,
                sort_column: sort_col,
                sort_direction: sort_dir,
            };

            let matches = SearchEngine::search(&arena, &ast, &config);
            let total_matches = matches.len();

            let mut results = Vec::with_capacity(matches.len());
            for m in matches {
                if let Some(record) = arena.get(m.record_idx) {
                    if record.flags.is_deleted() {
                        continue;
                    }
                    if let Some(path) = arena.reconstruct_path(record) {
                        let name = arena
                            .get_record_name(record)
                            .unwrap_or_default()
                            .to_string();
                        results.push(SearchResultItem {
                            id: m.record_idx as u64,
                            name,
                            path: path.to_string_lossy().into_owned(),
                            size_bytes: record.size_bytes,
                            mtime_secs: record.mtime_secs,
                            is_dir: record.is_dir(),
                        });
                    }
                }
            }

            let elapsed_ms = start.elapsed().as_millis() as u64;
            (results, elapsed_ms, total_matches)
        });

        handle.await.unwrap_or_default()
    }

    /// Instantly purges arena records matching excluded rules without a full disk recrawl (<2ms)
    pub fn purge_matching<F>(&self, is_excluded: F) -> usize
    where
        F: FnMut(&str, &Path, bool) -> bool,
    {
        let mut arena = self.arena.write();
        arena.purge_matching(is_excluded)
    }

    /// Spawns a background filesystem crawler to re-index all configured roots with advanced rules
    pub fn rebuild_index_with_rules(
        &self,
        roots: Vec<PathBuf>,
        exclusions: Vec<PathBuf>,
        excluded_names: Vec<String>,
        excluded_extensions: Vec<String>,
        excluded_globs: Vec<String>,
    ) {
        if self.is_indexing.swap(true, Ordering::SeqCst) {
            // Already indexing, avoid starting duplicate concurrent rebuilds
            return;
        }

        let _ = get_event_bus().send(EngineEvent::Started);
        let _ = self.event_tx.send(EngineEvent::Started);

        let arena_clone = Arc::clone(&self.arena);
        let is_indexing_clone = Arc::clone(&self.is_indexing);
        let event_tx_clone = self.event_tx.clone();

        std::thread::spawn(move || {
            let mut config = CrawlerConfig::with_roots(roots);
            for exc in exclusions {
                config = config.add_exclusion(exc);
            }
            for name in excluded_names {
                config = config.add_excluded_name(name);
            }
            for ext in excluded_extensions {
                config = config.add_excluded_extension(ext);
            }
            for glob in excluded_globs {
                config = config.add_excluded_glob(glob);
            }
            config = config.skip_virtual_filesystems(true);

            let mut new_arena = FileArena::new();
            let crawler = Crawler::new(config);

            let tx_progress = event_tx_clone.clone();
            let crawl_res = crawler.crawl_with_progress(&mut new_arena, move |progress| {
                let current_path = match progress.current_directory {
                    Some(ref p) => p.to_string_lossy().to_string(),
                    None => "Scanning directory tree...".to_string(),
                };
                let ev = EngineEvent::Progress {
                    files_found: progress.files_indexed,
                    dirs_scanned: progress.dirs_scanned,
                    current_path,
                    percent: 0.0,
                };
                let _ = get_event_bus().send(ev.clone());
                let _ = tx_progress.send(ev);
            });

            match crawl_res {
                Ok(stats) => {
                    new_arena.rebuild_lookup();
                    let total_files = stats.files_indexed;
                    let total_folders = stats.dirs_scanned;
                    let memory_bytes = new_arena.memory_usage_bytes() as u64;
                    let duration_ms = stats.elapsed.as_millis() as u64;

                    {
                        let mut lock = arena_clone.write();
                        *lock = new_arena;
                    }
                    is_indexing_clone.store(false, Ordering::SeqCst);

                    let ev = EngineEvent::IndexReady {
                        total_files,
                        total_folders,
                        memory_bytes,
                        duration_ms,
                    };
                    let _ = get_event_bus().send(ev.clone());
                    let _ = event_tx_clone.send(ev);
                }
                Err(e) => {
                    is_indexing_clone.store(false, Ordering::SeqCst);
                    let ev = EngineEvent::Error(e.to_string());
                    let _ = get_event_bus().send(ev.clone());
                    let _ = event_tx_clone.send(ev);
                }
            }
        });
    }

    /// Spawns a background filesystem crawler to re-index all configured roots
    pub fn rebuild_index(
        &self,
        roots: Vec<PathBuf>,
        exclusions: Vec<PathBuf>,
        excluded_names: Vec<String>,
    ) {
        self.rebuild_index_with_rules(roots, exclusions, excluded_names, Vec::new(), Vec::new());
    }

    /// Clears the in-memory index database
    pub fn clear_index(&self) {
        {
            let mut lock = self.arena.write();
            *lock = FileArena::new();
        }
        let ev = EngineEvent::IndexReady {
            total_files: 0,
            total_folders: 0,
            memory_bytes: 0,
            duration_ms: 0,
        };
        let _ = get_event_bus().send(ev.clone());
        let _ = self.event_tx.send(ev);
    }

    /// Starts real-time inotify watcher over configured roots
    pub fn start_watcher(&self, roots: Vec<PathBuf>) -> Result<(), String> {
        let config = FsWatcherConfig {
            debounce_duration: std::time::Duration::from_millis(150),
            watch_paths: roots,
        };
        match FsWatcher::new(Arc::clone(&self.arena), config) {
            Ok(watcher) => {
                *self.watcher.write() = Some(watcher);
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        }
    }

    /// Populates standard development and system sample files for instant UI initialization
    pub fn default_sample_arena() -> FileArena {
        let mut arena = FileArena::new();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(1732000000);

        let sample_files = [
            // Rust project files
            (
                "/home/user/projects/find-me/Cargo.toml",
                FileFlags::NONE,
                1_850,
                now - 3600,
            ),
            (
                "/home/user/projects/find-me/Cargo.lock",
                FileFlags::NONE,
                48_320,
                now - 3600,
            ),
            (
                "/home/user/projects/find-me/README.md",
                FileFlags::NONE,
                8_420,
                now - 7200,
            ),
            (
                "/home/user/projects/find-me/crates/findme-core/src/lib.rs",
                FileFlags::NONE,
                1_240,
                now - 14400,
            ),
            (
                "/home/user/projects/find-me/crates/findme-core/src/arena.rs",
                FileFlags::NONE,
                15_820,
                now - 28800,
            ),
            (
                "/home/user/projects/find-me/crates/findme-core/src/crawler.rs",
                FileFlags::NONE,
                14_110,
                now - 43200,
            ),
            (
                "/home/user/projects/find-me/crates/findme-core/src/matcher.rs",
                FileFlags::NONE,
                18_650,
                now - 57600,
            ),
            (
                "/home/user/projects/find-me/crates/findme-core/src/watcher.rs",
                FileFlags::NONE,
                16_200,
                now - 72000,
            ),
            (
                "/home/user/projects/find-me/crates/findme-core/src/query.rs",
                FileFlags::NONE,
                17_400,
                now - 86400,
            ),
            (
                "/home/user/projects/find-me/crates/findme-ui/src/main.rs",
                FileFlags::NONE,
                850,
                now - 3600,
            ),
            (
                "/home/user/projects/find-me/crates/findme-ui/src/app.rs",
                FileFlags::NONE,
                24_300,
                now - 1800,
            ),
            (
                "/home/user/projects/find-me/crates/findme-ui/src/engine.rs",
                FileFlags::NONE,
                12_800,
                now - 900,
            ),
            (
                "/home/user/projects/find-me/crates/findme-ui/src/views/search.rs",
                FileFlags::NONE,
                3_450,
                now - 7200,
            ),
            (
                "/home/user/projects/find-me/crates/findme-ui/src/views/advanced.rs",
                FileFlags::NONE,
                12_900,
                now - 14400,
            ),
            (
                "/home/user/projects/find-me/crates/findme-ui/src/views/table.rs",
                FileFlags::NONE,
                6_700,
                now - 10800,
            ),
            // Documents
            (
                "/home/user/documents/taxes_2025.pdf",
                FileFlags::NONE,
                1_450_000,
                now - 86400 * 5,
            ),
            (
                "/home/user/documents/annual_report.docx",
                FileFlags::NONE,
                820_000,
                now - 86400 * 2,
            ),
            (
                "/home/user/documents/project_notes.txt",
                FileFlags::NONE,
                4_200,
                now - 600,
            ),
            // Pictures & Media
            (
                "/home/user/pictures/desktop_wallpaper.png",
                FileFlags::NONE,
                3_850_000,
                now - 86400 * 10,
            ),
            (
                "/home/user/pictures/vacation_photo.jpg",
                FileFlags::NONE,
                2_400_000,
                now - 86400 * 30,
            ),
            (
                "/home/user/music/soundtrack.mp3",
                FileFlags::NONE,
                8_950_000,
                now - 86400 * 15,
            ),
            (
                "/home/user/videos/rust_presentation.mp4",
                FileFlags::NONE,
                450_000_000,
                now - 86400 * 3,
            ),
            // Archives & Scripts
            (
                "/home/user/downloads/archive_backup.tar.gz",
                FileFlags::NONE,
                125_000_000,
                now - 86400,
            ),
            (
                "/home/user/downloads/distribution.zip",
                FileFlags::NONE,
                45_000_000,
                now - 86400 * 4,
            ),
            (
                "/home/user/scripts/deploy_cluster.sh",
                FileFlags::IS_EXECUTABLE,
                2_100,
                now - 86400 * 7,
            ),
            (
                "/home/user/scripts/sync_data.py",
                FileFlags::NONE,
                3_800,
                now - 86400 * 6,
            ),
            // Config & System
            (
                "/home/user/.config/find-me/settings.json",
                FileFlags::IS_HIDDEN,
                512,
                now - 120,
            ),
            (
                "/var/log/syslog",
                FileFlags::IS_READONLY,
                5_400_000,
                now - 60,
            ),
            ("/etc/hosts", FileFlags::IS_READONLY, 240, now - 86400 * 100),
            (
                "/usr/bin/find-me",
                FileFlags::IS_EXECUTABLE,
                14_800_000,
                now - 3600,
            ),
        ];

        for (path_str, flags, size, mtime) in sample_files {
            let path = Path::new(path_str);
            let _ = arena.add_file(path, flags, size, mtime);
        }

        arena.rebuild_lookup();
        arena
    }
}
