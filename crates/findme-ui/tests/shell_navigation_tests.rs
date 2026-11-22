//! Headless integration test suite for Find-Me application shell, Elm loop, and navigation routes

use findme_core::SearchResultItem;
use findme_ui::{window_settings, ColumnId, FindMeApp, Message, ScreenId};
use iced::Size;

#[test]
fn test_shell_initial_state() {
    let app = FindMeApp::new();

    assert_eq!(app.state.active_screen, ScreenId::Search);
    assert!(app.state.sidebar_expanded);
    assert_eq!(app.state.window_width, 1200.0);
    assert_eq!(app.state.window_height, 800.0);
    assert_eq!(app.state.theme, app.state.theme_pref.resolve());
    assert_eq!(app.state.query, "");
    assert!(app.state.results.is_empty());
    assert_eq!(app.state.notification, None);
    assert_eq!(app.title(), "Find-Me — Search");
}

#[test]
fn test_shell_window_settings() {
    let settings = window_settings();

    assert_eq!(settings.size, Size::new(1200.0, 800.0));
    assert_eq!(settings.min_size, Some(Size::new(800.0, 600.0)));
    assert!(settings.resizable);
    assert!(settings.decorations);
}

#[test]
fn test_shell_navigation_all_routes() {
    let mut app = FindMeApp::new();

    for screen in ScreenId::ALL {
        let _ = app.update(Message::SwitchScreen(screen));
        assert_eq!(app.state.active_screen, screen);
        assert_eq!(app.title(), format!("Find-Me — {}", screen.title()));
    }
}

#[test]
fn test_shell_view_rendering_all_screens() {
    let mut app = FindMeApp::new();

    // Verify rendering for all 6 screens in both expanded and collapsed sidebar modes
    for screen in ScreenId::ALL {
        let _ = app.update(Message::SwitchScreen(screen));

        // Expanded mode
        app.state.sidebar_expanded = true;
        {
            let _ = app.view();
        }

        // Collapsed mode
        app.state.sidebar_expanded = false;
        {
            let _ = app.view();
        }
    }
}

#[test]
fn test_shell_sidebar_toggle() {
    let mut app = FindMeApp::new();
    assert!(app.state.sidebar_expanded);

    let _ = app.update(Message::ToggleSidebar);
    assert!(!app.state.sidebar_expanded);

    let _ = app.update(Message::ToggleSidebar);
    assert!(app.state.sidebar_expanded);
}

#[test]
fn test_shell_window_resize() {
    let mut app = FindMeApp::new();

    let _ = app.update(Message::WindowResized {
        width: 1920.0,
        height: 1080.0,
    });
    assert_eq!(app.state.window_width, 1920.0);
    assert_eq!(app.state.window_height, 1080.0);
}

#[test]
fn test_shell_notification_lifecycle() {
    let mut app = FindMeApp::new();
    assert_eq!(app.state.notification, None);

    let _ = app.update(Message::ShowNotification("Index loaded in 34ms".into()));
    assert_eq!(
        app.state.notification.as_deref(),
        Some("Index loaded in 34ms")
    );

    // Verify view with notification banner active
    {
        let _ = app.view();
    }

    let _ = app.update(Message::DismissNotification);
    assert_eq!(app.state.notification, None);

    // Verify view without notification banner
    {
        let _ = app.view();
    }
}

#[test]
fn test_shell_run_query_redirect() {
    let mut app = FindMeApp::new();

    // Start on Advanced Search screen
    let _ = app.update(Message::SwitchScreen(ScreenId::AdvancedSearch));
    assert_eq!(app.state.active_screen, ScreenId::AdvancedSearch);

    // Run query should transition user back to Search screen with query prefilled
    let _ = app.update(Message::RunQuery("ext:rs size:>100kb".into()));
    assert_eq!(app.state.active_screen, ScreenId::Search);
    assert_eq!(app.state.query, "ext:rs size:>100kb");
    assert!(app.state.is_searching);
}

#[test]
fn test_shell_theme_toggle() {
    let mut app = FindMeApp::new();
    let initial_theme = app.state.theme;

    let _ = app.update(Message::ToggleTheme);
    assert_eq!(app.state.theme, initial_theme.toggle());

    let _ = app.update(Message::ToggleTheme);
    assert_eq!(app.state.theme, initial_theme);
}

#[test]
fn test_shell_search_results_and_sorting() {
    let mut app = FindMeApp::new();

    let item1 = SearchResultItem {
        id: 1,
        name: "b_file.txt".to_string(),
        path: "/home/user/b_file.txt".to_string(),
        size_bytes: 200,
        mtime_secs: 1000,
        is_dir: false,
    };

    let item2 = SearchResultItem {
        id: 2,
        name: "a_file.rs".to_string(),
        path: "/home/user/a_file.rs".to_string(),
        size_bytes: 100,
        mtime_secs: 2000,
        is_dir: false,
    };

    let _ = app.update(Message::ResultsReceived {
        results: vec![item1, item2],
        search_duration_ms: 5,
        total_matches: 2,
    });

    assert_eq!(app.state.results.len(), 2);
    assert_eq!(app.state.last_search_duration_ms, 5);

    // Sort by Name (Default Ascending)
    assert_eq!(app.state.results[0].name, "a_file.rs");
    assert_eq!(app.state.results[1].name, "b_file.txt");

    // Toggle Sort by Name (Descending)
    let _ = app.update(Message::SortByColumn(ColumnId::Name));
    assert_eq!(app.state.results[0].name, "b_file.txt");
    assert_eq!(app.state.results[1].name, "a_file.rs");

    // Sort by Size (Ascending)
    let _ = app.update(Message::SortByColumn(ColumnId::Size));
    assert_eq!(app.state.results[0].size_bytes, 100);
    assert_eq!(app.state.results[1].size_bytes, 200);

    // Sort by Modified (Ascending)
    let _ = app.update(Message::SortByColumn(ColumnId::Modified));
    assert_eq!(app.state.results[0].mtime_secs, 1000);
    assert_eq!(app.state.results[1].mtime_secs, 2000);

    // Clear Search
    let _ = app.update(Message::ClearSearch);
    assert_eq!(app.state.query, "");
    assert!(app.state.results.is_empty());
}

#[test]
fn test_sidebar_tree_node_expansion_and_filtering() {
    let mut app = FindMeApp::new();

    // Verify initial folder tree state
    assert!(app.state.expanded_folders.contains("General Knowledge"));
    assert_eq!(app.state.active_folder_filter, None);

    // Render initial view with folder tree
    {
        let _ = app.view();
    }

    // Toggle tree node collapse
    let _ = app.update(Message::ToggleFolderTreeNode(
        "General Knowledge".to_string(),
    ));
    assert!(!app.state.expanded_folders.contains("General Knowledge"));

    // Render with tree node collapsed
    {
        let _ = app.view();
    }

    // Toggle tree node expand again
    let _ = app.update(Message::ToggleFolderTreeNode(
        "General Knowledge".to_string(),
    ));
    assert!(app.state.expanded_folders.contains("General Knowledge"));

    // Select child folder filter
    let _ = app.update(Message::SelectFolderFilter(Some("Projects".to_string())));
    assert_eq!(app.state.active_folder_filter.as_deref(), Some("Projects"));
    assert_eq!(app.state.query, "path:\"Projects\"");
    assert_eq!(app.state.active_screen, ScreenId::Search);

    // Clear folder filter
    let _ = app.update(Message::SelectFolderFilter(None));
    assert_eq!(app.state.active_folder_filter, None);
    assert_eq!(app.state.query, "");
}

#[test]
fn test_ctrl_k_quick_search_shortcut() {
    let mut app = FindMeApp::new();

    // Switch to non-search screen
    let _ = app.update(Message::SwitchScreen(ScreenId::Settings));
    assert_eq!(app.state.active_screen, ScreenId::Settings);

    // Trigger Ctrl+K
    let ctrl_k = iced::keyboard::Event::KeyPressed {
        key: iced::keyboard::Key::Character("k".into()),
        modified_key: iced::keyboard::Key::Character("k".into()),
        physical_key: iced::keyboard::key::Physical::Code(iced::keyboard::key::Code::KeyK),
        location: iced::keyboard::Location::Standard,
        modifiers: iced::keyboard::Modifiers::CTRL,
        text: None,
        repeat: false,
    };

    let _ = app.update(Message::KeyboardEvent(ctrl_k));
    assert_eq!(app.state.active_screen, ScreenId::Search);
}
