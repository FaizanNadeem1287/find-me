//! Integration and unit tests for Headless CLI Search Query Mode & GUI Pre-Fill (Task 11.1)

use findme_cli::{parse_cli_args, search_arena, CliOptions, CliSearchResult};
use findme_core::{FileArena, FileFlags};
use findme_ui::FindMeApp;
use std::path::{Path, PathBuf};
use std::process::Command;

fn find_repo_root() -> PathBuf {
    let mut current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    while !current.join("Cargo.toml").exists() || !current.join("crates").exists() {
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }
    current
}

#[test]
fn test_parse_cli_arguments() {
    // 1. Positional query
    let args1 = vec!["find-me".to_string(), "ext:rs main".to_string()];
    let opts1 = parse_cli_args(&args1);
    assert_eq!(opts1.query, Some("ext:rs main".to_string()));
    assert!(!opts1.is_json);
    assert_eq!(opts1.max_results, 100);

    // 2. Explicit -q flag with -n limit and --json
    let args2 = vec![
        "find-me".to_string(),
        "-q".to_string(),
        "size:>10mb".to_string(),
        "-n".to_string(),
        "42".to_string(),
        "--json".to_string(),
    ];
    let opts2 = parse_cli_args(&args2);
    assert_eq!(opts2.query, Some("size:>10mb".to_string()));
    assert_eq!(opts2.max_results, 42);
    assert!(opts2.is_json);
    assert!(opts2.is_cli_mode);

    // 3. Flags: --stats, --cli, --rebuild-index
    let args3 = vec!["find-me".to_string(), "--stats".to_string()];
    let opts3 = parse_cli_args(&args3);
    assert!(opts3.show_stats);
    assert!(opts3.is_cli_mode);

    let args4 = vec!["find-me".to_string(), "--rebuild-index".to_string()];
    let opts4 = parse_cli_args(&args4);
    assert!(opts4.rebuild_index);
}

#[test]
fn test_cli_search_against_arena() {
    let mut arena = FileArena::new();
    arena
        .add_file(Path::new("/home/user/main.rs"), FileFlags::NONE, 1024, 1000)
        .unwrap();
    arena
        .add_file(Path::new("/home/user/lib.rs"), FileFlags::NONE, 2048, 2000)
        .unwrap();
    arena
        .add_file(
            Path::new("/home/user/photo.png"),
            FileFlags::NONE,
            50000,
            3000,
        )
        .unwrap();
    arena
        .add_file(
            Path::new("/home/user/archive.zip"),
            FileFlags::NONE,
            80000,
            4000,
        )
        .unwrap();

    // 1. Search for Rust files (ext:rs)
    let opts_rs = CliOptions {
        query: Some("ext:rs".to_string()),
        max_results: 10,
        ..Default::default()
    };
    let results_rs = search_arena(&opts_rs, &arena);
    assert_eq!(results_rs.len(), 2);
    let names: Vec<&str> = results_rs.iter().map(|r| r.name.as_str()).collect();
    assert!(names.contains(&"main.rs"));
    assert!(names.contains(&"lib.rs"));

    // 2. Search for exact photo
    let opts_photo = CliOptions {
        query: Some("photo".to_string()),
        max_results: 10,
        ..Default::default()
    };
    let results_photo = search_arena(&opts_photo, &arena);
    assert_eq!(results_photo.len(), 1);
    assert_eq!(results_photo[0].name, "photo.png");
    assert_eq!(results_photo[0].path, "/home/user/photo.png");
    assert_eq!(results_photo[0].size_bytes, 50000);
}

#[test]
fn test_cli_search_json_serialization() {
    let mut arena = FileArena::new();
    arena
        .add_file(
            Path::new("/var/log/syslog"),
            FileFlags::NONE,
            4096,
            12345678,
        )
        .unwrap();

    let opts = CliOptions {
        query: Some("syslog".to_string()),
        is_json: true,
        max_results: 10,
        ..Default::default()
    };

    let results = search_arena(&opts, &arena);
    assert_eq!(results.len(), 1);

    // Serialize to JSON and verify structure
    let json_str =
        serde_json::to_string_pretty(&results).expect("Failed to serialize results to JSON");
    assert!(json_str.contains("\"path\": \"/var/log/syslog\""));
    assert!(json_str.contains("\"name\": \"syslog\""));
    assert!(json_str.contains("\"size_bytes\": 4096"));
    assert!(json_str.contains("\"mtime_secs\": 12345678"));
    assert!(json_str.contains("\"is_dir\": false"));

    // Deserialize back into CliSearchResult
    let deserialized: Vec<CliSearchResult> =
        serde_json::from_str(&json_str).expect("Failed to deserialize JSON");
    assert_eq!(deserialized, results);
}

#[test]
fn test_cli_max_results_truncation() {
    let mut arena = FileArena::new();
    for i in 0..50 {
        let path_str = format!("/tmp/file_{:02}.tmp", i);
        arena
            .add_file(Path::new(&path_str), FileFlags::NONE, 100, 100)
            .unwrap();
    }

    let opts = CliOptions {
        query: Some("ext:tmp".to_string()),
        max_results: 5,
        ..Default::default()
    };

    let results = search_arena(&opts, &arena);
    assert_eq!(
        results.len(),
        5,
        "Results must be capped at max_results limit"
    );
}

#[test]
fn test_gui_boot_with_initial_query() {
    // 1. Boot with initial query
    let (app_with_query, _task) = FindMeApp::boot_with_query(Some("ext:rs findme".to_string()));
    assert_eq!(app_with_query.state.query, "ext:rs findme");
    assert!(app_with_query.state.is_searching);

    // 2. Boot with empty query
    let (app_empty, _task_empty) = FindMeApp::boot_with_query(None);
    assert!(app_empty.state.query.is_empty());
    assert!(!app_empty.state.is_searching);

    // 3. Boot with whitespace-only query
    let (app_ws, _task_ws) = FindMeApp::boot_with_query(Some("   ".to_string()));
    assert!(app_ws.state.query.is_empty());
    assert!(!app_ws.state.is_searching);
}

#[test]
fn test_cli_binary_execution_e2e() {
    let root = find_repo_root();
    let debug_bin = root.join("target/debug/find-me");
    let release_bin = root.join("target/release/find-me");

    let bin_path = if debug_bin.exists() {
        debug_bin
    } else if release_bin.exists() {
        release_bin
    } else {
        let status = Command::new("cargo")
            .args(["build", "-p", "findme-cli"])
            .status()
            .expect("Failed to build binary");
        assert!(status.success());
        root.join("target/debug/find-me")
    };

    // 1. Run --search in CLI mode
    let search_out = Command::new(&bin_path)
        .args(["--search", "main.rs"])
        .output()
        .expect("Failed to run find-me --search");
    assert!(search_out.status.success());
    let stdout_str = String::from_utf8_lossy(&search_out.stdout);
    assert!(
        stdout_str.contains("main.rs"),
        "CLI search output must contain matched file"
    );

    // 2. Run --json output mode
    let json_out = Command::new(&bin_path)
        .args(["--json", "main.rs"])
        .output()
        .expect("Failed to run find-me --json");
    assert!(json_out.status.success());
    let json_text = String::from_utf8_lossy(&json_out.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&json_text).expect("find-me --json must output valid JSON");
    assert!(parsed.is_array());
    let arr = parsed.as_array().unwrap();
    assert!(!arr.is_empty());
    assert!(arr[0].get("path").is_some());
    assert!(arr[0].get("name").is_some());

    // 3. Run --stats
    let stats_out = Command::new(&bin_path)
        .arg("--stats")
        .output()
        .expect("Failed to run find-me --stats");
    assert!(stats_out.status.success());
    let stats_text = String::from_utf8_lossy(&stats_out.stdout);
    assert!(stats_text.contains("Find-Me Index Statistics:"));
    assert!(stats_text.contains("Indexed Files:"));
    assert!(stats_text.contains("Arena Memory Usage:"));
}
