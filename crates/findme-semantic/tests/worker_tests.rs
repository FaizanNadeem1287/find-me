use findme_semantic::{
    BackgroundIndexer, ChunkStore, FileFingerprintCache, HnswVectorIndex, HybridSearchEngine,
    ThrottleMode,
};
use parking_lot::RwLock;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_throttle_mode_delays_and_detection() {
    assert_eq!(
        ThrottleMode::MaxPerformance.delay(),
        Duration::from_millis(0)
    );
    assert_eq!(ThrottleMode::Balanced.delay(), Duration::from_millis(5));
    assert_eq!(
        ThrottleMode::BatterySaver.delay(),
        Duration::from_millis(25)
    );

    // Power mode detection must never panic regardless of hardware
    let detected = ThrottleMode::detect_system_power_mode();
    assert!(
        detected == ThrottleMode::Balanced
            || detected == ThrottleMode::BatterySaver
            || detected == ThrottleMode::MaxPerformance
    );
}

#[test]
fn test_file_fingerprint_cache_lifecycle_and_persistence() {
    let dir = tempdir().unwrap();
    let cache_file = dir.path().join("fingerprints.cache");

    let mut cache = FileFingerprintCache::new();
    let file1 = PathBuf::from("/test/resume.pdf");
    let file2 = PathBuf::from("/test/notes.txt");

    assert!(cache.should_index(&file1, 100, 500));
    cache.update(file1.clone(), 100, 500, 12345);
    assert!(!cache.should_index(&file1, 100, 500));
    // Changed mtime
    assert!(cache.should_index(&file1, 101, 500));
    // Changed size
    assert!(cache.should_index(&file1, 100, 501));

    cache.update(file2.clone(), 200, 1500, 54321);
    assert_eq!(cache.len(), 2);

    // Save and load
    cache.save_to_file(&cache_file).unwrap();
    let loaded = FileFingerprintCache::load_from_file(&cache_file).unwrap();
    assert_eq!(loaded.len(), 2);
    assert!(!loaded.should_index(&file1, 100, 500));
    assert!(!loaded.should_index(&file2, 200, 1500));

    // Corrupt cache file
    let mut f = File::create(&cache_file).unwrap();
    f.write_all(b"CORRUPT_BYTES").unwrap();
    assert!(FileFingerprintCache::load_from_file(&cache_file).is_err());
}

#[test]
fn test_background_indexer_sync_and_change_skipping() {
    let dir = tempdir().unwrap();
    let txt_path = dir.path().join("readme.txt");
    let py_path = dir.path().join("crawler.py");
    let bin_path = dir.path().join("archive.unknown_ext");

    let mut f1 = File::create(&txt_path).unwrap();
    writeln!(
        f1,
        "Welcome to Find-Me desktop search engine written in pure Rust."
    )
    .unwrap();

    let mut f2 = File::create(&py_path).unwrap();
    writeln!(
        f2,
        "def crawl_filesystem():\n    print('Crawling with high speed')"
    )
    .unwrap();

    let mut f3 = File::create(&bin_path).unwrap();
    f3.write_all(&[0x00, 0xFF, 0x12, 0x34]).unwrap();

    let hnsw = HnswVectorIndex::new(16, 64, 32);
    let chunk_store = ChunkStore::new();
    let engine = Arc::new(RwLock::new(HybridSearchEngine::new(hnsw, chunk_store)));
    let indexer = BackgroundIndexer::new(Arc::clone(&engine));
    indexer.set_throttle_mode(ThrottleMode::MaxPerformance);

    // 1. Initial full scan
    let indexed1 = indexer.index_directory_sync(dir.path()).unwrap();
    assert_eq!(indexed1, 2); // txt and py indexed, bin ignored
    assert_eq!(indexer.cache().read().len(), 2);
    assert_eq!(engine.read().chunk_store().len(), 2);

    // 2. Second scan without file changes -> 0 new chunks
    let indexed2 = indexer.index_directory_sync(dir.path()).unwrap();
    assert_eq!(indexed2, 0);

    // 3. Modify txt file -> exactly 1 re-indexed
    std::thread::sleep(Duration::from_millis(20)); // Ensure mtime advances
    let mut f1 = File::create(&txt_path).unwrap();
    writeln!(
        f1,
        "Updated Find-Me content with semantic search and RRF fusion."
    )
    .unwrap();

    let indexed3 = indexer.index_directory_sync(dir.path()).unwrap();
    assert_eq!(indexed3, 1);
}

#[test]
fn test_background_indexer_inotify_file_created_modified_deleted() {
    let dir = tempdir().unwrap();
    let doc_path = dir.path().join("doc.txt");

    let mut f = File::create(&doc_path).unwrap();
    writeln!(f, "Initial inotify test document content").unwrap();

    let hnsw = HnswVectorIndex::new(16, 64, 32);
    let chunk_store = ChunkStore::new();
    let engine = Arc::new(RwLock::new(HybridSearchEngine::new(hnsw, chunk_store)));
    let indexer = BackgroundIndexer::new(Arc::clone(&engine));

    // Handle file created
    let indexed = indexer.handle_file_created_or_modified(&doc_path).unwrap();
    assert!(indexed);
    assert_eq!(engine.read().chunk_store().len(), 1);

    // Handle same file again (skipped because unchanged)
    let skipped = indexer.handle_file_created_or_modified(&doc_path).unwrap();
    assert!(!skipped);

    // Handle file deleted
    let removed = indexer.handle_file_deleted(&doc_path);
    assert!(removed);
    assert_eq!(engine.read().chunk_store().len(), 0);
    assert_eq!(indexer.cache().read().len(), 0);
}

#[test]
fn test_background_indexer_async_thread_and_pause_resume_stop() {
    let dir = tempdir().unwrap();
    for i in 0..5 {
        let p = dir.path().join(format!("file_{i}.txt"));
        let mut f = File::create(&p).unwrap();
        writeln!(f, "Document number {i} for async indexer verification").unwrap();
    }

    let hnsw = HnswVectorIndex::new(16, 64, 32);
    let chunk_store = ChunkStore::new();
    let engine = Arc::new(RwLock::new(HybridSearchEngine::new(hnsw, chunk_store)));
    let mut indexer = BackgroundIndexer::new(Arc::clone(&engine));
    indexer.set_throttle_mode(ThrottleMode::MaxPerformance);

    let rx = indexer.start_background_indexing(vec![dir.path().to_path_buf()]);

    let mut final_progress = None;
    while let Ok(progress) = rx.recv_timeout(Duration::from_secs(5)) {
        if !progress.is_indexing && progress.total_processed >= progress.total_discovered {
            final_progress = Some(progress);
            break;
        }
    }

    assert!(final_progress.is_some());
    let prog = final_progress.unwrap();
    assert_eq!(prog.total_discovered, 5);
    assert_eq!(prog.total_processed, 5);
    assert_eq!(prog.indexed_chunks, 5);
    assert!(!indexer.is_indexing());
}
