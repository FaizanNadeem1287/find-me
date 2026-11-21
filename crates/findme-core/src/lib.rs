//! Find-Me Core Engine
//!
//! High-performance, offline-first search and indexing engine for Linux.
//! Provides multithreaded directory traversal, compact memory arenas,
//! fuzzy and exact query evaluation, and Linux inotify event synchronization.

pub mod directory;
pub mod error;
pub mod record;

pub use directory::{DirectoryEntry, DirectoryTable};
pub use error::{EngineError, Result};
pub use record::{FileFlags, FileRecord, SearchResultItem};

/// Engine version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
