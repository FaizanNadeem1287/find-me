//! Find-Me Core Engine
//!
//! High-performance, offline-first search and indexing engine for Linux.
//! Provides multithreaded directory traversal, compact memory arenas,
//! fuzzy and exact query evaluation, and Linux inotify event synchronization.

pub mod arena;
pub mod crawler;
pub mod directory;
pub mod error;
pub mod pool;
pub mod record;

pub use arena::FileArena;
pub use crawler::{
    extract_flags, Crawler, CrawlerConfig, CrawlerProgress, CrawlerStats, CycleDetector,
};
pub use directory::{DirectoryEntry, DirectoryTable, ROOT_DIR_ID};
pub use error::{EngineError, Result};
pub use pool::StringPool;
pub use record::{reconstruct_path, FileFlags, FileRecord, SearchResultItem};

/// Engine version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
