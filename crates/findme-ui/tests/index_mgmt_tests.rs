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
        excluded_folders: Vec::new(),
        excluded_extensions: Vec::new(),
        excluded_paths: Vec::new(),
        indexing_state: IndexingState::Idle,
        new_root_input: String::new(),
        new_exclusion_input: String::new(),
        new_extension_input: String::new(),
        new_folder_input: String::new(),
        view_gitignore_raw: false,
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

#[test]
fn test_smart_rule_routing_and_categorization() {
    let mut state = IndexManagementState::default();

    // 1. Extensions
    assert!(state.add_smart_rule("*.xyz"));
    assert!(state.excluded_extensions.contains(&"xyz".to_string()));
    assert!(state.add_smart_rule(".custom"));
    assert!(state.excluded_extensions.contains(&"custom".to_string()));

    // 2. Paths
    assert!(state.add_smart_rule("/opt/var/cache/"));
    assert!(state.excluded_paths.contains(&"/opt/var/cache".to_string()));
    assert!(state.add_smart_rule("~/.npm"));
    assert!(state.excluded_paths.contains(&"~/.npm".to_string()));

    // 3. Folders
    assert!(state.add_smart_rule("vendor/"));
    assert!(state.excluded_folders.contains(&"vendor".to_string()));
    assert!(state.add_smart_rule("dist_out"));
    assert!(state.excluded_folders.contains(&"dist_out".to_string()));

    // Verify .gitignore export string
    let gitignore = state.to_gitignore_string();
    assert!(gitignore.contains("vendor/"));
    assert!(gitignore.contains("*.xyz"));
    assert!(gitignore.contains("/opt/var/cache/"));
}

#[test]
fn test_preset_toggling() {
    let mut state = IndexManagementState::default();

    assert!(state.is_preset_active(findme_ui::index_mgmt::ExclusionPreset::NodeModules));
    state.toggle_preset(findme_ui::index_mgmt::ExclusionPreset::NodeModules);
    assert!(!state.is_preset_active(findme_ui::index_mgmt::ExclusionPreset::NodeModules));

    state.toggle_preset(findme_ui::index_mgmt::ExclusionPreset::NodeModules);
    assert!(state.is_preset_active(findme_ui::index_mgmt::ExclusionPreset::NodeModules));
}

#[test]
fn test_settings_schema_extension_roundtrip() {
    let mut settings = findme_ui::settings::AppSettings {
        indexed_roots: vec!["/custom/root".to_string()],
        ..Default::default()
    };
    settings.excluded_folders.push("custom_build".to_string());
    settings.excluded_extensions.push("ogg".to_string());
    settings.excluded_paths.push("/custom/cache".to_string());

    let temp_dir = std::env::temp_dir().join(format!("findme_test_{}", std::process::id()));
    let temp_file = temp_dir.join("test_settings.json");

    assert!(settings.save_to_path(&temp_file).is_ok());
    let loaded = findme_ui::settings::AppSettings::load_from_path(&temp_file)
        .expect("must load saved settings");

    assert_eq!(loaded.indexed_roots, vec!["/custom/root"]);
    assert!(loaded
        .excluded_folders
        .contains(&"custom_build".to_string()));
    assert!(loaded.excluded_extensions.contains(&"ogg".to_string()));
    assert!(loaded.excluded_paths.contains(&"/custom/cache".to_string()));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_app_task_14_3_messages_and_rendering() {
    let mut app = FindMeApp::new();

    // 1. Quick root adder
    let _ = app.update(Message::IndexManagement(
        IndexManagementMessage::AddQuickRoot("~/MyTestDir".to_string()),
    ));
    assert!(app
        .state
        .index_mgmt
        .roots
        .iter()
        .any(|r| r.path.to_string_lossy().contains("MyTestDir")));

    // 2. Preset toggling via app message
    let _ = app.update(Message::IndexManagement(
        IndexManagementMessage::TogglePreset(findme_ui::index_mgmt::ExclusionPreset::NodeModules),
    ));
    assert!(!app
        .state
        .index_mgmt
        .is_preset_active(findme_ui::index_mgmt::ExclusionPreset::NodeModules));

    // 3. Extension adding & removing
    let _ = app.update(Message::IndexManagement(
        IndexManagementMessage::AddExcludedExtension("mytestext".to_string()),
    ));
    assert!(app
        .state
        .index_mgmt
        .excluded_extensions
        .contains(&"mytestext".to_string()));
    let _ = app.update(Message::IndexManagement(
        IndexManagementMessage::RemoveExcludedExtension("mytestext".to_string()),
    ));
    assert!(!app
        .state
        .index_mgmt
        .excluded_extensions
        .contains(&"mytestext".to_string()));

    // 4. Folder adding & removing
    let _ = app.update(Message::IndexManagement(
        IndexManagementMessage::AddExcludedFolder("mytestfolder".to_string()),
    ));
    assert!(app
        .state
        .index_mgmt
        .excluded_folders
        .contains(&"mytestfolder".to_string()));
    let _ = app.update(Message::IndexManagement(
        IndexManagementMessage::RemoveExcludedFolder("mytestfolder".to_string()),
    ));
    assert!(!app
        .state
        .index_mgmt
        .excluded_folders
        .contains(&"mytestfolder".to_string()));

    // 5. Raw view toggling & copying
    assert!(!app.state.index_mgmt.view_gitignore_raw);
    let _ = app.update(Message::IndexManagement(
        IndexManagementMessage::ToggleGitignoreRawView,
    ));
    assert!(app.state.index_mgmt.view_gitignore_raw);

    // Verify rendering with raw view enabled
    {
        let _view_raw = findme_ui::views::index_mgmt::view(&app.state);
    }

    let _ = app.update(Message::IndexManagement(
        IndexManagementMessage::CopyGitignoreRules,
    ));
    assert_eq!(
        app.state.notification,
        Some("Copied .gitignore rules to clipboard".to_string())
    );
}

#[test]
fn test_live_root_counting_downloads_and_subdirectories() {
    use findme_core::arena::FileArena;
    use findme_core::record::FileFlags;
    use findme_ui::app::FindMeApp;
    use findme_ui::engine::EngineBridge;
    use findme_ui::message::{IndexManagementMessage, Message};
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    let mut arena = FileArena::new();
    let _ = arena.add_file(
        Path::new("/home/user/document.txt"),
        FileFlags::NONE,
        100,
        10,
    );
    let _ = arena.add_file(
        Path::new("/home/user/Downloads/cv.pdf"),
        FileFlags::NONE,
        200,
        10,
    );
    let _ = arena.add_file(
        Path::new("/home/user/Downloads/archive.zip"),
        FileFlags::NONE,
        300,
        10,
    );
    let _ = arena.add_file(
        Path::new("/home/user/Downloads/sub/image.png"),
        FileFlags::NONE,
        400,
        10,
    );

    let engine = Arc::new(EngineBridge::with_arena(arena));
    let mut app = FindMeApp::with_engine(engine);

    let _ = app.update(Message::IndexManagement(IndexManagementMessage::AddRoot(
        PathBuf::from("/home/user/Downloads"),
    )));

    let dl_root = app
        .state
        .index_mgmt
        .roots
        .iter()
        .find(|r| r.path == Path::new("/home/user/Downloads"))
        .expect("Downloads root present");

    assert_eq!(dl_root.file_count, 3);
    assert_eq!(dl_root.folder_count, 1);
}
