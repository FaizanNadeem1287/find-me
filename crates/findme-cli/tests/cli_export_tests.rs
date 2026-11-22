//! Integration test suite for CLI search result export functionality
//!
//! Validates:
//! 1. Argument parsing of `--export-csv`, `--export-txt`, `--export-efu`, `--export-json`.
//! 2. Headless CLI query execution combined with exports.
//! 3. File content integrity across formats.
//! 4. Truncation with `-n` / `--max-results`.

use findme_cli::{execute_cli_search, parse_cli_args, search_arena_items};
use findme_core::SearchResultItem;
use std::fs::read_to_string;
use tempfile::tempdir;

#[test]
fn test_parse_cli_export_flags() {
    let args: Vec<String> = vec![
        "find-me".into(),
        "-q".into(),
        "ext:rs".into(),
        "--export-csv".into(),
        "/tmp/results.csv".into(),
        "--export-txt".into(),
        "/tmp/results.txt".into(),
        "--export-efu".into(),
        "/tmp/results.efu".into(),
        "--export-json".into(),
        "/tmp/results.json".into(),
        "-n".into(),
        "50".into(),
    ];

    let opts = parse_cli_args(&args);
    assert_eq!(opts.query.as_deref(), Some("ext:rs"));
    assert_eq!(opts.export_csv.as_deref(), Some("/tmp/results.csv"));
    assert_eq!(opts.export_txt.as_deref(), Some("/tmp/results.txt"));
    assert_eq!(opts.export_efu.as_deref(), Some("/tmp/results.efu"));
    assert_eq!(opts.export_json.as_deref(), Some("/tmp/results.json"));
    assert_eq!(opts.max_results, 50);
    assert!(opts.has_export());
    assert!(opts.is_cli_mode);
}

#[test]
fn test_cli_export_csv_and_txt_e2e() {
    let dir = tempdir().unwrap();
    let csv_path = dir.path().join("output.csv");
    let txt_path = dir.path().join("output.txt");

    let args: Vec<String> = vec![
        "find-me".into(),
        "-q".into(),
        "ext:rs".into(),
        "--export-csv".into(),
        csv_path.to_string_lossy().to_string(),
        "--export-txt".into(),
        txt_path.to_string_lossy().to_string(),
        "-n".into(),
        "10".into(),
    ];

    let opts = parse_cli_args(&args);
    let cache_path = dir.path().join("non_existent_cache.bin");

    // Execute search with fallback to sample arena
    execute_cli_search(&opts, &cache_path).unwrap();

    // Verify CSV file exists and contains matches
    assert!(csv_path.exists());
    let csv_content = read_to_string(&csv_path).unwrap();
    assert!(csv_content.starts_with("\"Name\",\"Path\",\"Size\",\"Date Modified\",\"Type\"\n"));
    assert!(csv_content.contains(".rs"));

    // Verify TXT file exists and has one path per line
    assert!(txt_path.exists());
    let txt_content = read_to_string(&txt_path).unwrap();
    let lines: Vec<&str> = txt_content.lines().collect();
    assert!(!lines.is_empty());
    for line in &lines {
        assert!(line.contains(".rs"));
    }
}

#[test]
fn test_cli_export_efu_and_json_e2e() {
    let dir = tempdir().unwrap();
    let efu_path = dir.path().join("output.efu");
    let json_path = dir.path().join("output.json");

    let args: Vec<String> = vec![
        "find-me".into(),
        "-q".into(),
        "Cargo".into(),
        "--export-efu".into(),
        efu_path.to_string_lossy().to_string(),
        "--export-json".into(),
        json_path.to_string_lossy().to_string(),
    ];

    let opts = parse_cli_args(&args);
    let cache_path = dir.path().join("non_existent_cache.bin");

    execute_cli_search(&opts, &cache_path).unwrap();

    // Verify EFU
    assert!(efu_path.exists());
    let efu_content = read_to_string(&efu_path).unwrap();
    assert!(efu_content.starts_with("Filename,Size,Date Modified,Date Created,Attributes\n"));
    assert!(efu_content.contains("Cargo"));

    // Verify JSON
    assert!(json_path.exists());
    let json_content = read_to_string(&json_path).unwrap();
    let parsed: Vec<SearchResultItem> = serde_json::from_str(&json_content).unwrap();
    assert!(!parsed.is_empty());
    assert!(parsed.iter().any(|item| item.name.contains("Cargo")));
}

#[test]
fn test_cli_export_with_max_results() {
    let dir = tempdir().unwrap();
    let txt_path = dir.path().join("capped.txt");

    let args: Vec<String> = vec![
        "find-me".into(),
        "-q".into(),
        "".into(), // Match all
        "--export-txt".into(),
        txt_path.to_string_lossy().to_string(),
        "-n".into(),
        "3".into(),
    ];

    let opts = parse_cli_args(&args);
    let cache_path = dir.path().join("dummy.bin");

    execute_cli_search(&opts, &cache_path).unwrap();

    assert!(txt_path.exists());
    let txt_content = read_to_string(&txt_path).unwrap();
    let lines: Vec<&str> = txt_content.lines().collect();
    assert_eq!(lines.len(), 3);
}

#[test]
fn test_search_arena_items_helper() {
    let sample_arena = findme_ui::engine::EngineBridge::default_sample_arena();
    let opts = parse_cli_args(&["find-me".into(), "-q".into(), "Cargo.toml".into()]);

    let items = search_arena_items(&opts, &sample_arena);
    assert!(!items.is_empty());
    assert_eq!(items[0].name, "Cargo.toml");
    assert!(items[0].path.contains("Cargo.toml"));
    assert!(!items[0].is_dir);
}
