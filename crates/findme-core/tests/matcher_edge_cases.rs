//! Comprehensive edge-case test suite for SearchEngine, nucleo-matcher integration,
//! score ordering, and multi-column sorting.

use findme_core::matcher::{SearchConfig, SearchEngine, SortColumn, SortDirection};
use findme_core::query::QueryParser;
use findme_core::record::FileFlags;
use findme_core::FileArena;
use std::path::Path;

fn populate_test_arena() -> FileArena {
    let mut arena = FileArena::new();

    // Add diverse files
    // 0: exact stem match for "main"
    arena
        .add_file(Path::new("/src/main.rs"), FileFlags::NONE, 1024, 1795260000)
        .unwrap();

    // 1: prefix match for "main"
    arena
        .add_file(
            Path::new("/src/main_window.rs"),
            FileFlags::NONE,
            2048,
            1795261000,
        )
        .unwrap();

    // 2: substring match for "main"
    arena
        .add_file(
            Path::new("/src/domain.rs"),
            FileFlags::NONE,
            4096,
            1795262000,
        )
        .unwrap();

    // 3: fuzzy match for "main" (m...a...i...n)
    arena
        .add_file(
            Path::new("/src/my_app_internal_node.rs"),
            FileFlags::NONE,
            8192,
            1795263000,
        )
        .unwrap();

    // 4: completely unrelated file
    arena
        .add_file(
            Path::new("/docs/readme.txt"),
            FileFlags::NONE,
            500,
            1795264000,
        )
        .unwrap();

    // 5: directory
    arena
        .add_file(Path::new("/src/utils"), FileFlags::IS_DIR, 0, 1795265000)
        .unwrap();

    // 6: hidden file
    arena
        .add_file(
            Path::new("/home/user/.config.toml"),
            FileFlags::IS_HIDDEN,
            256,
            1795266000,
        )
        .unwrap();

    // 7: large file
    arena
        .add_file(
            Path::new("/data/archive.iso"),
            FileFlags::NONE,
            1024 * 1024 * 50,
            1795267000,
        )
        .unwrap();

    // 8: unicode file
    arena
        .add_file(
            Path::new("/home/user/🦀_findme.rs"),
            FileFlags::NONE,
            2000,
            1795268000,
        )
        .unwrap();

    arena
}

#[test]
fn test_strict_score_ordering_hierarchy() {
    let arena = populate_test_arena();
    let ast = QueryParser::parse("main");
    let config = SearchConfig {
        max_results: 10,
        sort_column: SortColumn::Score,
        sort_direction: SortDirection::Descending,
    };

    let results = SearchEngine::search(&arena, &ast, &config);

    // Exact stem match ("main.rs") must score higher than prefix match ("main_window.rs")
    // Prefix match ("main_window.rs") must score higher than substring match ("domain.rs")
    // Substring match ("domain.rs") must score higher than fuzzy match ("my_app_internal_node.rs")

    let idx_exact = 0; // main.rs
    let idx_prefix = 1; // main_window.rs
    let idx_substring = 2; // domain.rs
    let idx_fuzzy = 3; // my_app_internal_node.rs

    let score_exact = results
        .iter()
        .find(|r| r.record_idx == idx_exact)
        .unwrap()
        .score;
    let score_prefix = results
        .iter()
        .find(|r| r.record_idx == idx_prefix)
        .unwrap()
        .score;
    let score_substring = results
        .iter()
        .find(|r| r.record_idx == idx_substring)
        .unwrap()
        .score;
    let score_fuzzy = results
        .iter()
        .find(|r| r.record_idx == idx_fuzzy)
        .unwrap()
        .score;

    assert!(
        score_exact > score_prefix,
        "Exact score {} must be > prefix score {}",
        score_exact,
        score_prefix
    );
    assert!(
        score_prefix > score_substring,
        "Prefix score {} must be > substring score {}",
        score_prefix,
        score_substring
    );
    assert!(
        score_substring > score_fuzzy,
        "Substring score {} must be > fuzzy score {}",
        score_substring,
        score_fuzzy
    );
}

#[test]
fn test_quoted_search_disables_fuzzy() {
    let arena = populate_test_arena();

    // Unquoted "main" matches fuzzy "my_app_internal_node.rs"
    let ast_fuzzy = QueryParser::parse("main");
    let res_fuzzy = SearchEngine::search(&arena, &ast_fuzzy, &SearchConfig::default());
    assert!(res_fuzzy.iter().any(|r| r.record_idx == 3));

    // Quoted "\"main\"" must NOT match fuzzy "my_app_internal_node.rs"
    let ast_quoted = QueryParser::parse("\"main\"");
    let res_quoted = SearchEngine::search(&arena, &ast_quoted, &SearchConfig::default());
    assert!(!res_quoted.iter().any(|r| r.record_idx == 3));
    // But still matches exact stem / prefix / substring
    assert!(res_quoted.iter().any(|r| r.record_idx == 0));
    assert!(res_quoted.iter().any(|r| r.record_idx == 1));
    assert!(res_quoted.iter().any(|r| r.record_idx == 2));
}

#[test]
fn test_case_insensitive_matching() {
    let arena = populate_test_arena();
    let ast = QueryParser::parse("MAIN");
    let results = SearchEngine::search(&arena, &ast, &SearchConfig::default());

    assert!(results.iter().any(|r| r.record_idx == 0)); // main.rs
    assert!(results.iter().any(|r| r.record_idx == 1)); // main_window.rs
}

#[test]
fn test_extension_filter_and_combination() {
    let arena = populate_test_arena();
    let ast = QueryParser::parse("ext:rs main");
    let results = SearchEngine::search(&arena, &ast, &SearchConfig::default());

    for r in &results {
        let name = arena.get_name(r.record_idx).unwrap();
        assert!(name.ends_with(".rs"));
    }
}

#[test]
fn test_size_filter() {
    let arena = populate_test_arena();
    let ast = QueryParser::parse("size:>10mb");
    let results = SearchEngine::search(&arena, &ast, &SearchConfig::default());

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].record_idx, 7); // archive.iso (50MB)
}

#[test]
fn test_attribute_filter() {
    let arena = populate_test_arena();

    // Match only directories
    let ast_dir = QueryParser::parse("attrib:d");
    let results_dir = SearchEngine::search(&arena, &ast_dir, &SearchConfig::default());
    assert_eq!(results_dir.len(), 1);
    assert_eq!(results_dir[0].record_idx, 5); // /src/utils

    // Match only hidden
    let ast_hidden = QueryParser::parse("attrib:h");
    let results_hidden = SearchEngine::search(&arena, &ast_hidden, &SearchConfig::default());
    assert_eq!(results_hidden.len(), 1);
    assert_eq!(results_hidden[0].record_idx, 6); // .config.toml
}

#[test]
fn test_negation_filter() {
    let arena = populate_test_arena();
    let ast = QueryParser::parse("ext:rs !main");
    let results = SearchEngine::search(&arena, &ast, &SearchConfig::default());

    // Should match domain.rs (2) and 🦀_findme.rs (8), but NOT main.rs (0) or main_window.rs (1)
    assert!(!results.iter().any(|r| r.record_idx == 0));
    assert!(!results.iter().any(|r| r.record_idx == 1));
    assert!(results.iter().any(|r| r.record_idx == 8));
}

#[test]
fn test_path_prefix_filter() {
    let arena = populate_test_arena();
    let ast = QueryParser::parse("path:/home/user");
    let results = SearchEngine::search(&arena, &ast, &SearchConfig::default());

    for r in &results {
        let path = arena.reconstruct_path_by_index(r.record_idx).unwrap();
        assert!(path.starts_with("/home/user"));
    }
}

#[test]
fn test_empty_query_matches_all() {
    let arena = populate_test_arena();
    let ast = QueryParser::parse("");
    let results = SearchEngine::search(&arena, &ast, &SearchConfig::default());

    assert_eq!(results.len(), arena.len());
}

#[test]
fn test_result_truncation_max_results() {
    let arena = populate_test_arena();
    let ast = QueryParser::parse("");
    let config = SearchConfig {
        max_results: 3,
        sort_column: SortColumn::Score,
        sort_direction: SortDirection::Descending,
    };
    let results = SearchEngine::search(&arena, &ast, &config);

    assert_eq!(results.len(), 3);
}

#[test]
fn test_multi_column_sorting() {
    let arena = populate_test_arena();
    let ast = QueryParser::parse("");

    // Sort by Name Ascending
    let cfg_name_asc = SearchConfig {
        max_results: 100,
        sort_column: SortColumn::Name,
        sort_direction: SortDirection::Ascending,
    };
    let res_name_asc = SearchEngine::search(&arena, &ast, &cfg_name_asc);
    for window in res_name_asc.windows(2) {
        let name_a = arena.get_name(window[0].record_idx).unwrap();
        let name_b = arena.get_name(window[1].record_idx).unwrap();
        assert!(name_a <= name_b);
    }

    // Sort by Size Descending
    let cfg_size_desc = SearchConfig {
        max_results: 100,
        sort_column: SortColumn::Size,
        sort_direction: SortDirection::Descending,
    };
    let res_size_desc = SearchEngine::search(&arena, &ast, &cfg_size_desc);
    for window in res_size_desc.windows(2) {
        let size_a = arena.get(window[0].record_idx).unwrap().size_bytes;
        let size_b = arena.get(window[1].record_idx).unwrap().size_bytes;
        assert!(size_a >= size_b);
    }

    // Sort by Modified Ascending
    let cfg_mtime_asc = SearchConfig {
        max_results: 100,
        sort_column: SortColumn::Modified,
        sort_direction: SortDirection::Ascending,
    };
    let res_mtime_asc = SearchEngine::search(&arena, &ast, &cfg_mtime_asc);
    for window in res_mtime_asc.windows(2) {
        let mtime_a = arena.get(window[0].record_idx).unwrap().mtime_secs;
        let mtime_b = arena.get(window[1].record_idx).unwrap().mtime_secs;
        assert!(mtime_a <= mtime_b);
    }
}

#[test]
fn test_materialize_search_results() {
    let arena = populate_test_arena();
    let ast = QueryParser::parse("main.rs");
    let items = SearchEngine::search_and_materialize(&arena, &ast, &SearchConfig::default());

    assert!(!items.is_empty());
    assert_eq!(items[0].name, "main.rs");
    assert_eq!(items[0].path, "/src/main.rs");
    assert_eq!(items[0].size_bytes, 1024);
    assert_eq!(items[0].formatted_size(), "1.0 KB");
    assert!(!items[0].is_dir);
}
