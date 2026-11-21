//! Comprehensive test suite for Dark/Light theming, system theme sync, and WCAG AA contrast verification

use findme_ui::{
    colors, contrast_ratio, custom_dark_theme, custom_light_theme, detect_system_theme, AppTheme,
    FindMeApp, Message, ThemePreference,
};
use tempfile::tempdir;

#[test]
fn test_custom_dark_theme_properties() {
    let theme = custom_dark_theme();
    let palette = theme.palette();

    assert_eq!(palette.background, colors::DARK_BG);
    assert_eq!(palette.text, colors::DARK_TEXT);
    assert_eq!(palette.primary, colors::DARK_ACCENT);
    assert_eq!(palette.success, colors::DARK_SUCCESS);
    assert_eq!(palette.warning, colors::DARK_WARNING);
    assert_eq!(palette.danger, colors::DARK_DANGER);
}

#[test]
fn test_custom_light_theme_properties() {
    let theme = custom_light_theme();
    let palette = theme.palette();

    assert_eq!(palette.background, colors::LIGHT_BG);
    assert_eq!(palette.text, colors::LIGHT_TEXT);
    assert_eq!(palette.primary, colors::LIGHT_ACCENT);
    assert_eq!(palette.success, colors::LIGHT_SUCCESS);
    assert_eq!(palette.warning, colors::LIGHT_WARNING);
    assert_eq!(palette.danger, colors::LIGHT_DANGER);
}

#[test]
fn test_wcag_contrast_standards_compliance() {
    // ------------------------------------------------------------------------
    // WCAG 2.1 Level AA:
    // - Normal body text requires minimum contrast ratio of 4.5:1
    // - UI components and large text require minimum contrast ratio of 3.0:1
    // ------------------------------------------------------------------------

    // Dark theme contrast checks
    let dark_text_bg = contrast_ratio(colors::DARK_TEXT, colors::DARK_BG);
    assert!(
        dark_text_bg >= 4.5,
        "Dark theme text on background contrast ratio {dark_text_bg:.2} must be >= 4.5:1"
    );

    let dark_text_surface = contrast_ratio(colors::DARK_TEXT, colors::DARK_SURFACE);
    assert!(
        dark_text_surface >= 4.5,
        "Dark theme text on surface contrast ratio {dark_text_surface:.2} must be >= 4.5:1"
    );

    let dark_accent_bg = contrast_ratio(colors::DARK_ACCENT, colors::DARK_BG);
    assert!(
        dark_accent_bg >= 3.0,
        "Dark theme accent on background contrast ratio {dark_accent_bg:.2} must be >= 3.0:1"
    );

    // Light theme contrast checks
    let light_text_bg = contrast_ratio(colors::LIGHT_TEXT, colors::LIGHT_BG);
    assert!(
        light_text_bg >= 4.5,
        "Light theme text on background contrast ratio {light_text_bg:.2} must be >= 4.5:1"
    );

    let light_text_surface = contrast_ratio(colors::LIGHT_TEXT, colors::LIGHT_SURFACE);
    assert!(
        light_text_surface >= 4.5,
        "Light theme text on surface contrast ratio {light_text_surface:.2} must be >= 4.5:1"
    );

    let light_accent_surface = contrast_ratio(colors::LIGHT_ACCENT, colors::LIGHT_SURFACE);
    assert!(
        light_accent_surface >= 3.0,
        "Light theme accent on surface contrast ratio {light_accent_surface:.2} must be >= 3.0:1"
    );
}

#[test]
fn test_theme_preference_persistence_roundtrip() {
    let dir = tempdir().expect("tempdir");
    let config_path = dir.path().join("theme.json");

    // Initially file does not exist
    assert!(ThemePreference::load_from_path(&config_path).is_none());

    // Save Light preference
    ThemePreference::Light
        .save_to_path(&config_path)
        .expect("save Light preference");
    let loaded1 = ThemePreference::load_from_path(&config_path);
    assert_eq!(loaded1, Some(ThemePreference::Light));

    // Overwrite with Dark preference
    ThemePreference::Dark
        .save_to_path(&config_path)
        .expect("save Dark preference");
    let loaded2 = ThemePreference::load_from_path(&config_path);
    assert_eq!(loaded2, Some(ThemePreference::Dark));

    // Overwrite with System preference
    ThemePreference::System
        .save_to_path(&config_path)
        .expect("save System preference");
    let loaded3 = ThemePreference::load_from_path(&config_path);
    assert_eq!(loaded3, Some(ThemePreference::System));
}

#[test]
fn test_system_theme_detection_and_resolution() {
    let detected = detect_system_theme();
    assert!(matches!(detected, AppTheme::Dark | AppTheme::Light));

    assert_eq!(ThemePreference::Dark.resolve(), AppTheme::Dark);
    assert_eq!(ThemePreference::Light.resolve(), AppTheme::Light);
    assert_eq!(ThemePreference::System.resolve(), detected);
}

#[test]
fn test_app_theme_toggle_and_rendering() {
    let mut app = FindMeApp::new();

    // Verify initial rendering
    {
        let _ = app.view();
    }
    let initial_is_dark = app.state.theme.is_dark();

    // Toggle theme
    let _ = app.update(Message::ToggleTheme);
    assert_ne!(app.state.theme.is_dark(), initial_is_dark);

    // Verify view renders cleanly after toggle
    {
        let _ = app.view();
    }

    // Toggle back
    let _ = app.update(Message::ToggleTheme);
    assert_eq!(app.state.theme.is_dark(), initial_is_dark);

    // Verify view renders cleanly again
    {
        let _ = app.view();
    }
}
