//! Integration test suite for multi-format search result exporting
//!
//! Validates:
//! 1. RFC 4180 CSV export with proper quotation escaping for commas, quotes, and newlines.
//! 2. Plain text (TXT) line-by-line path output.
//! 3. Everything File List (EFU) compatibility with Windows FILETIME math and attribute bitmasks.
//! 4. Structured JSON serialization and roundtrip parsing.
//! 5. Automatic parent directory creation in `export_to_file`.

use findme_core::{
    export_to_file, export_to_writer, unix_secs_to_windows_filetime, windows_filetime_to_unix_secs,
    ExportFormat, SearchResultItem,
};
use std::fs::read_to_string;
use tempfile::tempdir;

fn sample_results() -> Vec<SearchResultItem> {
    vec![
        SearchResultItem {
            id: 1,
            name: "simple.txt".to_string(),
            path: "/home/user/docs/simple.txt".to_string(),
            size_bytes: 2048,
            mtime_secs: 1732000000,
            is_dir: false,
        },
        SearchResultItem {
            id: 2,
            name: "report, with commas and \"quotes\".csv".to_string(),
            path: "/home/user/docs/report, with commas and \"quotes\".csv".to_string(),
            size_bytes: 40960,
            mtime_secs: 1732000500,
            is_dir: false,
        },
        SearchResultItem {
            id: 3,
            name: "project_folder".to_string(),
            path: "/home/user/projects/project_folder".to_string(),
            size_bytes: 0,
            mtime_secs: 1732001000,
            is_dir: true,
        },
        SearchResultItem {
            id: 4,
            name: "multiline\nname.log".to_string(),
            path: "/home/user/logs/multiline\nname.log".to_string(),
            size_bytes: 512,
            mtime_secs: 1732002000,
            is_dir: false,
        },
    ]
}

#[test]
fn test_export_txt_lines() {
    let items = sample_results();
    let mut buf = Vec::new();
    let count = export_to_writer(ExportFormat::Txt, &items, &mut buf).unwrap();

    assert_eq!(count, 4);
    let output = String::from_utf8(buf).unwrap();
    let lines: Vec<&str> = output.lines().collect();

    assert_eq!(lines.len(), 5); // 4 items, but item 4 has a newline in its path
    assert_eq!(lines[0], "/home/user/docs/simple.txt");
    assert_eq!(lines[2], "/home/user/projects/project_folder");
}

#[test]
fn test_export_csv_formatting_and_escaping() {
    let items = sample_results();
    let mut buf = Vec::new();
    let count = export_to_writer(ExportFormat::Csv, &items, &mut buf).unwrap();

    assert_eq!(count, 4);
    let output = String::from_utf8(buf).unwrap();

    // Verify CSV Header
    assert!(output.starts_with("\"Name\",\"Path\",\"Size\",\"Date Modified\",\"Type\"\n"));

    // Verify simple item
    assert!(output.contains("simple.txt,/home/user/docs/simple.txt,2048,"));

    // Verify quotes and commas escaped
    assert!(output.contains(
        "\"report, with commas and \"\"quotes\"\".csv\",\"/home/user/docs/report, with commas and \"\"quotes\"\".csv\",40960,"
    ));

    // Verify multiline name quoted
    assert!(output.contains("\"multiline\nname.log\",\"/home/user/logs/multiline\nname.log\",512,"));

    // Verify folder kind
    assert!(output.contains("project_folder,/home/user/projects/project_folder,0,"));
    assert!(output.contains(",Folder\n"));
}

#[test]
fn test_export_efu_format_and_timestamps() {
    let items = sample_results();
    let mut buf = Vec::new();
    let count = export_to_writer(ExportFormat::Efu, &items, &mut buf).unwrap();

    assert_eq!(count, 4);
    let output = String::from_utf8(buf).unwrap();

    // Verify standard EFU header
    assert!(output.starts_with("Filename,Size,Date Modified,Date Created,Attributes\n"));

    // Verify filetimes
    let filetime_item1 = unix_secs_to_windows_filetime(1732000000);
    assert_eq!(windows_filetime_to_unix_secs(filetime_item1), 1732000000);

    let expected_item1 = format!(
        "\"/home/user/docs/simple.txt\",2048,{},{},32\n",
        filetime_item1, filetime_item1
    );
    assert!(output.contains(&expected_item1));

    // Verify directory has size 0 and attributes 16
    let filetime_dir = unix_secs_to_windows_filetime(1732001000);
    let expected_dir = format!(
        "\"/home/user/projects/project_folder\",0,{},{},16\n",
        filetime_dir, filetime_dir
    );
    assert!(output.contains(&expected_dir));
}

#[test]
fn test_export_json_format() {
    let items = sample_results();
    let mut buf = Vec::new();
    let count = export_to_writer(ExportFormat::Json, &items, &mut buf).unwrap();

    assert_eq!(count, 4);
    let parsed: Vec<SearchResultItem> = serde_json::from_slice(&buf).unwrap();
    assert_eq!(parsed.len(), 4);
    assert_eq!(parsed[0].name, "simple.txt");
    assert!(parsed[2].is_dir);
    assert_eq!(parsed[2].name, "project_folder");
}

#[test]
fn test_export_to_file_creates_parent_directories() {
    let items = sample_results();
    let dir = tempdir().unwrap();
    let target_path = dir
        .path()
        .join("nested")
        .join("subfolder")
        .join("results.csv");

    let count = export_to_file(ExportFormat::Csv, &items, &target_path).unwrap();
    assert_eq!(count, 4);
    assert!(target_path.exists());

    let content = read_to_string(&target_path).unwrap();
    assert!(content.contains("\"Name\",\"Path\",\"Size\",\"Date Modified\",\"Type\""));
    assert!(content.contains("simple.txt"));
}
