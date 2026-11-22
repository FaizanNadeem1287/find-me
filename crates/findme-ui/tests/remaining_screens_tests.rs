//! Comprehensive test suite for Search History, Application Settings, and About Screens

use findme_ui::app::FindMeApp;
use findme_ui::history::{HistoryEntry, SearchHistoryState};
use findme_ui::message::{HistoryMessage, Message, ScreenId, SettingsMessage};
use findme_ui::settings::AppSettings;

#[test]
fn test_history_entry_formatting() {
    let now = 1_700_000_000;
    let entry = HistoryEntry {
        id: 1,
        query: "ext:rs".to_string(),
        timestamp_secs: now - 30, // 30s ago
        match_count: 50,
        duration_ms: 2,
        is_bookmarked: false,
    };

    assert_eq!(entry.formatted_matches(), "50 matches");
    assert_eq!(entry.formatted_relative_time(now), "30s ago");

    // Relative times: minutes, hours, days, just now
    let entry_mins = HistoryEntry {
        timestamp_secs: now - 300, // 5m ago
        ..entry.clone()
    };
    assert_eq!(entry_mins.formatted_relative_time(now), "5m ago");

    let entry_hours = HistoryEntry {
        timestamp_secs: now - 7200, // 2h ago
        ..entry.clone()
    };
    assert_eq!(entry_hours.formatted_relative_time(now), "2h ago");

    let entry_days = HistoryEntry {
        timestamp_secs: now - 172_800, // 2d ago
        ..entry.clone()
    };
    assert_eq!(entry_days.formatted_relative_time(now), "2d ago");

    let entry_future = HistoryEntry {
        timestamp_secs: now + 10,
        ..entry
    };
    assert_eq!(entry_future.formatted_relative_time(now), "Just now");
}

#[test]
fn test_history_record_query() {
    let mut history = SearchHistoryState::default();
    let now = 1_700_000_000;

    // Record first query
    history.record_query("ext:py", 120, 5, now);
    assert_eq!(history.entries.len(), 1);
    assert_eq!(history.entries[0].query, "ext:py");
    assert_eq!(history.entries[0].match_count, 120);

    // Empty query should be ignored
    history.record_query("", 0, 0, now);
    history.record_query("   ", 0, 0, now);
    assert_eq!(history.entries.len(), 1);

    // Record second query
    history.record_query("Cargo.lock", 10, 1, now + 10);
    assert_eq!(history.entries.len(), 2);
    assert_eq!(history.entries[0].query, "Cargo.lock");

    // Bookmark the first query
    let py_id = history.entries[1].id;
    assert!(history.toggle_bookmark(py_id));
    assert!(history.entries[1].is_bookmarked);

    // Re-recording "ext:py" should move it to top and keep bookmark intact
    history.record_query("ext:py", 150, 4, now + 20);
    assert_eq!(history.entries.len(), 2);
    assert_eq!(history.entries[0].query, "ext:py");
    assert_eq!(history.entries[0].match_count, 150);
    assert!(history.entries[0].is_bookmarked);
}

#[test]
fn test_history_filtering() {
    let mut history = SearchHistoryState::default();
    let now = 1_700_000_000;

    history.record_query("ext:rs", 10, 1, now);
    history.record_query("ext:c", 20, 2, now);
    history.record_query("Cargo.toml", 5, 1, now);

    // Bookmark one query
    let rs_id = history
        .entries
        .iter()
        .find(|e| e.query == "ext:rs")
        .unwrap()
        .id;
    history.toggle_bookmark(rs_id);

    // All entries
    assert_eq!(history.filtered_entries().len(), 3);

    // Starred only
    history.filter_bookmarked_only = true;
    let starred = history.filtered_entries();
    assert_eq!(starred.len(), 1);
    assert_eq!(starred[0].query, "ext:rs");

    // Reset bookmark filter and apply search filter
    history.filter_bookmarked_only = false;
    history.search_filter = "cargo".to_string();
    let search_results = history.filtered_entries();
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].query, "Cargo.toml");

    // Remove entry
    assert!(history.remove_entry(rs_id));
    assert_eq!(history.entries.len(), 2);

    // Clear history
    history.clear();
    assert!(history.entries.is_empty());
}

#[test]
fn test_settings_defaults_and_mutations() {
    let mut settings = AppSettings::default();
    assert!(!settings.autostart);
    assert!(settings.close_to_tray);
    assert!(!settings.default_regex);
    assert!(!settings.default_case_sensitive);
    assert_eq!(settings.max_results, 1000);
    assert!(settings.exclude_hidden_files);
    assert!(settings.exclude_system_paths);

    // Mutate settings
    settings.autostart = true;
    settings.default_regex = true;
    settings.max_results = 50_000;

    assert!(settings.autostart);
    assert!(settings.default_regex);
    assert_eq!(settings.max_results, 50_000);
}

#[test]
fn test_settings_json_persistence_roundtrip() {
    let temp_dir = tempfile::tempdir().expect("Failed to create tempdir");
    let config_file = temp_dir.path().join("sub_dir").join("settings.json");

    let original = AppSettings {
        autostart: true,
        close_to_tray: false,
        default_regex: true,
        default_case_sensitive: true,
        max_results: 10_000,
        exclude_hidden_files: false,
        exclude_system_paths: true,
        ..AppSettings::default()
    };

    // Save to disk (verifying parent dir auto-creation)
    assert!(original.save_to_path(&config_file).is_ok());

    // Load from disk
    let loaded = AppSettings::load_from_path(&config_file).expect("Failed to load settings");
    assert_eq!(original, loaded);

    // Verify non-existent file error handling
    let non_existent = temp_dir.path().join("missing.json");
    assert!(AppSettings::load_from_path(&non_existent).is_err());
}

#[test]
fn test_app_history_message_handling() {
    let mut app = FindMeApp::new();

    // 1. Execute query from history (routes to Search screen and sets query)
    let _ = app.update(Message::SwitchScreen(ScreenId::SearchHistory));
    assert_eq!(app.state.active_screen, ScreenId::SearchHistory);

    let _ = app.update(Message::History(HistoryMessage::ExecuteQuery(
        "ext:toml size:>5kb".to_string(),
    )));
    assert_eq!(app.state.active_screen, ScreenId::Search);
    assert_eq!(app.state.query, "ext:toml size:>5kb");

    // 2. Toggle Bookmark
    if let Some(first) = app.state.history.entries.first() {
        let first_id = first.id;
        let initial_bookmarked = first.is_bookmarked;
        let _ = app.update(Message::History(HistoryMessage::ToggleBookmark(first_id)));
        assert_ne!(
            app.state
                .history
                .entries
                .iter()
                .find(|e| e.id == first_id)
                .unwrap()
                .is_bookmarked,
            initial_bookmarked
        );
    }

    // 3. Search Filter & Bookmark Filter
    let _ = app.update(Message::History(HistoryMessage::UpdateSearchFilter(
        "Cargo".to_string(),
    )));
    assert_eq!(app.state.history.search_filter, "Cargo");

    let _ = app.update(Message::History(HistoryMessage::ToggleFilterBookmarked(
        true,
    )));
    assert!(app.state.history.filter_bookmarked_only);

    // 4. Clear History
    let _ = app.update(Message::History(HistoryMessage::ClearHistory));
    assert!(app.state.history.entries.is_empty());
    assert_eq!(
        app.state.notification,
        Some("Search history cleared".to_string())
    );
}

#[test]
fn test_app_settings_message_handling() {
    let mut app = FindMeApp::new();
    let _ = app.update(Message::SwitchScreen(ScreenId::Settings));
    assert_eq!(app.state.active_screen, ScreenId::Settings);

    let _ = app.update(Message::Settings(SettingsMessage::ToggleAutostart(true)));
    assert!(app.state.settings.autostart);

    let _ = app.update(Message::Settings(SettingsMessage::ToggleCloseToTray(false)));
    assert!(!app.state.settings.close_to_tray);

    let _ = app.update(Message::Settings(SettingsMessage::ToggleDefaultRegex(true)));
    assert!(app.state.settings.default_regex);

    let _ = app.update(Message::Settings(SettingsMessage::ToggleDefaultCase(true)));
    assert!(app.state.settings.default_case_sensitive);

    let _ = app.update(Message::Settings(SettingsMessage::SetMaxResults(50_000)));
    assert_eq!(app.state.settings.max_results, 50_000);

    let _ = app.update(Message::Settings(SettingsMessage::ToggleExcludeHidden(
        false,
    )));
    assert!(!app.state.settings.exclude_hidden_files);

    let _ = app.update(Message::Settings(SettingsMessage::ToggleExcludeSystem(
        false,
    )));
    assert!(!app.state.settings.exclude_system_paths);

    // Reset to defaults
    let _ = app.update(Message::Settings(SettingsMessage::ResetToDefaults));
    assert_eq!(app.state.settings, AppSettings::default());
    assert_eq!(
        app.state.notification,
        Some("Settings restored to defaults".to_string())
    );
}

#[test]
fn test_all_remaining_views_rendering() {
    let mut app = FindMeApp::new();

    // 1. History View with entries
    {
        let _view_hist = findme_ui::views::history::view(&app.state);
    }

    // 2. History View empty state
    let _ = app.update(Message::History(HistoryMessage::ClearHistory));
    {
        let _view_hist_empty = findme_ui::views::history::view(&app.state);
    }

    // 3. Settings View
    {
        let _view_settings = findme_ui::views::settings::view(&app.state);
    }

    // 4. About View
    {
        let _view_about = findme_ui::views::about::view(&app.state);
    }
}

#[test]
fn test_modern_secondary_screens_theme_toggle_rendering() {
    let mut app = FindMeApp::new();

    for theme in [findme_ui::AppTheme::Dark, findme_ui::AppTheme::Light] {
        app.state.theme = theme;

        // Verify each screen renders cleanly without panic in active theme
        let _adv = findme_ui::views::advanced::view(&app.state);
        let _idx = findme_ui::views::index_mgmt::view(&app.state);
        let _hist = findme_ui::views::history::view(&app.state);
        let _sett = findme_ui::views::settings::view(&app.state);
        let _abt = findme_ui::views::about::view(&app.state);
    }
}
