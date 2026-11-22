//! Comprehensive integration tests for revamped modern sidebar navigation component (Task 12.2)

use findme_ui::{colors, contrast_ratio, FindMeApp, HistoryEntry, Message, ScreenId};

#[test]
fn test_sidebar_monogram_and_header_rendering() {
    let mut app = FindMeApp::new();

    // Verify initial expanded mode
    assert!(app.state.sidebar_expanded);
    {
        let _ = app.view();
    }

    // Toggle to collapsed mode
    let _ = app.update(Message::ToggleSidebar);
    assert!(!app.state.sidebar_expanded);
    {
        let _ = app.view();
    }

    // Toggle back to expanded
    let _ = app.update(Message::ToggleSidebar);
    assert!(app.state.sidebar_expanded);
    {
        let _ = app.view();
    }
}

#[test]
fn test_quick_search_pill_and_ctrl_k_interaction() {
    let mut app = FindMeApp::new();

    // 1. Switch to Settings screen
    let _ = app.update(Message::SwitchScreen(ScreenId::Settings));
    assert_eq!(app.state.active_screen, ScreenId::Settings);

    // 2. Click Quick Search pill (triggers FocusSearchInput)
    let _ = app.update(Message::FocusSearchInput);
    assert_eq!(
        app.state.active_screen,
        ScreenId::Search,
        "FocusSearchInput must route back to Search screen"
    );

    // 3. Switch to About screen
    let _ = app.update(Message::SwitchScreen(ScreenId::About));
    assert_eq!(app.state.active_screen, ScreenId::About);

    // 4. Trigger Ctrl+K keyboard shortcut
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
    assert_eq!(
        app.state.active_screen,
        ScreenId::Search,
        "Ctrl+K must route back to Search screen and focus input"
    );
}

#[test]
fn test_navigation_links_and_badges() {
    let mut app = FindMeApp::new();

    // Add search history entries to test badge counter
    app.state.history.entries.push(HistoryEntry {
        id: 1,
        query: "ext:rs test".to_string(),
        timestamp_secs: 1000,
        match_count: 42,
        duration_ms: 3,
        is_bookmarked: false,
    });
    app.state.history.entries.push(HistoryEntry {
        id: 2,
        query: "size:>100mb".to_string(),
        timestamp_secs: 1010,
        match_count: 8,
        duration_ms: 5,
        is_bookmarked: true,
    });

    // Add indexed root
    let root = findme_ui::IndexedRoot::new("/test/path", 100, 10);
    app.state.index_mgmt.roots.push(root);

    // Verify each screen switch and view rendering with badge counters active
    for screen in ScreenId::ALL {
        let _ = app.update(Message::SwitchScreen(screen));
        assert_eq!(app.state.active_screen, screen);

        // Expanded view
        app.state.sidebar_expanded = true;
        {
            let _ = app.view();
        }

        // Collapsed view
        app.state.sidebar_expanded = false;
        {
            let _ = app.view();
        }
    }
}

#[test]
fn test_color_coded_folder_tree_interactions() {
    let mut app = FindMeApp::new();

    // Verify initial folder tree state
    assert!(app.state.expanded_folders.contains("General Knowledge"));
    assert_eq!(app.state.active_folder_filter, None);

    // Toggle tree node collapse
    let _ = app.update(Message::ToggleFolderTreeNode(
        "General Knowledge".to_string(),
    ));
    assert!(!app.state.expanded_folders.contains("General Knowledge"));
    {
        let _ = app.view();
    }

    // Toggle tree node expand again
    let _ = app.update(Message::ToggleFolderTreeNode(
        "General Knowledge".to_string(),
    ));
    assert!(app.state.expanded_folders.contains("General Knowledge"));
    {
        let _ = app.view();
    }

    // 1. Select Amber folder (Documents)
    let _ = app.update(Message::SelectFolderFilter(Some("Documents".to_string())));
    assert_eq!(app.state.active_folder_filter.as_deref(), Some("Documents"));
    assert_eq!(app.state.query, "path:\"Documents\"");
    assert_eq!(app.state.active_screen, ScreenId::Search);
    {
        let _ = app.view();
    }

    // 2. Select Blue folder (Projects)
    let _ = app.update(Message::SelectFolderFilter(Some("Projects".to_string())));
    assert_eq!(app.state.active_folder_filter.as_deref(), Some("Projects"));
    assert_eq!(app.state.query, "path:\"Projects\"");
    {
        let _ = app.view();
    }

    // 3. Select Slate folder (Downloads)
    let _ = app.update(Message::SelectFolderFilter(Some("Downloads".to_string())));
    assert_eq!(app.state.active_folder_filter.as_deref(), Some("Downloads"));
    assert_eq!(app.state.query, "path:\"Downloads\"");
    {
        let _ = app.view();
    }

    // 4. Select Purple folder (Media / Pictures)
    let _ = app.update(Message::SelectFolderFilter(Some("Pictures".to_string())));
    assert_eq!(app.state.active_folder_filter.as_deref(), Some("Pictures"));
    assert_eq!(app.state.query, "path:\"Pictures\"");
    {
        let _ = app.view();
    }

    // 5. Clear folder filter
    let _ = app.update(Message::SelectFolderFilter(None));
    assert_eq!(app.state.active_folder_filter, None);
    assert_eq!(app.state.query, "");
    {
        let _ = app.view();
    }
}

#[test]
fn test_sidebar_theme_toggle_and_wcag_contrast() {
    let mut app = FindMeApp::new();
    let initial_is_dark = app.state.theme.is_dark();
    {
        let _ = app.view();
    }

    // Toggle theme
    let _ = app.update(Message::ToggleTheme);
    assert_ne!(app.state.theme.is_dark(), initial_is_dark);
    {
        let _ = app.view();
    }

    // Verify contrast compliance of sidebar tokens in Light theme
    let light_contrast = contrast_ratio(colors::LIGHT_TEXT, colors::LIGHT_SIDEBAR);
    assert!(
        light_contrast >= 4.5,
        "Light sidebar text contrast {light_contrast:.2} must be >= 4.5 (WCAG AA)"
    );

    // Verify contrast compliance of sidebar tokens in Dark theme
    let dark_contrast = contrast_ratio(colors::DARK_TEXT, colors::DARK_SIDEBAR);
    assert!(
        dark_contrast >= 4.5,
        "Dark sidebar text contrast {dark_contrast:.2} must be >= 4.5 (WCAG AA)"
    );

    // Toggle back to initial theme
    let _ = app.update(Message::ToggleTheme);
    assert_eq!(app.state.theme.is_dark(), initial_is_dark);
}

#[test]
fn test_dynamic_indexed_roots_in_sidebar() {
    let mut app = FindMeApp::new();
    app.state.index_mgmt.roots.clear();

    app.state
        .index_mgmt
        .roots
        .push(findme_ui::IndexedRoot::new("/custom/projects", 150, 12));
    app.state
        .index_mgmt
        .roots
        .push(findme_ui::IndexedRoot::new("/custom/documents", 45, 5));

    // Verify view renders with dynamic roots
    {
        let _ = app.view();
    }

    // Select dynamic root
    let _ = app.update(Message::SelectFolderFilter(Some(
        "/custom/projects".to_string(),
    )));
    assert_eq!(
        app.state.active_folder_filter.as_deref(),
        Some("/custom/projects")
    );
    assert_eq!(app.state.query, "path:\"/custom/projects\"");

    // Toggle off by passing None
    let _ = app.update(Message::SelectFolderFilter(None));
    assert_eq!(app.state.active_folder_filter, None);
    assert_eq!(app.state.query, "");
}
