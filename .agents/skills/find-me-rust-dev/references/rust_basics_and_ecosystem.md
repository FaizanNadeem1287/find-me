# Rust Basics, Concurrency & Systems Architecture

This guide covers the core Rust conventions, type patterns, concurrency models, and library choices used in the Find-Me project.

---

## 1. Rust Foundations & Best Practices

### Memory Safety & Ownership
- **No Unnecessary Clones**: Avoid `clone()` in hot loops. Store paths and strings in an interned arena (`StringPool` or `lasso`) and pass lightweight indices (`u32` or string slices `&str`).
- **RAII & Scope Guards**: Resource cleanup (file handles, database locks, inotify watches) must rely on standard `Drop` implementations.
- **Error Handling**:
  - Internal engine crates: Use `thiserror` to define precise, typed enum errors.
  - UI layer / top-level handlers: Use `anyhow` or display user-friendly error dialogs/notifications.
  - Never call `unwrap()` or `expect()` in production code where unexpected user input or filesystem state can trigger a panic.

```rust
// Idiomatic typed error definition
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("I/O error occurred while accessing '{path}': {source}")]
    Io {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Failed to parse query '{query}': {reason}")]
    QuerySyntax {
        query: String,
        reason: String,
    },
    #[error("Index storage corrupted: {0}")]
    CorruptedIndex(String),
}
```

---

## 2. Concurrency & Asynchronous Architecture

### Engine vs. UI Concurrency
Find-Me uses a hybrid concurrency model:
1. **CPU-bound Parallelism (Rayon)**: Multithreaded filesystem crawling, batch scanning, sorting, and filter evaluation across all available CPU cores.
2. **I/O & Event Asynchrony (Tokio)**: Handling inotify filesystem event streams, background database writes, periodic reconciliation timers, and async message passing to the UI.
3. **Cross-Thread Communication**:
   - `tokio::sync::mpsc`: UI command channel (`UiCommand::Search(query)`, `UiCommand::RebuildIndex`, `UiCommand::CancelSearch`).
   - `tokio::sync::watch` or `tokio::sync::broadcast`: Engine state broadcasts (`EngineState::Indexing { progress, total }`, `EngineState::Ready`).
   - `std::sync::Arc<parking_lot::RwLock<IndexSnapshot>>`: Lock-free / read-biased shared in-memory index access for zero-latency queries.

```rust
// Clean channel pattern between UI and Background Engine
pub enum EngineCommand {
    Search {
        query: String,
        filters: SearchFilters,
        limit: usize,
    },
    RebuildIndex,
    AddWatchPath(std::path::PathBuf),
    RemoveWatchPath(std::path::PathBuf),
}

pub enum EngineEvent {
    SearchCompleted {
        query: String,
        results: Vec<SearchResultItem>,
        duration_ms: u64,
        total_matches: usize,
    },
    IndexingProgress {
        indexed_files: usize,
        indexed_folders: usize,
        current_path: String,
    },
    IndexingFinished {
        total_items: usize,
        database_size_bytes: u64,
        elapsed_seconds: f64,
    },
    Error(String),
}
```

---

## 3. High Performance Crate Ecosystem

Crate | Purpose | Justification
:--- | :--- | :---
`tokio` | Async runtime | Non-blocking background worker management, event streams
`rayon` | Work-stealing thread pool | Parallel directory traversal, parallel filtering/sorting
`parking_lot` | High-performance sync | Faster, smaller `RwLock` and `Mutex` than standard library
`jwalk` | Parallel filesystem traversal | Rayon-powered recursive directory walk, sorted output, skips ignores
`notify` | Filesystem notifications | Cross-platform abstraction over Linux `inotify`
`nucleo-matcher` | High-speed fuzzy/sub matching | 6-8x faster than traditional fuzzy matchers, dual-matrix Smith-Waterman
`regex` | Regular expression engine | Guaranteed linear-time regex execution (no ReDoS vulnerability)
`rkyv` / `bincode` | Binary serialization | Zero-copy deserialization for instantaneous index loading (<100ms for 1M records)
`memmap2` | Memory-mapped I/O | Memory mapping index files for low-RAM instant queries
`directories` | XDG directory resolution | Standard Linux paths (`~/.config/find-me`, `~/.cache/find-me`)
