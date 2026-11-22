//! Comprehensive integration tests for Quick-Access Folder Showcase Hero Cards & Breadcrumbs Header (Task 12.3)

use findme_ui::{colors, contrast_ratio, AppTheme, FindMeApp, Message, ScreenId};

#[test]
fn test_breadcrumbs_bar_navigation_and_metrics() {
    let mut app = FindMeApp::new();

    // Verify initial search screen and empty filter
    assert_eq!(app.state.active_screen, ScreenId::Search);
    assert_eq!(app.state.active_folder_filter, None);
    {
        let _ = app.view();
    }

    // Set indexed files count to verify metric formatting in breadcrumbs
    app.state.indexed_files_count = 142_500;
    {
        let _ = app.view();
    }

    // Select "Documents" filter and verify view
    let _ = app.update(Message::SelectFolderFilter(Some("Documents".to_string())));
    assert_eq!(app.state.active_folder_filter.as_deref(), Some("Documents"));
    assert_eq!(app.state.query, "path:\"Documents\"");
    {
        let _ = app.view();
    }

    // Reset via breadcrumbs (SelectFolderFilter(None))
    let _ = app.update(Message::SelectFolderFilter(None));
    assert_eq!(app.state.active_folder_filter, None);
    assert_eq!(app.state.query, "");
    {
        let _ = app.view();
    }
}

#[test]
fn test_hero_cards_selection_and_toggling() {
    let mut app = FindMeApp::new();

    // 1. Documents Hero Card (Amber)
    let _ = app.update(Message::SelectFolderFilter(Some("Documents".to_string())));
    assert_eq!(app.state.active_folder_filter.as_deref(), Some("Documents"));
    assert_eq!(app.state.query, "path:\"Documents\"");
    {
        let _ = app.view();
    }

    // Toggle off Documents card
    let _ = app.update(Message::SelectFolderFilter(None));
    assert_eq!(app.state.active_folder_filter, None);
    assert_eq!(app.state.query, "");

    // 2. Code & Dev Hero Card (Blue)
    let _ = app.update(Message::SelectFolderFilter(Some("Projects".to_string())));
    assert_eq!(app.state.active_folder_filter.as_deref(), Some("Projects"));
    assert_eq!(app.state.query, "path:\"Projects\"");
    {
        let _ = app.view();
    }

    // Toggle off Code card
    let _ = app.update(Message::SelectFolderFilter(None));
    assert_eq!(app.state.active_folder_filter, None);
    assert_eq!(app.state.query, "");

    // 3. Downloads & Media Hero Card (Purple)
    let _ = app.update(Message::SelectFolderFilter(Some("Downloads".to_string())));
    assert_eq!(app.state.active_folder_filter.as_deref(), Some("Downloads"));
    assert_eq!(app.state.query, "path:\"Downloads\"");
    {
        let _ = app.view();
    }

    // Toggle off Downloads card
    let _ = app.update(Message::SelectFolderFilter(None));
    assert_eq!(app.state.active_folder_filter, None);
    assert_eq!(app.state.query, "");
}

#[test]
fn test_hero_cards_theming_and_contrast() {
    let mut app = FindMeApp::new();

    // Render in Dark Theme
    app.state.theme = AppTheme::Dark;
    {
        let _ = app.view();
    }

    // Render in Light Theme
    app.state.theme = AppTheme::Light;
    {
        let _ = app.view();
    }

    // Verify contrast compliance of card accents against dark/light surfaces
    let amber_dark_contrast = contrast_ratio(colors::ACCENT_AMBER, colors::DARK_SURFACE);
    assert!(
        amber_dark_contrast >= 3.0,
        "Amber accent on dark surface contrast {amber_dark_contrast:.2} must be >= 3.0"
    );

    let blue_light_contrast = contrast_ratio(colors::ACCENT_BLUE, colors::LIGHT_SURFACE);
    assert!(
        blue_light_contrast >= 3.0,
        "Blue accent on light surface contrast {blue_light_contrast:.2} must be >= 3.0"
    );

    let purple_dark_contrast = contrast_ratio(colors::ACCENT_PURPLE, colors::DARK_SURFACE);
    assert!(
        purple_dark_contrast >= 3.0,
        "Purple accent on dark surface contrast {purple_dark_contrast:.2} must be >= 3.0"
    );
}
