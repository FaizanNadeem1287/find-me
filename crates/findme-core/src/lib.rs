//! Find-Me Core Engine
//!
//! High-performance, offline-first search and indexing engine for Linux.
//! Provides multithreaded directory traversal, compact memory arenas,
//! fuzzy and exact query evaluation, and Linux inotify event synchronization.

pub mod arena;
pub mod crawler;
pub mod directory;
pub mod error;
pub mod matcher;
pub mod network_fs;
pub mod pool;
pub mod query;
pub mod record;
pub mod storage;
pub mod watcher;

pub use arena::FileArena;
pub use crawler::{
    extract_flags, Crawler, CrawlerConfig, CrawlerProgress, CrawlerStats, CycleDetector,
};
pub use directory::{DirectoryEntry, DirectoryTable, ROOT_DIR_ID};
pub use error::{EngineError, Result};
pub use matcher::{MatchResult, SearchConfig, SearchEngine, SortColumn, SortDirection};
pub use network_fs::{
    detect_network_filesystem, is_network_path, parse_mountinfo_network_shares, NetworkFsType,
    NetworkPoller, NetworkShareConfig, PollStats,
};
pub use pool::StringPool;
pub use query::{
    parse_attrib_filter, parse_date_filter, parse_iso_date, parse_size_bytes, parse_size_filter,
    Lexer, QueryAst, QueryNode, QueryParser, SearchPredicate, Token, KNOWN_OPERATORS,
};
pub use record::{reconstruct_path, FileFlags, FileRecord, SearchResultItem};
pub use storage::{IndexStorage, StorageHeader, HEADER_SIZE, SCHEMA_VERSION, STORAGE_MAGIC};
pub use watcher::{FsWatcher, FsWatcherConfig};

/// Engine version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
