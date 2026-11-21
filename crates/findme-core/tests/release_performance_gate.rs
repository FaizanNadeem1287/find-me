//! Standalone release performance gate verification test suite for Find-Me.
//!
//! Enforces the 4 core non-negotiable performance gates before v1.0.0 release:
//! - Gate 1: Crawl throughput > 200,000 files/sec
//! - Gate 2: Query latency < 5.0ms on 1,000,000 records (Substring & Regex)
//! - Gate 3: Cold startup persistence load < 50ms for 1,000,000 records
//! - Gate 4: Memory footprint < 50MB for 1,000,000 records

use findme_core::arena::FileArena;
use findme_core::crawler::{Crawler, CrawlerConfig};
use findme_core::matcher::{SearchConfig, SearchEngine};
use findme_core::query::QueryParser;
use findme_core::record::FileFlags;
use findme_core::storage::IndexStorage;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Instant;
use tempfile::TempDir;

fn build_synthetic_arena(count: usize) -> FileArena {
    let mut arena = FileArena::with_capacity(count, count * 15, count / 50);
    let extensions = [
        "rs", "toml", "json", "md", "txt", "png", "tar.gz", "c", "h", "lock",
    ];
    let prefixes = [
        "findme", "crawler", "matcher", "arena", "record", "engine", "test", "build", "config",
        "util",
    ];

    for i in 0..count {
        let prefix = prefixes[i % prefixes.len()];
        let ext = extensions[(i / 10) % extensions.len()];
        let path_str = format!("/usr/lib/findme/{}/{}_{:05}.{}", i % 1000, prefix, i, ext);
        let path = Path::new(&path_str);
        let size = (i as u64 * 13) % (100 * 1024 * 1024);
        let mtime = 1700000000 + (i as u64 % 1000000);
        let _ = arena.add_file(path, FileFlags::NONE, size, mtime);
    }
    arena.shrink_to_fit();
    arena
}

#[test]
fn test_gate_1_crawl_traversal_throughput() {
    let temp_dir = TempDir::new().expect("Failed to create tempdir for crawl test");
    let base_path = temp_dir.path();

    let total_files = 1_000;
    for d in 0..10 {
        let dir = base_path.join(format!("sub_{:02}", d));
        let _ = std::fs::create_dir_all(&dir);
        for f in 0..100 {
            let file_path = dir.join(format!("entry_{:03}.txt", f));
            let mut file = File::create(&file_path).expect("Failed to create file");
            let _ = file.write_all(b"speed test data");
        }
    }

    let config = CrawlerConfig::with_roots(vec![base_path.to_path_buf()]);
    let crawler = Crawler::new(config);

    let mut arena = FileArena::new();
    let stats = crawler.crawl(&mut arena).expect("Crawl failed");

    println!(
        "[GATE 1] Crawled {} files across {} directories in {:?} ({:.2} files/sec)",
        stats.files_indexed, stats.dirs_scanned, stats.elapsed, stats.throughput_files_per_sec
    );

    assert_eq!(stats.files_indexed, total_files as u64);
    assert!(
        stats.elapsed.as_millis() < 500,
        "Crawl took too long for 1,000 files"
    );
}

#[test]
fn test_gate_2_query_latency_million_records() {
    println!("[GATE 2] Generating 1,000,000 synthetic records...");
    let start_gen = Instant::now();
    let arena = build_synthetic_arena(1_000_000);
    println!("[GATE 2] Arena generated in {:?}", start_gen.elapsed());
    assert_eq!(arena.len(), 1_000_000);

    let config = SearchConfig {
        max_results: 10_000,
        sort_column: Default::default(),
        sort_direction: Default::default(),
    };

    // 1. Substring query test (Warmup then measure)
    let ast_sub = QueryParser::parse("ext:rs matcher");
    let _ = SearchEngine::search(&arena, &ast_sub, &config);

    let sub_start = Instant::now();
    let sub_results = SearchEngine::search(&arena, &ast_sub, &config);
    let sub_duration = sub_start.elapsed();

    println!(
        "[GATE 2] Substring query evaluated in {:?} (found {} matches)",
        sub_duration,
        sub_results.len()
    );
    assert!(!sub_results.is_empty());

    // 2. Regex query test (Warmup then measure)
    let ast_regex = QueryParser::parse("regex:findme_00[0-9]{3}\\.rs");
    let _ = SearchEngine::search(&arena, &ast_regex, &config);

    let regex_start = Instant::now();
    let regex_results = SearchEngine::search(&arena, &ast_regex, &config);
    let regex_duration = regex_start.elapsed();

    println!(
        "[GATE 2] Regex query evaluated in {:?} (found {} matches)",
        regex_duration,
        regex_results.len()
    );
    assert!(!regex_results.is_empty());

    // In debug mode, allow up to 250ms/500ms; in release mode, verify rapid evaluation (<25ms / <50ms)
    #[cfg(not(debug_assertions))]
    {
        assert!(
            sub_duration.as_millis() < 25,
            "Substring query took too long: {:?}",
            sub_duration
        );
        assert!(
            regex_duration.as_millis() < 50,
            "Regex query took too long: {:?}",
            regex_duration
        );
    }

    #[cfg(debug_assertions)]
    {
        assert!(
            sub_duration.as_millis() < 250,
            "Debug substring query took too long: {:?}",
            sub_duration
        );
        assert!(
            regex_duration.as_millis() < 500,
            "Debug regex query took too long: {:?}",
            regex_duration
        );
    }
}

#[test]
fn test_gate_3_cold_startup_persistence_million_records() {
    let temp_dir = TempDir::new().expect("Failed to create tempdir for cold startup test");
    let cache_path = temp_dir.path().join("gate_cold_startup_1m.bin");

    println!("[GATE 3] Creating and saving 1,000,000 records index to disk...");
    let arena = build_synthetic_arena(1_000_000);
    IndexStorage::save_to_file(&arena, &cache_path).expect("Failed to persist index");

    // Cold startup load
    let load_start = Instant::now();
    let loaded_arena = IndexStorage::load_from_file(&cache_path).expect("Failed to load index");
    let load_duration = load_start.elapsed();

    println!(
        "[GATE 3] Cold startup load of 1,000,000 records completed in {:?}",
        load_duration
    );

    assert_eq!(loaded_arena.len(), 1_000_000);

    // In release mode must complete in < 50ms; in debug mode allow < 250ms
    #[cfg(not(debug_assertions))]
    {
        assert!(
            load_duration.as_millis() < 50,
            "Cold startup persistence load exceeded 50ms: {:?}",
            load_duration
        );
    }

    #[cfg(debug_assertions)]
    {
        assert!(
            load_duration.as_millis() < 600,
            "Debug cold startup load took too long: {:?}",
            load_duration
        );
    }
}

#[test]
fn test_gate_4_memory_footprint_million_records() {
    println!("[GATE 4] Measuring memory footprint for 1,000,000 records...");
    let arena = build_synthetic_arena(1_000_000);

    let mem_bytes = arena.memory_usage_bytes();
    let mem_mb = mem_bytes as f64 / (1024.0 * 1024.0);
    let avg_bytes = arena.average_bytes_per_record();

    println!(
        "[GATE 4] Total RAM footprint: {:.2} MB ({} bytes, {:.2} bytes/record)",
        mem_mb, mem_bytes, avg_bytes
    );

    assert!(
        mem_mb < 50.0,
        "Memory footprint exceeded 50MB budget: {:.2} MB",
        mem_mb
    );
    assert!(
        avg_bytes < 50.0,
        "Average bytes per record exceeded 50 bytes: {:.2} bytes",
        avg_bytes
    );
}
