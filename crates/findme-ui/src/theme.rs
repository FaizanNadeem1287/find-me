//! Custom themes, color palettes, OS theme detection, and WCAG contrast for Find-Me

use iced::Color;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Precision color palettes adhering to the Linux desktop design specification
pub mod colors {
    use iced::Color;

    // Dark Theme Palette
    pub const DARK_BG: Color = Color::from_rgb(
        0x1a as f32 / 255.0,
        0x1b as f32 / 255.0,
        0x1e as f32 / 255.0,
    ); // #1A1B1E
    pub const DARK_SURFACE: Color = Color::from_rgb(
        0x25 as f32 / 255.0,
        0x26 as f32 / 255.0,
        0x2b as f32 / 255.0,
    ); // #25262B
    pub const DARK_BORDER: Color = Color::from_rgb(
        0x2c as f32 / 255.0,
        0x2e as f32 / 255.0,
        0x33 as f32 / 255.0,
    ); // #2C2E33
    pub const DARK_TEXT: Color = Color::from_rgb(
        0xc1 as f32 / 255.0,
        0xc2 as f32 / 255.0,
        0xc5 as f32 / 255.0,
    ); // #C1C2C5
    pub const DARK_SUBTEXT: Color = Color::from_rgb(
        0x90 as f32 / 255.0,
        0x92 as f32 / 255.0,
        0x96 as f32 / 255.0,
    ); // #909296
    pub const DARK_ACCENT: Color = Color::from_rgb(
        0x33 as f32 / 255.0,
        0x9a as f32 / 255.0,
        0xf0 as f32 / 255.0,
    ); // #339AF0
    pub const DARK_SUCCESS: Color = Color::from_rgb(
        0x40 as f32 / 255.0,
        0xc0 as f32 / 255.0,
        0x57 as f32 / 255.0,
    ); // #40C057
    pub const DARK_WARNING: Color = Color::from_rgb(
        0xfc as f32 / 255.0,
        0xc4 as f32 / 255.0,
        0x19 as f32 / 255.0,
    ); // #FCC419
    pub const DARK_DANGER: Color = Color::from_rgb(
        0xfa as f32 / 255.0,
        0x52 as f32 / 255.0,
        0x52 as f32 / 255.0,
    ); // #FA5252

    // Light Theme Palette
    pub const LIGHT_BG: Color = Color::from_rgb(
        0xf8 as f32 / 255.0,
        0xf9 as f32 / 255.0,
        0xfa as f32 / 255.0,
    ); // #F8F9FA
    pub const LIGHT_SURFACE: Color = Color::from_rgb(1.0, 1.0, 1.0); // #FFFFFF
    pub const LIGHT_BORDER: Color = Color::from_rgb(
        0xde as f32 / 255.0,
        0xe2 as f32 / 255.0,
        0xe6 as f32 / 255.0,
    ); // #DEE2E6
    pub const LIGHT_TEXT: Color = Color::from_rgb(
        0x21 as f32 / 255.0,
        0x25 as f32 / 255.0,
        0x29 as f32 / 255.0,
    ); // #212529
    pub const LIGHT_SUBTEXT: Color = Color::from_rgb(
        0x6c as f32 / 255.0,
        0x6f as f32 / 255.0,
        0x85 as f32 / 255.0,
    ); // #6C6F85
    pub const LIGHT_ACCENT: Color = Color::from_rgb(
        0x1c as f32 / 255.0,
        0x7e as f32 / 255.0,
        0xd6 as f32 / 255.0,
    ); // #1C7ED6
    pub const LIGHT_SUCCESS: Color = Color::from_rgb(
        0x37 as f32 / 255.0,
        0xb2 as f32 / 255.0,
        0x4d as f32 / 255.0,
    ); // #37B24D
    pub const LIGHT_WARNING: Color = Color::from_rgb(
        0xf5 as f32 / 255.0,
        0x9f as f32 / 255.0,
        0x00 as f32 / 255.0,
    ); // #F59F00
    pub const LIGHT_DANGER: Color = Color::from_rgb(
        0xf0 as f32 / 255.0,
        0x3e as f32 / 255.0,
        0x3e as f32 / 255.0,
    ); // #F03E3E
}

/// Active Application Theme identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AppTheme {
    #[default]
    Dark,
    Light,
}

impl AppTheme {
    /// Returns true if active theme is Dark
    pub fn is_dark(&self) -> bool {
        matches!(self, AppTheme::Dark)
    }

    /// Toggles between Dark and Light themes
    pub fn toggle(&self) -> Self {
        match self {
            AppTheme::Dark => AppTheme::Light,
            AppTheme::Light => AppTheme::Dark,
        }
    }

    /// Converts into a styled Iced Theme
    pub fn as_iced_theme(&self) -> iced::Theme {
        match self {
            AppTheme::Dark => custom_dark_theme(),
            AppTheme::Light => custom_light_theme(),
        }
    }
}

/// User's persistent theme configuration preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ThemePreference {
    /// Automatically follows OS desktop preference (GNOME/KDE/XFCE)
    #[default]
    System,
    /// Explicitly forced Dark mode
    Dark,
    /// Explicitly forced Light mode
    Light,
}

impl ThemePreference {
    /// Resolves the preference into an active `AppTheme`
    pub fn resolve(&self) -> AppTheme {
        match self {
            ThemePreference::System => detect_system_theme(),
            ThemePreference::Dark => AppTheme::Dark,
            ThemePreference::Light => AppTheme::Light,
        }
    }

    /// Returns standard XDG config file path for user theme preference
    pub fn config_path() -> PathBuf {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            if !xdg.trim().is_empty() {
                return PathBuf::from(xdg).join("find-me").join("theme.json");
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join(".config")
                .join("find-me")
                .join("theme.json")
        } else {
            PathBuf::from("/tmp/find-me-theme.json")
        }
    }

    /// Saves theme preference to the specified file path atomically
    pub fn save_to_path(&self, path: &Path) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let data = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(path, data)
    }

    /// Loads theme preference from the specified file path
    pub fn load_from_path(path: &Path) -> Option<Self> {
        if !path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }
}

/// Creates the custom Dark theme for Find-Me
pub fn custom_dark_theme() -> iced::Theme {
    iced::Theme::custom(
        "FindMeDark".to_string(),
        iced::theme::Palette {
            background: colors::DARK_BG,
            text: colors::DARK_TEXT,
            primary: colors::DARK_ACCENT,
            success: colors::DARK_SUCCESS,
            warning: colors::DARK_WARNING,
            danger: colors::DARK_DANGER,
        },
    )
}

/// Creates the custom Light theme for Find-Me
pub fn custom_light_theme() -> iced::Theme {
    iced::Theme::custom(
        "FindMeLight".to_string(),
        iced::theme::Palette {
            background: colors::LIGHT_BG,
            text: colors::LIGHT_TEXT,
            primary: colors::LIGHT_ACCENT,
            success: colors::LIGHT_SUCCESS,
            warning: colors::LIGHT_WARNING,
            danger: colors::LIGHT_DANGER,
        },
    )
}

/// Detects the host Linux system theme preference (GNOME/KDE/XFCE)
pub fn detect_system_theme() -> AppTheme {
    // 1. Check environment variables
    if let Ok(scheme) = std::env::var("COLOR_SCHEME") {
        let s = scheme.to_ascii_lowercase();
        if s.contains("dark") {
            return AppTheme::Dark;
        } else if s.contains("light") {
            return AppTheme::Light;
        }
    }

    if let Ok(gtk_theme) = std::env::var("GTK_THEME") {
        let t = gtk_theme.to_ascii_lowercase();
        if t.contains("dark") {
            return AppTheme::Dark;
        } else if t.contains("light") {
            return AppTheme::Light;
        }
    }

    // 2. Query GNOME desktop interface color-scheme if gsettings is installed
    if let Ok(output) = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output()
    {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
            if s.contains("dark") {
                return AppTheme::Dark;
            } else if s.contains("light") || s.contains("default") {
                return AppTheme::Light;
            }
        }
    }

    // Default to dark theme for Find-Me desktop
    AppTheme::Dark
}

/// Calculates linear luminance of an sRGB color component according to W3C WCAG 2.1
fn linear_channel(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Calculates relative luminance of a color according to W3C WCAG 2.1
pub fn relative_luminance(color: Color) -> f32 {
    let r = linear_channel(color.r);
    let g = linear_channel(color.g);
    let b = linear_channel(color.b);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Calculates the contrast ratio between two colors according to W3C WCAG 2.1
///
/// Returns a value between 1.0 and 21.0. WCAG AA requires >= 4.5 for normal text,
/// and >= 3.0 for large text and UI components.
pub fn contrast_ratio(c1: Color, c2: Color) -> f32 {
    let l1 = relative_luminance(c1);
    let l2 = relative_luminance(c2);
    let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    (lighter + 0.05) / (darker + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wcag_contrast_dark_theme() {
        let text_contrast = contrast_ratio(colors::DARK_TEXT, colors::DARK_BG);
        assert!(
            text_contrast >= 4.5,
            "Dark theme text contrast {text_contrast:.2} must be >= 4.5 (WCAG AA)"
        );

        let accent_contrast = contrast_ratio(colors::DARK_ACCENT, colors::DARK_BG);
        assert!(
            accent_contrast >= 3.0,
            "Dark theme accent contrast {accent_contrast:.2} must be >= 3.0 for UI controls"
        );
    }

    #[test]
    fn test_wcag_contrast_light_theme() {
        let text_contrast = contrast_ratio(colors::LIGHT_TEXT, colors::LIGHT_BG);
        assert!(
            text_contrast >= 4.5,
            "Light theme text contrast {text_contrast:.2} must be >= 4.5 (WCAG AA)"
        );

        let accent_contrast = contrast_ratio(colors::LIGHT_ACCENT, colors::LIGHT_SURFACE);
        assert!(
            accent_contrast >= 3.0,
            "Light theme accent contrast {accent_contrast:.2} must be >= 3.0 for UI controls"
        );
    }
}
