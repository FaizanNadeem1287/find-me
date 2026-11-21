# Performance Benchmarking & Regression Detection Guide

This guide details how to benchmark Find-Me's throughput, latency, and memory footprint, and how to verify that performance budgets are strictly enforced.

---

## 1. Performance Budgets

Find-Me enforces non-negotiable performance targets:

Metric | Target Budget | Verification Tool
:--- | :--- | :---
**SSD Crawl Throughput** | $> 200,000$ files / second | Criterion micro-benchmark (`benches/crawler.rs`)
**Query Latency (1M files)** | $< 5.0$ milliseconds | Criterion micro-benchmark (`benches/search.rs`)
**Memory Footprint (1M files)** | $< 45$ Megabytes total | Allocator tracking benchmark (`benches/memory.rs`)
**Cold Index Load Latency** | $< 50$ milliseconds | Startup persistence benchmark (`benches/persistence.rs`)
**UI Framerate Under Load** | Steady $60$ FPS | Frame-timing assertion during fast scroll

---

## 2. Benchmarking with `criterion`

We use the industry-standard `criterion` crate for statistical benchmarking with automatic outlier detection and regression reporting:

```toml
# In Cargo.toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "search_benchmark"
harness = false
```

### Example Benchmark: Query Matching Latency
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use findme_core::{IndexArena, QueryEngine};

fn bench_query_search(c: &mut Criterion) {
    // Generate synthetic index with 1,000,000 file records
    let arena = IndexArena::generate_synthetic(1_000_000);
    let engine = QueryEngine::new(&arena);

    c.bench_function("search_exact_substring_1m", |b| {
        b.iter(|| {
            let results = engine.search(black_box("config.json"));
            black_box(results);
        });
    });

    c.bench_function("search_fuzzy_nucleo_1m", |b| {
        b.iter(|| {
            let results = engine.search_fuzzy(black_box("cfg_jsn"));
            black_box(results);
        });
    });

    c.bench_function("search_filter_size_and_ext_1m", |b| {
        b.iter(|| {
            let results = engine.search(black_box("ext:rs size:>100kb"));
            black_box(results);
        });
    });
}

criterion_group!(benches, bench_query_search);
criterion_main!(benches);
```

### Running Benchmarks
```bash
# Verify benchmarks compile without running full iterations
cargo bench --no-run

# Run benchmarks and generate HTML comparison report
cargo bench
```

---

## 3. Memory Profiling & Leak Detection

To ensure memory does not bloat or leak across repeated queries or index rescans:
1. **`stats_alloc`**: Tracks allocated bytes during indexing tests:
   ```rust
   let initial_bytes = ALLOCATOR.stats().bytes_allocated;
   arena.index_directory(&path);
   let allocated = ALLOCATOR.stats().bytes_allocated - initial_bytes;
   assert!(allocated < 45 * 1024 * 1024, "Memory exceeded 45MB budget!");
   ```
2. **Valgrind / Heaptrack**:
   Run during release candidate validation:
   ```bash
   heaptrack target/release/find-me
   ```
