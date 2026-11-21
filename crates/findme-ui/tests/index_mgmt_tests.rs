//! Comprehensive integration test suite for Index Management, Directory Roots, and Custom Exclusions

use findme_ui::app::FindMeApp;
use findme_ui::index_mgmt::{IndexManagementState, IndexStats, IndexingState, RootStatus};
use findme_ui::message::{IndexManagementMessage, Message, ScreenId};
use std::path::{Path, PathBuf};

#[test]
fn test_default_index_state() {
    let state = IndexManagementState::default();
    assert_eq!(state.indexing_state, IndexingState::Idle);
    assert!(!state.indexing_state.is_scanning());

    // Stats
    assert!(state.stats.total_files > 0);
    assert!(state.stats.total_folders > 0);
    assert!(state.stats.memory_bytes > 0);
    assert!(state.stats.watcher_active);

    // Roots
    assert!(!state.roots.is_empty());
    assert_eq!(state.roots[0].status, RootStatus::Indexed);
    assert_eq!(state.roots[0].status.as_str(), "Indexed");

    // Exclusions
    assert!(state
        .exclusions
        .iter()
        .any(|e| e.pattern == "/proc" && e.is_builtin));
    assert!(state
        .exclusions
        .iter()
        .any(|e| e.pattern == "/sys" && e.is_builtin));
    assert!(state
        .exclusions
        .iter()
        .any(|e| e.pattern == "/dev" && e.is_builtin));
    assert!(state
        .exclusions
        .iter()
        .any(|e| e.pattern == "~/.cache" && !e.is_builtin));
    assert!(state
        .exclusions
        .iter()
        .any(|e| e.pattern == "node_modules" && !e.is_builtin));
}

#[test]
fn test_stats_formatting() {
    let stats = IndexStats {
        total_files: 1_234_567,
        total_folders: 89_012,
        memory_bytes: 52_428_800, // 50.0 MB
        last_scan_secs: Some(1_700_000_000),
        watcher_active: true,
        watcher_watch_count: 42,
    };

    assert_eq!(stats.formatted_files(), "1,234,567");
    assert_eq!(stats.formatted_folders(), "89,012");
    assert_eq!(stats.formatted_memory(), "50.0 MB");
    assert_eq!(stats.formatted_last_scan(), "Recent (Up to date)");
    assert_eq!(
        stats.watcher_status_label(),
        "● Active (inotify: 42 watches)"
    );

    // Test different memory units
    let bytes_stats = IndexStats {
        memory_bytes: 500,
        ..IndexStats::default()
    };
    assert_eq!(bytes_stats.formatted_memory(), "500 B");

    let kb_stats = IndexStats {
        memory_bytes: 204_800,
        ..IndexStats::default()
    };
    assert_eq!(kb_stats.formatted_memory(), "200.0 KB");

    let gb_stats = IndexStats {
        memory_bytes: 2_147_483_648,
        ..IndexStats::default()
    };
    assert_eq!(gb_stats.formatted_memory(), "2.0 GB");

    // Test inactive watcher
    let inactive_watcher = IndexStats {
        watcher_active: false,
        ..IndexStats::default()
    };
    assert_eq!(
        inactive_watcher.watcher_status_label(),
        "○ Inactive (Polling only)"
    );

    // Test never scanned
    let never_scanned = IndexStats {
        last_scan_secs: None,
        ..IndexStats::default()
    };
    assert_eq!(never_scanned.formatted_last_scan(), "Never");
}

#[test]
fn test_add_and_remove_roots() {
    let mut state = IndexManagementState::default();
    let initial_count = state.roots.len();

    // Add new root
    let new_path = PathBuf::from("/media/faizan/data");
    assert!(state.add_root(new_path.clone()));
    assert_eq!(state.roots.len(), initial_count + 1);

    // Duplicate path with trailing slash should be rejected
    let duplicate_path = PathBuf::from("/media/faizan/data/");
    assert!(!state.add_root(duplicate_path));
    assert_eq!(state.roots.len(), initial_count + 1);

    // Empty path should be rejected
    assert!(!state.add_root(PathBuf::from("")));
    assert!(!state.add_root(PathBuf::from("   ")));

    // Remove existing root
    assert!(state.remove_root(&new_path));
    assert_eq!(state.roots.len(), initial_count);

    // Removing non-existent root returns false
    assert!(!state.remove_root(Path::new("/non/existent/path")));
}

#[test]
fn test_add_and_remove_exclusions() {
    let mut state = IndexManagementState::default();
    let initial_count = state.exclusions.len();

    // Add custom exclusion
    assert!(state.add_exclusion("build_artifacts".to_string()));
    assert_eq!(state.exclusions.len(), initial_count + 1);

    // Duplicate (case-insensitive) should be rejected
    assert!(!state.add_exclusion("BUILD_ARTIFACTS".to_string()));
    assert_eq!(state.exclusions.len(), initial_count + 1);

    // Empty or whitespace pattern should be rejected
    assert!(!state.add_exclusion("".to_string()));
    assert!(!state.add_exclusion("   ".to_string()));

    // Remove custom exclusion
    assert!(state.remove_exclusion("build_artifacts"));
    assert_eq!(state.exclusions.len(), initial_count);

    // System built-in exclusions must not be removable
    assert!(!state.remove_exclusion("/proc"));
    assert!(!state.remove_exclusion("/sys"));
    assert!(!state.remove_exclusion("/dev"));
    assert!(state.exclusions.iter().any(|e| e.pattern == "/proc"));
}

#[test]
fn test_add_default_exclusions() {
    let mut state = IndexManagementState {
        stats: IndexStats::default(),
        roots: Vec::new(),
        exclusions: Vec::new(),
        indexing_state: IndexingState::Idle,
        new_root_input: String::new(),
        new_exclusion_input: String::new(),
    };

    assert!(state.exclusions.is_empty());
    state.add_default_exclusions();
    assert!(!state.exclusions.is_empty());

    let count_after_first = state.exclusions.len();
    // Adding again should not create duplicates
    state.add_default_exclusions();
    assert_eq!(state.exclusions.len(), count_after_first);
}

#[test]
fn test_indexing_state_enum() {
    let idle = IndexingState::Idle;
    assert!(!idle.is_scanning());

    let scanning = IndexingState::Scanning {
        current_path: "/home/user".to_string(),
        files_found: 10_000,
        percent: 45.5,
    };
    assert!(scanning.is_scanning());

    let error = IndexingState::Error("Permission denied".to_string());
    assert!(!error.is_scanning());
}

#[test]
fn test_app_index_management_messages() {
    let mut app = FindMeApp::new();
    let _ = app.update(Message::SwitchScreen(ScreenId::IndexManagement));
    assert_eq!(app.state.active_screen, ScreenId::IndexManagement);

    // 1. Add root from text input
    let _ = app.update(Message::IndexManagement(
        IndexManagementMessage::UpdateNewRootInput("/opt/custom_dir".to_string()),
    ));
    assert_eq!(app.state.index_mgmt.new_root_input, "/opt/custom_dir");

    let _ = app.update(Message::IndexManagement(
        IndexManagementMessage::AddRootFromInput,
    ));
    assert!(app
        .state
        .index_mgmt
        .roots
        .iter()
        .any(|r| r.path == Path::new("/opt/custom_dir")));
    assert!(app.state.index_mgmt.new_root_input.is_empty());
    assert_eq!(
        app.state.notification,
        Some("Added indexed root directory".to_string())
    );

    // 2. Remove root
    let _ = app.update(Message::IndexManagement(
        IndexManagementMessage::RemoveRoot(PathBuf::from("/opt/custom_dir")),
    ));
    assert!(!app
        .state
        .index_mgmt
        .roots
        .iter()
        .any(|r| r.path == Path::new("/opt/custom_dir")));

    // 3. Add exclusion from text input
    let _ = app.update(Message::IndexManagement(
        IndexManagementMessage::UpdateNewExclusionInput("*.bak".to_string()),
    ));
    assert_eq!(app.state.index_mgmt.new_exclusion_input, "*.bak");

    let _ = app.update(Message::IndexManagement(
        IndexManagementMessage::AddExclusionFromInput,
    ));
    assert!(app
        .state
        .index_mgmt
        .exclusions
        .iter()
        .any(|e| e.pattern == "*.bak"));
    assert!(app.state.index_mgmt.new_exclusion_input.is_empty());
    assert_eq!(
        app.state.notification,
        Some("Added exclusion rule".to_string())
    );

    // 4. Remove exclusion
    let _ = app.update(Message::IndexManagement(
        IndexManagementMessage::RemoveExclusion("*.bak".to_string()),
    ));
    assert!(!app
        .state
        .index_mgmt
        .exclusions
        .iter()
        .any(|e| e.pattern == "*.bak"));

    // 5. Add default exclusions
    let _ = app.update(Message::IndexManagement(
        IndexManagementMessage::AddDefaultExclusions,
    ));
    assert_eq!(
        app.state.notification,
        Some("Default exclusion rules added".to_string())
    );

    // 6. Trigger Rebuild Index
    let _ = app.update(Message::IndexManagement(
        IndexManagementMessage::TriggerRebuildIndex,
    ));
    assert!(app.state.index_mgmt.indexing_state.is_scanning());
    assert_eq!(
        app.state.notification,
        Some("Starting full index rebuild...".to_string())
    );

    // 7. Progress update
    let _ = app.update(Message::IndexManagement(
        IndexManagementMessage::UpdateProgress {
            current_path: "/home/user/docs".to_string(),
            files_found: 25_000,
            percent: 50.0,
        },
    ));
    match &app.state.index_mgmt.indexing_state {
        IndexingState::Scanning {
            current_path,
            files_found,
            percent,
        } => {
            assert_eq!(current_path, "/home/user/docs");
            assert_eq!(*files_found, 25_000);
            assert_eq!(*percent, 50.0);
        }
        _ => panic!("Expected Scanning state"),
    }

    // 8. Rebuild Finished
    let new_stats = IndexStats {
        total_files: 200_000,
        total_folders: 25_000,
        memory_bytes: 20_000_000,
        last_scan_secs: Some(1_700_000_100),
        watcher_active: true,
        watcher_watch_count: 30,
    };
    let _ = app.update(Message::IndexManagement(
        IndexManagementMessage::RebuildFinished(new_stats),
    ));
    assert_eq!(app.state.index_mgmt.indexing_state, IndexingState::Idle);
    assert_eq!(app.state.index_mgmt.stats.total_files, 200_000);
    assert_eq!(app.state.indexed_files_count, 200_000);
    assert_eq!(
        app.state.notification,
        Some("Index rebuild completed successfully".to_string())
    );

    // 9. Trigger Refresh
    let _ = app.update(Message::IndexManagement(
        IndexManagementMessage::TriggerRefreshIndex,
    ));
    assert_eq!(
        app.state.notification,
        Some("Incremental refresh completed (inotify synced)".to_string())
    );

    // 10. Clear Index
    let _ = app.update(Message::IndexManagement(IndexManagementMessage::ClearIndex));
    assert_eq!(app.state.index_mgmt.stats.total_files, 0);
    assert_eq!(app.state.index_mgmt.stats.total_folders, 0);
    assert_eq!(app.state.index_mgmt.stats.memory_bytes, 0);
    assert_eq!(app.state.indexed_files_count, 0);
    assert_eq!(app.state.indexed_folders_count, 0);
    assert_eq!(
        app.state.notification,
        Some("Index database cleared".to_string())
    );
}

#[test]
fn test_index_mgmt_view_rendering() {
    let mut app = FindMeApp::new();

    // 1. Render in Idle state
    {
        let _view_idle = findme_ui::views::index_mgmt::view(&app.state);
    }

    // 2. Render in Scanning state
    let _ = app.update(Message::IndexManagement(
        IndexManagementMessage::TriggerRebuildIndex,
    ));
    let _ = app.update(Message::IndexManagement(
        IndexManagementMessage::UpdateProgress {
            current_path: "/var/log/syslog".to_string(),
            files_found: 12_345,
            percent: 32.5,
        },
    ));
    {
        let _view_scanning = findme_ui::views::index_mgmt::view(&app.state);
    }
}
