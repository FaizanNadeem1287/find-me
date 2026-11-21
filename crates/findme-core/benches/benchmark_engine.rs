//! Comprehensive Criterion performance benchmarks for Find-Me Core Engine.
//!
//! Validates the 4 non-negotiable performance budgets:
//! 1. SSD Crawl Throughput: > 200,000 files / sec
//! 2. Query Latency: < 5.0ms on 1,000,000 records (Substring & Regex)
//! 3. Cold Startup Latency: < 50ms reading memory-mapped binary cache
//! 4. Memory Footprint: < 50MB RAM for 1,000,000 indexed file records (<45 bytes/record)

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use findme_core::arena::FileArena;
use findme_core::crawler::{Crawler, CrawlerConfig};
use findme_core::matcher::{SearchConfig, SearchEngine};
use findme_core::query::QueryParser;
use findme_core::record::FileFlags;
use findme_core::storage::IndexStorage;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;
use tempfile::TempDir;

/// Global lazily-initialized 1,000,000 record synthetic arena for benchmarks
static ARENA_1M: OnceLock<FileArena> = OnceLock::new();

/// Global lazily-initialized persisted binary index path and tempdir
static PERSISTED_1M: OnceLock<(TempDir, PathBuf)> = OnceLock::new();

/// Generates a synthetic in-memory arena with `count` records
fn generate_synthetic_arena(count: usize) -> FileArena {
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

fn get_arena_1m() -> &'static FileArena {
    ARENA_1M.get_or_init(|| generate_synthetic_arena(1_000_000))
}

fn get_persisted_1m() -> &'static (TempDir, PathBuf) {
    PERSISTED_1M.get_or_init(|| {
        let temp_dir = TempDir::new().expect("Failed to create tempdir for persistence bench");
        let cache_path = temp_dir.path().join("bench_index_1m.bin");
        let arena = get_arena_1m();
        IndexStorage::save_to_file(arena, &cache_path)
            .expect("Failed to persist 1M record index for bench");
        (temp_dir, cache_path)
    })
}

/// 1. Crawl Throughput Benchmark
fn bench_crawl_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("crawl_throughput");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(3));

    // Create a temporary hierarchy with 2,000 physical files
    let temp_dir = TempDir::new().expect("Failed to create tempdir for crawl bench");
    let base_path = temp_dir.path();

    for d in 0..20 {
        let dir = base_path.join(format!("dir_{:02}", d));
        let _ = std::fs::create_dir_all(&dir);
        for f in 0..100 {
            let file_path = dir.join(format!("file_{:03}.txt", f));
            let mut file = File::create(&file_path).expect("Failed to create test file");
            let _ = file.write_all(b"benchmark data");
        }
    }

    let config = CrawlerConfig::with_roots(vec![base_path.to_path_buf()]);
    let crawler = Crawler::new(config);

    group.throughput(Throughput::Elements(2000));
    group.bench_function("crawl_2k_hierarchy", |b| {
        b.iter(|| {
            let mut arena = FileArena::new();
            let stats = crawler.crawl(&mut arena).expect("crawl failed");
            black_box(stats.files_indexed)
        });
    });

    group.finish();
}

/// 2. Query Latency Benchmark across 1,000,000 Records
fn bench_query_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_latency");
    group.sample_size(15);
    group.measurement_time(Duration::from_secs(3));

    let arena = get_arena_1m();
    let config = SearchConfig {
        max_results: 10_000,
        sort_column: Default::default(),
        sort_direction: Default::default(),
    };

    // Substring query (<5ms target)
    let ast_sub = QueryParser::parse("ext:rs matcher");
    group.bench_function("substring_ext_rs_matcher_1m", |b| {
        b.iter(|| {
            let results = SearchEngine::search(arena, black_box(&ast_sub), black_box(&config));
            black_box(results.len())
        });
    });

    // Regex query (<5ms target)
    let ast_regex = QueryParser::parse("regex:findme_00[0-9]{3}\\.rs");
    group.bench_function("regex_pattern_1m", |b| {
        b.iter(|| {
            let results = SearchEngine::search(arena, black_box(&ast_regex), black_box(&config));
            black_box(results.len())
        });
    });

    // Fuzzy query (<15ms target)
    let ast_fuzzy = QueryParser::parse("mtchr");
    group.bench_function("fuzzy_nucleo_1m", |b| {
        b.iter(|| {
            let results = SearchEngine::search(arena, black_box(&ast_fuzzy), black_box(&config));
            black_box(results.len())
        });
    });

    // Compound multi-criteria filter
    let ast_compound = QueryParser::parse("ext:rs size:>100kb");
    group.bench_function("compound_size_ext_1m", |b| {
        b.iter(|| {
            let results = SearchEngine::search(arena, black_box(&ast_compound), black_box(&config));
            black_box(results.len())
        });
    });

    group.finish();
}

/// 3. Cold Startup Persistence Benchmark
fn bench_cold_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("cold_startup");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(3));

    let (_temp_dir, cache_path) = get_persisted_1m();

    group.throughput(Throughput::Elements(1_000_000));
    group.bench_function("load_from_file_1m_records", |b| {
        b.iter(|| {
            let arena = IndexStorage::load_from_file(black_box(cache_path))
                .expect("Failed to load binary index");
            black_box(arena.len())
        });
    });

    group.finish();
}

/// 4. Memory Footprint Audit Benchmark (<50MB target for 1M records)
fn bench_memory_footprint(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_footprint");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(2));

    let arena = get_arena_1m();

    group.bench_function("measure_arena_1m_memory", |b| {
        b.iter(|| {
            let mem = arena.memory_usage_bytes();
            let avg = arena.average_bytes_per_record();
            black_box((mem, avg))
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_crawl_throughput,
    bench_query_latency,
    bench_cold_startup,
    bench_memory_footprint
);
criterion_main!(benches);
