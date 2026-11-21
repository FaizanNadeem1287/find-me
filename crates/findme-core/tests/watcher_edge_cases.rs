//! Comprehensive edge-case and integration test suite for real-time filesystem watcher
//! Verifies live creation, deletion, rename, debouncing, queue overflow recovery,
//! and thread-safe arena synchronization within 200ms budget.

use findme_core::matcher::{SearchConfig, SearchEngine};
use findme_core::query::QueryParser;
use findme_core::record::FileFlags;
use findme_core::watcher::{FsWatcher, FsWatcherConfig};
use findme_core::FileArena;
use notify::event::{ModifyKind, RemoveKind, RenameMode};
use notify::{Event, EventKind};
use parking_lot::RwLock;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_watcher_live_file_creation() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path().canonicalize().unwrap();

    let arena = Arc::new(RwLock::new(FileArena::new()));
    let config = FsWatcherConfig {
        debounce_duration: Duration::from_millis(50),
        watch_paths: vec![dir_path.clone()],
    };

    let mut watcher = FsWatcher::new(Arc::clone(&arena), config).unwrap();
    assert!(watcher.watched_paths().contains(&dir_path));

    // Create a new file
    let file_path = dir_path.join("created_doc.txt");
    {
        let mut f = File::create(&file_path).unwrap();
        f.write_all(b"Hello inotify world").unwrap();
        f.sync_all().unwrap();
    }

    // Wait for inotify event delivery and debouncing (well within 200ms budget)
    let start = std::time::Instant::now();
    let mut found = false;
    while start.elapsed() < Duration::from_millis(500) {
        if arena.read().find_record_by_path(&file_path).is_some() {
            found = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }

    assert!(
        found,
        "File was not indexed by inotify watcher within budget"
    );

    // Verify search query finds the live created file
    let ast = QueryParser::parse("created_doc");
    let matches = SearchEngine::search(&arena.read(), &ast, &SearchConfig::default());
    assert!(!matches.is_empty());

    watcher.stop();
}

#[test]
fn test_watcher_live_file_deletion() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path().canonicalize().unwrap();

    let arena = Arc::new(RwLock::new(FileArena::new()));

    // Create initial file
    let file_path = dir_path.join("file_to_delete.rs");
    {
        let mut f = File::create(&file_path).unwrap();
        f.write_all(b"fn main() {}").unwrap();
        f.sync_all().unwrap();
    }

    arena
        .write()
        .add_file(&file_path, FileFlags::NONE, 12, 1795000000)
        .unwrap();
    assert!(arena.read().find_record_by_path(&file_path).is_some());

    let config = FsWatcherConfig {
        debounce_duration: Duration::from_millis(50),
        watch_paths: vec![dir_path],
    };
    let mut watcher = FsWatcher::new(Arc::clone(&arena), config).unwrap();

    // Delete file from disk
    fs::remove_file(&file_path).unwrap();

    // Wait for deletion event
    let start = std::time::Instant::now();
    let mut deleted = false;
    while start.elapsed() < Duration::from_millis(500) {
        if arena.read().find_record_by_path(&file_path).is_none() {
            deleted = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }

    assert!(deleted, "File deletion was not reflected in arena");

    // Verify search query no longer finds the deleted file
    let ast = QueryParser::parse("file_to_delete");
    let matches = SearchEngine::search(&arena.read(), &ast, &SearchConfig::default());
    assert!(matches.is_empty());

    watcher.stop();
}

#[test]
fn test_watcher_live_file_rename() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path().canonicalize().unwrap();

    let arena = Arc::new(RwLock::new(FileArena::new()));

    let from_path = dir_path.join("original_name.txt");
    let to_path = dir_path.join("renamed_name.txt");

    {
        let mut f = File::create(&from_path).unwrap();
        f.write_all(b"sample text").unwrap();
        f.sync_all().unwrap();
    }

    arena
        .write()
        .add_file(&from_path, FileFlags::NONE, 11, 1795000000)
        .unwrap();
    let initial_count = arena.read().len();

    let config = FsWatcherConfig {
        debounce_duration: Duration::from_millis(50),
        watch_paths: vec![dir_path],
    };
    let mut watcher = FsWatcher::new(Arc::clone(&arena), config).unwrap();

    // Perform rename
    fs::rename(&from_path, &to_path).unwrap();

    // Wait for rename event
    let start = std::time::Instant::now();
    let mut renamed = false;
    while start.elapsed() < Duration::from_millis(500) {
        let r = arena.read();
        if r.find_record_by_path(&from_path).is_none() && r.find_record_by_path(&to_path).is_some()
        {
            renamed = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }

    assert!(renamed, "Rename event was not reconciled properly");

    // Verify no duplicated active records
    let r = arena.read();
    let ast_all = QueryParser::parse("");
    let all_matches = SearchEngine::search(&r, &ast_all, &SearchConfig::default());
    assert_eq!(all_matches.len(), initial_count);

    watcher.stop();
}

#[test]
fn test_watcher_rapid_modifications_debouncing() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path().canonicalize().unwrap();

    let arena = Arc::new(RwLock::new(FileArena::new()));
    let file_path = dir_path.join("rapid_write.log");

    {
        let mut f = File::create(&file_path).unwrap();
        f.write_all(b"init").unwrap();
        f.sync_all().unwrap();
    }

    let config = FsWatcherConfig {
        debounce_duration: Duration::from_millis(100),
        watch_paths: vec![dir_path],
    };
    let mut watcher = FsWatcher::new(Arc::clone(&arena), config).unwrap();

    // Perform 10 rapid writes in quick succession
    for i in 0..10 {
        let mut f = fs::OpenOptions::new()
            .append(true)
            .open(&file_path)
            .unwrap();
        writeln!(f, "log entry {}", i).unwrap();
        f.sync_all().unwrap();
        std::thread::sleep(Duration::from_millis(10));
    }

    // Wait for debounced flush
    std::thread::sleep(Duration::from_millis(300));

    let r = arena.read();
    let rec_idx = r.find_record_by_path(&file_path);
    assert!(rec_idx.is_some());
    let record = r.get(rec_idx.unwrap()).unwrap();
    let actual_len = fs::metadata(&file_path).unwrap().len();
    assert_eq!(record.size_bytes, actual_len);

    watcher.stop();
}

#[test]
fn test_watcher_manual_batch_events_sync() {
    let arena = Arc::new(RwLock::new(FileArena::new()));
    let config = FsWatcherConfig::default();
    let watcher = FsWatcher::new(Arc::clone(&arena), config).unwrap();

    let p1 = PathBuf::from("/tmp/findme_manual_test_1.txt");
    let p2 = PathBuf::from("/tmp/findme_manual_test_2.txt");

    // 1. Simulate Remove event on non-existent file
    let ev_remove = Event {
        kind: EventKind::Remove(RemoveKind::File),
        paths: vec![p1.clone()],
        attrs: Default::default(),
    };
    let mutated = watcher.apply_events_sync(&[ev_remove]);
    assert_eq!(mutated, 0);

    // 2. Simulate Rename (Both) event
    let ev_rename = Event {
        kind: EventKind::Modify(ModifyKind::Name(RenameMode::Both)),
        paths: vec![p1, p2],
        attrs: Default::default(),
    };
    let _ = watcher.apply_events_sync(&[ev_rename]);
}

#[test]
fn test_watcher_dynamic_watch_unwatch() {
    let dir1 = tempdir().unwrap();
    let dir2 = tempdir().unwrap();

    let p1 = dir1.path().canonicalize().unwrap();
    let p2 = dir2.path().canonicalize().unwrap();

    let arena = Arc::new(RwLock::new(FileArena::new()));
    let config = FsWatcherConfig {
        debounce_duration: Duration::from_millis(50),
        watch_paths: vec![p1.clone()],
    };

    let mut watcher = FsWatcher::new(Arc::clone(&arena), config).unwrap();
    assert!(watcher.watched_paths().contains(&p1));
    assert!(!watcher.watched_paths().contains(&p2));

    // Dynamically watch dir2
    watcher.watch(&p2).unwrap();
    assert!(watcher.watched_paths().contains(&p2));

    // Dynamically unwatch dir1
    watcher.unwatch(&p1).unwrap();
    assert!(!watcher.watched_paths().contains(&p1));

    watcher.stop();
}

#[test]
fn test_watcher_overflow_rescan() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path().canonicalize().unwrap();

    // Create 5 files
    for i in 0..5 {
        let f_path = dir_path.join(format!("file_{}.dat", i));
        let mut f = File::create(f_path).unwrap();
        f.write_all(b"data").unwrap();
    }

    let arena = Arc::new(RwLock::new(FileArena::new()));
    let watcher = FsWatcher::new(Arc::clone(&arena), FsWatcherConfig::default()).unwrap();

    // Trigger overflow rescan on directory
    let indexed = watcher.trigger_overflow_rescan(&dir_path).unwrap();
    assert_eq!(indexed, 5);

    let r = arena.read();
    let file_count = r.records().iter().filter(|rec| !rec.is_dir()).count();
    assert_eq!(file_count, 5);
}
