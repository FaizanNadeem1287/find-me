//! Comprehensive QA test suite for Task 8.2: Keyboard-First Shortcuts & Navigation
//!
//! Validates 100% keyboard-operability of Find-Me without touching the mouse.

use findme_core::SearchResultItem;
use findme_ui::message::{ColumnId, Message, ScreenId};
use findme_ui::FindMeApp;
use iced::keyboard::key::Named;
use iced::keyboard::{Event, Key, Modifiers};

/// Helper to create key press events
fn key_event(key: Key, modifiers: Modifiers) -> Event {
    Event::KeyPressed {
        key,
        modified_key: Key::Named(Named::Space), // placeholder
        physical_key: iced::keyboard::key::Physical::Code(iced::keyboard::key::Code::Space),
        location: iced::keyboard::Location::Standard,
        modifiers,
        repeat: false,
        text: None,
    }
}

fn sample_items() -> Vec<SearchResultItem> {
    (0u64..30)
        .map(|i| SearchResultItem {
            id: i,
            name: format!("file_{i:02}.rs"),
            path: format!("/projects/src/file_{i:02}.rs"),
            size_bytes: 1024 * (i + 1),
            mtime_secs: 1700000000 + i * 100,
            is_dir: false,
        })
        .collect()
}

#[test]
fn test_ctrl_1_through_6_screen_switching() {
    let mut app = FindMeApp::new();
    assert_eq!(app.state.active_screen, ScreenId::Search);

    let mut ctrl = Modifiers::empty();
    ctrl.insert(Modifiers::CTRL);

    // Ctrl + 2 -> AdvancedSearch
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Character("2".into()),
        ctrl,
    )));
    assert_eq!(app.state.active_screen, ScreenId::AdvancedSearch);

    // Ctrl + 3 -> IndexManagement
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Character("3".into()),
        ctrl,
    )));
    assert_eq!(app.state.active_screen, ScreenId::IndexManagement);

    // Ctrl + 4 -> SearchHistory
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Character("4".into()),
        ctrl,
    )));
    assert_eq!(app.state.active_screen, ScreenId::SearchHistory);

    // Ctrl + 5 -> Settings
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Character("5".into()),
        ctrl,
    )));
    assert_eq!(app.state.active_screen, ScreenId::Settings);

    // Ctrl + 6 -> About
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Character("6".into()),
        ctrl,
    )));
    assert_eq!(app.state.active_screen, ScreenId::About);

    // Ctrl + 1 -> Search
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Character("1".into()),
        ctrl,
    )));
    assert_eq!(app.state.active_screen, ScreenId::Search);
}

#[test]
fn test_focus_trap_prevention_on_screen_switch() {
    let mut app = FindMeApp::new();
    let items = sample_items();
    app.state.results = items;
    app.state.context_menu.is_open = true;

    // Switch screen via Ctrl+3 -> context menu must automatically close
    let mut ctrl = Modifiers::empty();
    ctrl.insert(Modifiers::CTRL);
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Character("3".into()),
        ctrl,
    )));

    assert_eq!(app.state.active_screen, ScreenId::IndexManagement);
    assert!(!app.state.context_menu.is_open);
}

#[test]
fn test_ctrl_b_sidebar_toggle() {
    let mut app = FindMeApp::new();
    let initial = app.state.sidebar_expanded;

    let mut ctrl = Modifiers::empty();
    ctrl.insert(Modifiers::CTRL);

    // Toggle once
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Character("b".into()),
        ctrl,
    )));
    assert_eq!(app.state.sidebar_expanded, !initial);

    // Toggle twice
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Character("B".into()),
        ctrl,
    )));
    assert_eq!(app.state.sidebar_expanded, initial);
}

#[test]
fn test_ctrl_t_theme_toggle() {
    let mut app = FindMeApp::new();
    let initial_theme = app.state.theme;

    let mut ctrl = Modifiers::empty();
    ctrl.insert(Modifiers::CTRL);

    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Character("t".into()),
        ctrl,
    )));
    assert_ne!(app.state.theme, initial_theme);

    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Character("T".into()),
        ctrl,
    )));
    assert_eq!(app.state.theme, initial_theme);
}

#[test]
fn test_f5_and_ctrl_r_query_refresh() {
    let mut app = FindMeApp::new();
    app.state.query = "ext:rs".to_string();

    // 1. F5
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Named(Named::F5),
        Modifiers::empty(),
    )));
    assert!(app.state.is_searching);

    // 2. Ctrl + R
    app.state.is_searching = false;
    let mut ctrl = Modifiers::empty();
    ctrl.insert(Modifiers::CTRL);
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Character("r".into()),
        ctrl,
    )));
    assert!(app.state.is_searching);
}

#[test]
fn test_alt_1_through_5_column_sorting() {
    let mut app = FindMeApp::new();
    app.state.results = sample_items();

    let mut alt = Modifiers::empty();
    alt.insert(Modifiers::ALT);

    // Alt + 1 -> Sort by Name
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Character("1".into()),
        alt,
    )));
    assert_eq!(app.state.sort_column, ColumnId::Name);

    // Alt + 2 -> Sort by Path
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Character("2".into()),
        alt,
    )));
    assert_eq!(app.state.sort_column, ColumnId::Path);

    // Alt + 3 -> Sort by Size
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Character("3".into()),
        alt,
    )));
    assert_eq!(app.state.sort_column, ColumnId::Size);

    // Alt + 4 -> Sort by Date Modified
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Character("4".into()),
        alt,
    )));
    assert_eq!(app.state.sort_column, ColumnId::Modified);

    // Alt + 5 -> Sort by Type
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Character("5".into()),
        alt,
    )));
    assert_eq!(app.state.sort_column, ColumnId::Type);
}

#[test]
fn test_ctrl_shift_c_n_u_clipboard_shortcuts() {
    let mut app = FindMeApp::new();
    app.state.results = sample_items();
    app.state.selected_row = Some(3);

    let mut ctrl_shift = Modifiers::empty();
    ctrl_shift.insert(Modifiers::CTRL);
    ctrl_shift.insert(Modifiers::SHIFT);

    // 1. Ctrl + Shift + C -> Copy Path
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Character("c".into()),
        ctrl_shift,
    )));
    assert!(app.state.notification.is_some());
    assert!(app.state.notification.as_ref().unwrap().contains("path"));

    // 2. Ctrl + Shift + N -> Copy File Name
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Character("n".into()),
        ctrl_shift,
    )));
    assert!(app.state.notification.is_some());
    assert!(app
        .state
        .notification
        .as_ref()
        .unwrap()
        .contains("file_03.rs"));

    // 3. Ctrl + Shift + U -> Copy File URL
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Character("u".into()),
        ctrl_shift,
    )));
    assert!(app.state.notification.is_some());
}

#[test]
fn test_selection_navigation_key_stepping() {
    let mut app = FindMeApp::new();
    app.state.results = sample_items();
    assert_eq!(app.state.results.len(), 30);
    app.state.selected_row = Some(0);

    // Down arrow steps by 1
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Named(Named::ArrowDown),
        Modifiers::empty(),
    )));
    assert_eq!(app.state.selected_row, Some(1));

    // PageDown steps by 15
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Named(Named::PageDown),
        Modifiers::empty(),
    )));
    assert_eq!(app.state.selected_row, Some(16));

    // End jumps to last item (29)
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Named(Named::End),
        Modifiers::empty(),
    )));
    assert_eq!(app.state.selected_row, Some(29));

    // PageUp steps up by 15
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Named(Named::PageUp),
        Modifiers::empty(),
    )));
    assert_eq!(app.state.selected_row, Some(14));

    // Up arrow steps by 1
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Named(Named::ArrowUp),
        Modifiers::empty(),
    )));
    assert_eq!(app.state.selected_row, Some(13));

    // Home jumps to 0
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Named(Named::Home),
        Modifiers::empty(),
    )));
    assert_eq!(app.state.selected_row, Some(0));
}

#[test]
fn test_escape_hierarchy_dismissal() {
    let mut app = FindMeApp::new();

    // 1. Level 1: Properties modal open
    let _ = app.update(Message::ShowProperties("Cargo.toml".to_string()));
    assert!(app.state.properties_modal.is_some());
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Named(Named::Escape),
        Modifiers::empty(),
    )));
    assert!(app.state.properties_modal.is_none());

    // 2. Level 2: Context menu open
    app.state.context_menu.is_open = true;
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Named(Named::Escape),
        Modifiers::empty(),
    )));
    assert!(!app.state.context_menu.is_open);

    // 3. Level 3: Non-empty query
    app.state.query = "active_search_text".to_string();
    app.state.results = sample_items();
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Named(Named::Escape),
        Modifiers::empty(),
    )));
    assert_eq!(app.state.query, "");
    assert!(app.state.results.is_empty());

    // 4. Level 4: Notification banner
    app.state.notification = Some("Notice message".to_string());
    let _ = app.update(Message::KeyboardEvent(key_event(
        Key::Named(Named::Escape),
        Modifiers::empty(),
    )));
    assert!(app.state.notification.is_none());
}
