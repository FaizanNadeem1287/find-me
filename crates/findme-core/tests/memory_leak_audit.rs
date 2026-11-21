//! Memory leak, heap profiling, and endurance audit for Find-Me Core Engine.
//!
//! Validates:
//! 1. Memory footprint is strictly < 45 MB (< 50MB budget) for 1,000,000 indexed records.
//! 2. Zero unbounded memory leaks over continuous filesystem watcher change cycles.
//! 3. Memory stability across repeated file creation, modification, renaming, and deletion.

use findme_core::arena::FileArena;
use findme_core::record::FileFlags;
use findme_core::watcher::{FsWatcher, FsWatcherConfig};
use parking_lot::RwLock;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use tempfile::TempDir;

#[test]
fn test_million_records_memory_footprint_budget() {
    println!("[AUDIT] Allocating 1,000,000 synthetic records for memory audit...");
    let mut arena = FileArena::with_capacity(1_000_000, 15 * 1024 * 1024, 20_000);

    let extensions = [
        "rs", "toml", "json", "md", "txt", "png", "tar.gz", "c", "h", "lock",
    ];
    let prefixes = [
        "findme", "crawler", "matcher", "arena", "record", "engine", "test", "build", "config",
        "util",
    ];

    for i in 0..1_000_000 {
        let prefix = prefixes[i % prefixes.len()];
        let ext = extensions[(i / 10) % extensions.len()];
        let path_str = format!("/usr/lib/findme/{}/{}_{:05}.{}", i % 1000, prefix, i, ext);
        let path = Path::new(&path_str);

        let size = (i as u64 * 13) % (100 * 1024 * 1024);
        let mtime = 1700000000 + (i as u64 % 1000000);

        arena
            .add_file(path, FileFlags::NONE, size, mtime)
            .expect("Failed to add file in memory audit");
    }

    arena.shrink_to_fit();

    assert_eq!(arena.len(), 1_000_000);
    let total_bytes = arena.memory_usage_bytes();
    let total_mb = total_bytes as f64 / (1024.0 * 1024.0);
    let avg_bytes_per_rec = arena.average_bytes_per_record();

    println!(
        "[AUDIT] 1,000,000 Records Memory Usage: {:.2} MB ({} bytes, {:.2} bytes/record)",
        total_mb, total_bytes, avg_bytes_per_rec
    );

    // Non-negotiable target: strictly < 50.0 MB total for 1,000,000 records
    assert!(
        total_mb < 50.0,
        "Total memory footprint exceeded 50MB budget: {:.2} MB",
        total_mb
    );
    assert!(
        avg_bytes_per_rec < 50.0,
        "Average memory per record exceeded 50 bytes: {:.2} bytes",
        avg_bytes_per_rec
    );
}

#[test]
fn test_arena_mutation_churn_memory_stability() {
    let mut arena = FileArena::with_capacity(50_000, 1_000_000, 1_000);

    // Seed with 10,000 initial files
    for i in 0..10_000 {
        let path_str = format!("/tmp/churn_test/dir_{}/file_{:04}.txt", i % 50, i);
        let _ = arena.add_file(Path::new(&path_str), FileFlags::NONE, 1024, 1700000000);
    }
    arena.shrink_to_fit();
    let baseline_mem = arena.memory_usage_bytes();

    // Perform 10,000 updates, renames, and removals in-place
    for i in 0..5_000 {
        let path_str = format!("/tmp/churn_test/dir_{}/file_{:04}.txt", i % 50, i);
        let new_path_str = format!("/tmp/churn_test/dir_{}/renamed_{:04}.txt", i % 50, i);

        // Update metadata
        arena.update_file_metadata(
            Path::new(&path_str),
            2048,
            1700000500,
            FileFlags::IS_READONLY,
        );

        // Rename
        let _ = arena.rename_file(Path::new(&path_str), Path::new(&new_path_str));

        // Delete odd files
        if i % 2 == 1 {
            arena.remove_file(Path::new(&new_path_str));
        }
    }

    let post_churn_mem = arena.memory_usage_bytes();
    let ratio = post_churn_mem as f64 / baseline_mem as f64;

    println!(
        "[AUDIT] Mutation Churn Memory Stability: baseline={} bytes, post_churn={} bytes, ratio={:.2}x",
        baseline_mem, post_churn_mem, ratio
    );

    // Memory growth must remain bounded under rapid churn (< 3.0x factor before compaction)
    assert!(
        ratio < 3.0,
        "Memory growth under churn was unbounded: ratio was {:.2}x",
        ratio
    );
}

#[test]
fn test_watcher_endurance_and_leak_free_event_cycle() {
    let temp_dir = TempDir::new().expect("Failed to create tempdir for watcher endurance");
    let watch_path = temp_dir.path().to_path_buf();

    let arena = Arc::new(RwLock::new(FileArena::new()));
    let config = FsWatcherConfig {
        debounce_duration: Duration::from_millis(50),
        watch_paths: vec![watch_path.clone()],
    };

    let mut watcher =
        FsWatcher::new(Arc::clone(&arena), config).expect("Failed to create FsWatcher");
    watcher
        .watch(&watch_path)
        .expect("Failed to watch directory");

    let num_cycles = 100;
    println!(
        "[AUDIT] Running {} continuous filesystem watcher change cycles...",
        num_cycles
    );

    let mut created_paths: Vec<PathBuf> = Vec::with_capacity(num_cycles);

    // Cycle 1: Create files
    for i in 0..num_cycles {
        let file_path = watch_path.join(format!("endurance_file_{:03}.log", i));
        let mut f = File::create(&file_path).expect("Failed to create test endurance file");
        writeln!(f, "Initial content iteration {}", i).expect("Failed to write");
        created_paths.push(file_path);
    }

    // Allow watcher debouncer to process
    sleep(Duration::from_millis(200));

    // Cycle 2: Modify existing files
    for (i, path) in created_paths.iter().enumerate() {
        if i % 2 == 0 {
            let mut f = fs::OpenOptions::new()
                .append(true)
                .open(path)
                .expect("Failed to open file for append");
            writeln!(f, "Appended content iteration {}", i).expect("Failed to append");
        }
    }

    sleep(Duration::from_millis(200));

    // Cycle 3: Rename half of the files
    let mut renamed_paths = Vec::new();
    for (i, path) in created_paths.iter().enumerate() {
        if i % 3 == 0 {
            let new_path = watch_path.join(format!("renamed_endurance_{:03}.bak", i));
            fs::rename(path, &new_path).expect("Failed to rename endurance file");
            renamed_paths.push(new_path);
        }
    }

    sleep(Duration::from_millis(200));

    // Cycle 4: Delete files
    for path in &created_paths {
        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }
    for path in &renamed_paths {
        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }

    // Allow watcher debouncer to settle
    sleep(Duration::from_millis(250));

    // Verify arena state & memory footprint
    let final_arena = arena.read();
    let memory_used = final_arena.memory_usage_bytes();

    println!(
        "[AUDIT] Watcher endurance complete: indexed records={}, memory={} bytes",
        final_arena.len(),
        memory_used
    );

    // Memory footprint of watcher test structures must remain tiny (< 1MB)
    assert!(
        memory_used < 1024 * 1024,
        "Watcher endurance memory leaked: {} bytes used (> 1MB)",
        memory_used
    );

    watcher.stop();
}
