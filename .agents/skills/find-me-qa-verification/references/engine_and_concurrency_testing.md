# Engine, Concurrency & Filesystem Testing Guide

This guide details best practices for testing Find-Me's low-level systems code: multithreaded directory crawling (`jwalk`), inotify kernel file watching (`notify`), and lock-free thread communication.

---

## 1. Testing Filesystem Traversal with `tempfile`

Never run filesystem tests against real home directories or `/tmp` root directly, as test files can collide or leave garbage behind. Always use scoped temporary directories:

```rust
use tempfile::tempdir;
use std::fs::{self, File};
use std::io::Write;

#[test]
fn test_parallel_crawler_scans_nested_hierarchy() {
    // 1. Create an isolated temporary directory (auto-deleted on drop)
    let temp = tempdir().expect("Failed to create temp dir");
    let root = temp.path();

    // 2. Build test directory fixture
    fs::create_dir_all(root.join("docs/subfolder")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();

    File::create(root.join("docs/subfolder/manual.pdf")).unwrap();
    File::create(root.join("src/main.rs")).unwrap();
    File::create(root.join(".hidden_file")).unwrap();

    // 3. Run parallel crawler
    let mut crawler = ParallelCrawler::new();
    let index = crawler.crawl_path(root).expect("Crawling failed");

    // 4. Assert correctness
    assert_eq!(index.file_count(), 3);
    assert_eq!(index.folder_count(), 3); // root, docs, subfolder, src
    assert!(index.contains_file("manual.pdf"));
    assert!(index.contains_file("main.rs"));
    assert!(index.contains_file(".hidden_file"));
}
```

---

## 2. Testing `inotify` Filesystem Monitoring Without Flakiness

A common mistake in file watcher tests is using `thread::sleep(Duration::from_millis(50))`. On busy CI servers or shared CPU cores, inotify delivery can take 100ms+, causing tests to intermittently fail.

### The Deterministic Testing Pattern:
Use Tokio asynchronous channels with an explicit bounded timeout:

```rust
use tempfile::tempdir;
use tokio::time::{timeout, Duration};
use tokio::sync::mpsc;

#[tokio::test]
async fn test_inotify_detects_file_creation() {
    let temp = tempdir().unwrap();
    let (tx, mut rx) = mpsc::channel(100);

    let mut watcher = FsWatcher::new(tx).unwrap();
    watcher.watch(temp.path()).unwrap();

    // Perform filesystem action
    let new_file = temp.path().join("created_by_test.txt");
    std::fs::File::create(&new_file).unwrap();

    // Await event with 3-second safety timeout
    let event = timeout(Duration::from_secs(3), rx.recv())
        .await
        .expect("Timed out waiting for inotify event")
        .expect("Channel closed prematurely");

    match event {
        FsEvent::Created(path) => assert_eq!(path, new_file),
        other => panic!("Expected FsEvent::Created, got {:?}", other),
    }
}
```

---

## 3. Concurrency & Thread-Safety Testing

### Testing In-Memory Arena Concurrent Reads and Writes
Find-Me uses `Arc<parking_lot::RwLock<IndexSnapshot>>` to allow thousands of read queries per second while background inotify threads apply batched updates:

```rust
use std::sync::Arc;
use parking_lot::RwLock;
use std::thread;

#[test]
fn test_concurrent_search_during_active_indexing() {
    let index = Arc::new(RwLock::new(IndexArena::new()));
    let mut handles = Vec::new();

    // Spawn 8 reader threads simulating user searches
    for _ in 0..8 {
        let index_clone = Arc::clone(&index);
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                let guard = index_clone.read();
                let _ = guard.search_exact("test");
            }
        }));
    }

    // Spawn 2 writer threads simulating live inotify modifications
    for i in 0..2 {
        let index_clone = Arc::clone(&index);
        handles.push(thread::spawn(move || {
            for j in 0..100 {
                let mut guard = index_clone.write();
                guard.insert_dummy_file(&format!("file_{}_{}.txt", i, j));
            }
        }));
    }

    // Ensure all threads complete without deadlock or data race
    for handle in handles {
        handle.join().unwrap();
    }
}
```
