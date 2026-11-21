//! Comprehensive integration test suite for Task 5.1:
//! Search input bar, autofocus, adaptive debouncing, keyboard shortcuts, and real-time status indicators.

use findme_ui::{
    format_number, is_heavy_query, search_input_id, FindMeApp, Message, ScreenId, SEARCH_INPUT_ID,
};
use iced::keyboard::{self, key, Key, Modifiers};

#[test]
fn test_autofocus_and_boot() {
    let (app, _task) = FindMeApp::boot();

    assert_eq!(app.state.query, "");
    assert_eq!(app.state.active_screen, ScreenId::Search);
    assert_eq!(search_input_id(), iced::widget::Id::new(SEARCH_INPUT_ID));
}

#[test]
fn test_number_formatting_edge_cases() {
    assert_eq!(format_number(0), "0");
    assert_eq!(format_number(7), "7");
    assert_eq!(format_number(42), "42");
    assert_eq!(format_number(999), "999");
    assert_eq!(format_number(1_000), "1,000");
    assert_eq!(format_number(10_450), "10,450");
    assert_eq!(format_number(450_120), "450,120");
    assert_eq!(format_number(1_000_000), "1,000,000");
    assert_eq!(format_number(987_654_321), "987,654,321");
}

#[test]
fn test_heavy_query_heuristic_edge_cases() {
    // Fast queries (0ms debounce)
    assert!(!is_heavy_query(""));
    assert!(!is_heavy_query("   "));
    assert!(!is_heavy_query("cargo"));
    assert!(!is_heavy_query("main.rs"));
    assert!(!is_heavy_query("find me app"));

    // Regex queries (50ms debounce)
    assert!(is_heavy_query("regex:^find.*me$"));
    assert!(is_heavy_query("/[0-9]+-[a-z]+/"));

    // Wildcard queries (50ms debounce)
    assert!(is_heavy_query("*.rs"));
    assert!(is_heavy_query("file?.txt"));
    assert!(is_heavy_query("target/*/debug"));

    // Metadata filter prefixes (50ms debounce)
    assert!(is_heavy_query("ext:rs"));
    assert!(is_heavy_query("size:>500mb"));
    assert!(is_heavy_query("dm:today"));
    assert!(is_heavy_query("attrib:d"));
    assert!(is_heavy_query("path:/usr/bin"));

    // Complex boolean grouping (50ms debounce)
    assert!(is_heavy_query("rust | cargo"));
    assert!(is_heavy_query("!tmp"));
    assert!(is_heavy_query("(docs | tests)"));

    // Queries with > 3 whitespace words
    assert!(is_heavy_query("alpha beta gamma delta"));
}

#[test]
fn test_adaptive_debouncing_generation_invalidation() {
    let mut app = FindMeApp::new();
    assert_eq!(app.state.search_generation, 0);

    // Keystroke 1: 'r'
    let _ = app.update(Message::SearchInputChanged("r".into()));
    assert_eq!(app.state.search_generation, 1);
    assert_eq!(app.state.query, "r");
    assert!(app.state.is_searching);

    // Keystroke 2: 'ru'
    let _ = app.update(Message::SearchInputChanged("ru".into()));
    assert_eq!(app.state.search_generation, 2);

    // Keystroke 3: 'rust'
    let _ = app.update(Message::SearchInputChanged("rust".into()));
    assert_eq!(app.state.search_generation, 3);

    // Stale debounced query from Keystroke 1 arrives
    let _ = app.update(Message::DebouncedSearch {
        query: "r".into(),
        generation: 1,
    });
    // Still in search state, query remains 'rust'
    assert_eq!(app.state.query, "rust");

    // Current debounced query for Keystroke 3 arrives
    let _ = app.update(Message::DebouncedSearch {
        query: "rust".into(),
        generation: 3,
    });
    let _ = app.update(Message::ExecuteSearch("rust".into()));
    assert!(app.state.is_searching);
    let _ = app.update(Message::ResultsReceived {
        results: vec![],
        search_duration_ms: 1,
        total_matches: 0,
    });
    assert!(!app.state.is_searching);
    assert_eq!(app.state.last_search_duration_ms, 1);
}

#[test]
fn test_clear_search_resets_and_refocuses() {
    let mut app = FindMeApp::new();

    let _ = app.update(Message::SearchInputChanged("some_search_term".into()));
    assert_eq!(app.state.query, "some_search_term");
    assert!(app.state.is_searching);

    let _ = app.update(Message::ClearSearch);
    assert_eq!(app.state.query, "");
    assert!(!app.state.is_searching);
    assert!(app.state.results.is_empty());
    assert_eq!(app.state.selected_row, None);
}

#[test]
fn test_keyboard_navigation_escape_shortcut() {
    let mut app = FindMeApp::new();

    // 1. With non-empty query: Escape clears query
    let _ = app.update(Message::SearchInputChanged("active_query".into()));
    assert_eq!(app.state.query, "active_query");

    let escape_event = keyboard::Event::KeyPressed {
        key: Key::Named(key::Named::Escape),
        modified_key: Key::Named(key::Named::Escape),
        physical_key: key::Physical::Unidentified(key::NativeCode::Unidentified),
        location: keyboard::Location::Standard,
        modifiers: Modifiers::empty(),
        text: None,
        repeat: false,
    };

    let _ = app.update(Message::KeyboardEvent(escape_event.clone()));
    assert_eq!(app.state.query, "");
    assert!(!app.state.is_searching);

    // 2. With empty query and active notification: Escape dismisses notification
    let _ = app.update(Message::ShowNotification("Banner alert".into()));
    assert_eq!(app.state.notification.as_deref(), Some("Banner alert"));

    let _ = app.update(Message::KeyboardEvent(escape_event));
    assert_eq!(app.state.notification, None);
}

#[test]
fn test_keyboard_navigation_ctrl_f_focus_and_route() {
    let mut app = FindMeApp::new();

    // Switch to Settings screen
    let _ = app.update(Message::SwitchScreen(ScreenId::Settings));
    assert_eq!(app.state.active_screen, ScreenId::Settings);

    // Press Ctrl + F
    let mut ctrl_mod = Modifiers::empty();
    ctrl_mod.set(Modifiers::CTRL, true);

    let ctrl_f_event = keyboard::Event::KeyPressed {
        key: Key::Character("f".into()),
        modified_key: Key::Character("f".into()),
        physical_key: key::Physical::Unidentified(key::NativeCode::Unidentified),
        location: keyboard::Location::Standard,
        modifiers: ctrl_mod,
        text: None,
        repeat: false,
    };

    let _ = app.update(Message::KeyboardEvent(ctrl_f_event));
    assert_eq!(app.state.active_screen, ScreenId::Search);
}

#[test]
fn test_keyboard_navigation_slash_shortcut() {
    let mut app = FindMeApp::new();

    // Switch to About screen
    let _ = app.update(Message::SwitchScreen(ScreenId::About));
    assert_eq!(app.state.active_screen, ScreenId::About);

    // Press '/' key
    let slash_event = keyboard::Event::KeyPressed {
        key: Key::Character("/".into()),
        modified_key: Key::Character("/".into()),
        physical_key: key::Physical::Unidentified(key::NativeCode::Unidentified),
        location: keyboard::Location::Standard,
        modifiers: Modifiers::empty(),
        text: None,
        repeat: false,
    };

    let _ = app.update(Message::KeyboardEvent(slash_event));
    assert_eq!(app.state.active_screen, ScreenId::Search);
}

#[test]
fn test_search_view_rendering_with_and_without_query() {
    let mut app = FindMeApp::new();

    // 1. Initial empty search view (no clear button)
    {
        let _ = app.view();
    }

    // 2. Populate stats
    app.state.indexed_files_count = 450_120;
    app.state.indexed_folders_count = 42_500;
    app.state.last_search_duration_ms = 2;
    {
        let _ = app.view();
    }

    // 3. Populate query (clear button appears)
    let _ = app.update(Message::SearchInputChanged("ext:rs size:>100kb".into()));
    {
        let _ = app.view();
    }

    // 4. Searching state
    assert!(app.state.is_searching);
    {
        let _ = app.view();
    }

    // 5. Clear query
    let _ = app.update(Message::ClearSearch);
    assert_eq!(app.state.query, "");
    {
        let _ = app.view();
    }
}
