//! Property-based fuzz tests and performance micro-benchmarks for query parsing.
//! Verifies zero panics under arbitrary UTF-8 inputs and $<50µs parsing latency.

use findme_core::query::QueryParser;
use proptest::prelude::*;
use std::time::Instant;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn prop_parser_never_panics_on_arbitrary_string(s in ".*") {
        let _ast = QueryParser::parse(&s);
    }

    #[test]
    fn prop_parser_never_panics_on_syntax_heavy_string(s in "[a-zA-Z0-9_|!()\":><=\\.\\- \t]{0,100}") {
        let _ast = QueryParser::parse(&s);
    }

    #[test]
    fn prop_parser_never_panics_with_random_now(
        s in ".*",
        now in 0u64..2_000_000_000,
    ) {
        let _ast = QueryParser::parse_with_now(&s, now);
    }
}

#[test]
fn test_query_parsing_latency_budget() {
    // The requirement states that query parsing latency must be <50µs per query.
    // We parse a complex realistic query 10,000 times and calculate the average latency.
    let complex_query =
        "ext:rs;toml size:>1mb !path:/target (dm:today | dm:yesterday) \"main loop\"";

    // Warm up
    for _ in 0..100 {
        let _ = QueryParser::parse(complex_query);
    }

    let iterations = 10_000;
    let start = Instant::now();

    for _ in 0..iterations {
        let ast = QueryParser::parse(complex_query);
        assert!(!ast.is_empty());
    }

    let elapsed = start.elapsed();
    let per_query_micros = elapsed.as_micros() as f64 / iterations as f64;

    println!(
        "Benchmark: 10,000 parses took {:?}, average = {:.3} µs/query",
        elapsed, per_query_micros
    );

    // Verify budget: must be well below 50 µs
    assert!(
        per_query_micros < 50.0,
        "Query parse latency {:.3} µs exceeded 50.0 µs budget",
        per_query_micros
    );
}
