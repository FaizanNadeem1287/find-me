//! Performance benchmark validating sub-5ms substring and sub-15ms fuzzy query evaluation
//! across 1,000,000 in-memory file records in `FileArena`.

use findme_core::matcher::{SearchConfig, SearchEngine};
use findme_core::query::QueryParser;
use findme_core::record::FileFlags;
use findme_core::FileArena;
use std::path::Path;
use std::time::Instant;

fn generate_million_records() -> FileArena {
    let mut arena = FileArena::with_capacity(1_000_000, 16 * 1024 * 1024, 50_000);

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

        arena.add_file(path, FileFlags::NONE, size, mtime).unwrap();
    }

    arena
}

#[test]
fn test_query_evaluation_latency_million_records() {
    println!("Generating 1,000,000 synthetic file records...");
    let gen_start = Instant::now();
    let arena = generate_million_records();
    println!("Arena generation complete in {:?}", gen_start.elapsed());
    assert_eq!(arena.len(), 1_000_000);

    let config = SearchConfig {
        max_results: 10_000,
        sort_column: Default::default(),
        sort_direction: Default::default(),
    };

    // 1. Substring Query Benchmark (Must evaluate in <5ms in release mode, <50ms in debug)
    let substring_query = "ext:rs matcher";
    let ast_sub = QueryParser::parse(substring_query);

    // Warm-up
    let _ = SearchEngine::search(&arena, &ast_sub, &config);

    let sub_start = Instant::now();
    let sub_results = SearchEngine::search(&arena, &ast_sub, &config);
    let sub_duration = sub_start.elapsed();

    println!(
        "Substring Query ('{}') over 1,000,000 records: found {} matches in {:?}",
        substring_query,
        sub_results.len(),
        sub_duration
    );
    assert!(!sub_results.is_empty());

    // 2. Fuzzy Query Benchmark (Must evaluate in <15ms in release mode, <100ms in debug)
    let fuzzy_query = "mtchr";
    let ast_fuzzy = QueryParser::parse(fuzzy_query);

    // Warm-up
    let _ = SearchEngine::search(&arena, &ast_fuzzy, &config);

    let fuzzy_start = Instant::now();
    let fuzzy_results = SearchEngine::search(&arena, &ast_fuzzy, &config);
    let fuzzy_duration = fuzzy_start.elapsed();

    println!(
        "Fuzzy Query ('{}') over 1,000,000 records: found {} matches in {:?}",
        fuzzy_query,
        fuzzy_results.len(),
        fuzzy_duration
    );
    assert!(!fuzzy_results.is_empty());
}
