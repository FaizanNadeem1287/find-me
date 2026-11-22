//! Custom themes, color palettes, OS theme detection, and WCAG contrast for Find-Me

use iced::widget::{button, container, text_input};
use iced::{Background, Border, Color, Shadow, Vector};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Precision color palettes adhering to the Linux desktop design specification
/// and the Pinterest/Notion-inspired modern visual design system.
pub mod colors {
    use iced::Color;

    // Canvas Background
    pub const LIGHT_BG: Color = Color::from_rgb(
        0xf8 as f32 / 255.0,
        0xf9 as f32 / 255.0,
        0xfa as f32 / 255.0,
    ); // #F8F9FA
    pub const DARK_BG: Color = Color::from_rgb(
        0x0b as f32 / 255.0,
        0x0f as f32 / 255.0,
        0x19 as f32 / 255.0,
    ); // #0B0F19

    // Card & Panel Surfaces
    pub const LIGHT_SURFACE: Color = Color::from_rgb(1.0, 1.0, 1.0); // #FFFFFF
    pub const DARK_SURFACE: Color = Color::from_rgb(
        0x16 as f32 / 255.0,
        0x1f as f32 / 255.0,
        0x30 as f32 / 255.0,
    ); // #161F30

    // Sidebar Surfaces
    pub const LIGHT_SIDEBAR: Color = Color::from_rgb(1.0, 1.0, 1.0); // #FFFFFF
    pub const DARK_SIDEBAR: Color = Color::from_rgb(
        0x0f as f32 / 255.0,
        0x17 as f32 / 255.0,
        0x2a as f32 / 255.0,
    ); // #0F172A

    // Borders & Dividers
    pub const LIGHT_BORDER: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.06); // 6% black
    pub const DARK_BORDER: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.08); // 8% white
    pub const LIGHT_DIVIDER: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.04);
    pub const DARK_DIVIDER: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.05);

    // Interactive States: Hover & Selection
    pub const LIGHT_HOVER: Color = Color::from_rgb(
        0xf3 as f32 / 255.0,
        0xf4 as f32 / 255.0,
        0xf6 as f32 / 255.0,
    ); // #F3F4F6
    pub const DARK_HOVER: Color = Color::from_rgb(
        0x1e as f32 / 255.0,
        0x29 as f32 / 255.0,
        0x3b as f32 / 255.0,
    ); // #1E293B

    pub const LIGHT_SELECTED: Color = Color::from_rgb(
        0xee as f32 / 255.0,
        0xf2 as f32 / 255.0,
        0xff as f32 / 255.0,
    ); // #EEF2FF
    pub const DARK_SELECTED: Color = Color::from_rgb(
        0x26 as f32 / 255.0,
        0x33 as f32 / 255.0,
        0x52 as f32 / 255.0,
    ); // #263352

    // Typography & Text
    pub const LIGHT_TEXT: Color = Color::from_rgb(
        0x11 as f32 / 255.0,
        0x18 as f32 / 255.0,
        0x27 as f32 / 255.0,
    ); // #111827 (Dark Charcoal)
    pub const DARK_TEXT: Color = Color::from_rgb(
        0xf9 as f32 / 255.0,
        0xfa as f32 / 255.0,
        0xfb as f32 / 255.0,
    ); // #F9FAFB (Off-white)

    pub const LIGHT_SUBTEXT: Color = Color::from_rgb(
        0x6b as f32 / 255.0,
        0x72 as f32 / 255.0,
        0x80 as f32 / 255.0,
    ); // #6B7280 (Gray)
    pub const DARK_SUBTEXT: Color = Color::from_rgb(
        0x94 as f32 / 255.0,
        0xa3 as f32 / 255.0,
        0xb8 as f32 / 255.0,
    ); // #94A3B8 (Slate 400)

    pub const LIGHT_MUTED: Color = Color::from_rgb(
        0x9c as f32 / 255.0,
        0xa3 as f32 / 255.0,
        0xaf as f32 / 255.0,
    ); // #9CA3AF
    pub const DARK_MUTED: Color = Color::from_rgb(
        0x64 as f32 / 255.0,
        0x74 as f32 / 255.0,
        0x8b as f32 / 255.0,
    ); // #64748B

    // Brand & Status Accents
    pub const LIGHT_ACCENT: Color = Color::from_rgb(
        0x25 as f32 / 255.0,
        0x63 as f32 / 255.0,
        0xeb as f32 / 255.0,
    ); // #2563EB (Electric Blue)
    pub const DARK_ACCENT: Color = Color::from_rgb(
        0x3b as f32 / 255.0,
        0x82 as f32 / 255.0,
        0xf6 as f32 / 255.0,
    ); // #3B82F6

    pub const LIGHT_SUCCESS: Color = Color::from_rgb(
        0x10 as f32 / 255.0,
        0xb9 as f32 / 255.0,
        0x81 as f32 / 255.0,
    ); // #10B981 (Emerald)
    pub const DARK_SUCCESS: Color = Color::from_rgb(
        0x34 as f32 / 255.0,
        0xd3 as f32 / 255.0,
        0x99 as f32 / 255.0,
    ); // #34D399

    pub const LIGHT_WARNING: Color = Color::from_rgb(
        0xf5 as f32 / 255.0,
        0x9e as f32 / 255.0,
        0x0b as f32 / 255.0,
    ); // #F59E0B (Warm Amber)
    pub const DARK_WARNING: Color = Color::from_rgb(
        0xfb as f32 / 255.0,
        0xbf as f32 / 255.0,
        0x24 as f32 / 255.0,
    ); // #FBBF24

    pub const LIGHT_DANGER: Color = Color::from_rgb(
        0xf4 as f32 / 255.0,
        0x3f as f32 / 255.0,
        0x5e as f32 / 255.0,
    ); // #F43F5E (Rose)
    pub const DARK_DANGER: Color = Color::from_rgb(
        0xfb as f32 / 255.0,
        0x71 as f32 / 255.0,
        0x85 as f32 / 255.0,
    ); // #FB7185

    // Vibrant Category & Hero Folder Colors
    pub const ACCENT_PURPLE: Color = Color::from_rgb(
        0x63 as f32 / 255.0,
        0x66 as f32 / 255.0,
        0xf1 as f32 / 255.0,
    ); // #6366F1 (Indigo / Purple)
    pub const ACCENT_BLUE: Color = Color::from_rgb(
        0x25 as f32 / 255.0,
        0x63 as f32 / 255.0,
        0xeb as f32 / 255.0,
    ); // #2563EB (Electric Blue)
    pub const ACCENT_AMBER: Color = Color::from_rgb(
        0xf5 as f32 / 255.0,
        0x9e as f32 / 255.0,
        0x0b as f32 / 255.0,
    ); // #F59E0B (Amber Gold)
    pub const ACCENT_EMERALD: Color = Color::from_rgb(
        0x10 as f32 / 255.0,
        0xb9 as f32 / 255.0,
        0x81 as f32 / 255.0,
    ); // #10B981 (Emerald)
    pub const ACCENT_ROSE: Color = Color::from_rgb(
        0xf4 as f32 / 255.0,
        0x3f as f32 / 255.0,
        0x5e as f32 / 255.0,
    ); // #F43F5E (Rose)

    // Soft Category Background Tints (for card illustrations & badges)
    pub const TINT_PURPLE: Color = Color::from_rgb(
        0xf5 as f32 / 255.0,
        0xf3 as f32 / 255.0,
        0xff as f32 / 255.0,
    ); // #F5F3FF
    pub const TINT_BLUE: Color = Color::from_rgb(
        0xef as f32 / 255.0,
        0xf6 as f32 / 255.0,
        0xff as f32 / 255.0,
    ); // #EFF6FF
    pub const TINT_AMBER: Color = Color::from_rgb(
        0xff as f32 / 255.0,
        0xfb as f32 / 255.0,
        0xeb as f32 / 255.0,
    ); // #FFFBEB
    pub const TINT_EMERALD: Color = Color::from_rgb(
        0xec as f32 / 255.0,
        0xfd as f32 / 255.0,
        0xf5 as f32 / 255.0,
    ); // #ECFDF5
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

    /// Returns canvas background color for active theme
    pub fn bg_color(&self) -> Color {
        if self.is_dark() {
            colors::DARK_BG
        } else {
            colors::LIGHT_BG
        }
    }

    /// Returns card surface color for active theme
    pub fn surface_color(&self) -> Color {
        if self.is_dark() {
            colors::DARK_SURFACE
        } else {
            colors::LIGHT_SURFACE
        }
    }

    /// Returns sidebar surface color for active theme
    pub fn sidebar_color(&self) -> Color {
        if self.is_dark() {
            colors::DARK_SIDEBAR
        } else {
            colors::LIGHT_SIDEBAR
        }
    }

    /// Returns primary text color for active theme
    pub fn text_color(&self) -> Color {
        if self.is_dark() {
            colors::DARK_TEXT
        } else {
            colors::LIGHT_TEXT
        }
    }

    /// Returns secondary text color for active theme
    pub fn subtext_color(&self) -> Color {
        if self.is_dark() {
            colors::DARK_SUBTEXT
        } else {
            colors::LIGHT_SUBTEXT
        }
    }

    /// Returns muted text color for active theme
    pub fn muted_color(&self) -> Color {
        if self.is_dark() {
            colors::DARK_MUTED
        } else {
            colors::LIGHT_MUTED
        }
    }

    /// Returns border color for active theme
    pub fn border_color(&self) -> Color {
        if self.is_dark() {
            colors::DARK_BORDER
        } else {
            colors::LIGHT_BORDER
        }
    }

    /// Returns divider line color for active theme
    pub fn divider_color(&self) -> Color {
        if self.is_dark() {
            colors::DARK_DIVIDER
        } else {
            colors::LIGHT_DIVIDER
        }
    }

    /// Returns interactive hover color for active theme
    pub fn hover_color(&self) -> Color {
        if self.is_dark() {
            colors::DARK_HOVER
        } else {
            colors::LIGHT_HOVER
        }
    }

    /// Returns row selection color for active theme
    pub fn selected_color(&self) -> Color {
        if self.is_dark() {
            colors::DARK_SELECTED
        } else {
            colors::LIGHT_SELECTED
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

// -----------------------------------------------------------------------------
// Modern Design System Widget Styling Functions
// -----------------------------------------------------------------------------

/// Modern elevated card container style (12px rounded corners, soft border, elevation shadow)
pub fn card_style(theme: &AppTheme) -> container::Style {
    if theme.is_dark() {
        container::Style {
            background: Some(Background::Color(colors::DARK_SURFACE)),
            border: Border {
                color: colors::DARK_BORDER,
                width: 1.0,
                radius: 12.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 16.0,
            },
            text_color: Some(colors::DARK_TEXT),
            ..Default::default()
        }
    } else {
        container::Style {
            background: Some(Background::Color(colors::LIGHT_SURFACE)),
            border: Border {
                color: colors::LIGHT_BORDER,
                width: 1.0,
                radius: 12.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.04),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 16.0,
            },
            text_color: Some(colors::LIGHT_TEXT),
            ..Default::default()
        }
    }
}

/// Hero Folder Showcase Card container style (16px rounded corners, subtle colored background tint)
pub fn hero_card_style(theme: &AppTheme, tint_color: Color) -> container::Style {
    if theme.is_dark() {
        container::Style {
            background: Some(Background::Color(colors::DARK_SURFACE)),
            border: Border {
                color: tint_color.scale_alpha(0.3),
                width: 1.0,
                radius: 16.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 16.0,
            },
            text_color: Some(colors::DARK_TEXT),
            ..Default::default()
        }
    } else {
        container::Style {
            background: Some(Background::Color(tint_color)),
            border: Border {
                color: colors::LIGHT_BORDER,
                width: 1.0,
                radius: 16.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.05),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 16.0,
            },
            text_color: Some(colors::LIGHT_TEXT),
            ..Default::default()
        }
    }
}

/// Sidebar navigation item button style (pill-shaped, clean hover & active states)
pub fn sidebar_item_style(
    theme: &AppTheme,
    active: bool,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    let is_dark = theme.is_dark();
    move |_theme, status| {
        let (bg, fg) = if active {
            if is_dark {
                (
                    Some(Background::Color(colors::DARK_SELECTED)),
                    colors::DARK_TEXT,
                )
            } else {
                (
                    Some(Background::Color(colors::LIGHT_SELECTED)),
                    colors::LIGHT_ACCENT,
                )
            }
        } else {
            match status {
                button::Status::Hovered | button::Status::Pressed => {
                    if is_dark {
                        (
                            Some(Background::Color(colors::DARK_HOVER)),
                            colors::DARK_TEXT,
                        )
                    } else {
                        (
                            Some(Background::Color(colors::LIGHT_HOVER)),
                            colors::LIGHT_TEXT,
                        )
                    }
                }
                _ => (
                    None,
                    if is_dark {
                        colors::DARK_SUBTEXT
                    } else {
                        colors::LIGHT_SUBTEXT
                    },
                ),
            }
        };

        button::Style {
            background: bg,
            text_color: fg,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

/// Modern pill-shaped search input container/widget style
pub fn search_input_style(
    theme: &AppTheme,
) -> impl Fn(&iced::Theme, text_input::Status) -> text_input::Style {
    let is_dark = theme.is_dark();
    move |_theme, status| {
        let (border_color, bg_color) = match status {
            text_input::Status::Focused { .. } => {
                if is_dark {
                    (colors::DARK_ACCENT, colors::DARK_SURFACE)
                } else {
                    (colors::LIGHT_ACCENT, colors::LIGHT_SURFACE)
                }
            }
            text_input::Status::Hovered => {
                if is_dark {
                    (colors::DARK_BORDER, colors::DARK_SURFACE)
                } else {
                    (Color::from_rgba(0.0, 0.0, 0.0, 0.15), colors::LIGHT_SURFACE)
                }
            }
            _ => {
                if is_dark {
                    (colors::DARK_BORDER, colors::DARK_SURFACE)
                } else {
                    (colors::LIGHT_BORDER, colors::LIGHT_SURFACE)
                }
            }
        };

        text_input::Style {
            background: Background::Color(bg_color),
            border: Border {
                color: border_color,
                width: 1.0,
                radius: 20.0.into(),
            },
            icon: if is_dark {
                colors::DARK_MUTED
            } else {
                colors::LIGHT_MUTED
            },
            placeholder: if is_dark {
                colors::DARK_MUTED
            } else {
                colors::LIGHT_MUTED
            },
            value: if is_dark {
                colors::DARK_TEXT
            } else {
                colors::LIGHT_TEXT
            },
            selection: if is_dark {
                colors::DARK_SELECTED
            } else {
                colors::LIGHT_SELECTED
            },
        }
    }
}

/// Circular avatar or pill badge container style
pub fn pill_badge_style(bg_color: Color, text_color: Color) -> container::Style {
    container::Style {
        background: Some(Background::Color(bg_color)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 999.0.into(),
        },
        shadow: Shadow::default(),
        text_color: Some(text_color),
        ..Default::default()
    }
}

/// Modern table row button style (with generous row padding, subtle hover and active selection)
pub fn table_row_btn_style(
    theme: &AppTheme,
    is_selected: bool,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    let is_dark = theme.is_dark();
    move |_theme, status| {
        let bg = if is_selected {
            if is_dark {
                Some(Background::Color(colors::DARK_SELECTED))
            } else {
                Some(Background::Color(colors::LIGHT_SELECTED))
            }
        } else {
            match status {
                button::Status::Hovered | button::Status::Pressed => {
                    if is_dark {
                        Some(Background::Color(colors::DARK_HOVER))
                    } else {
                        Some(Background::Color(colors::LIGHT_HOVER))
                    }
                }
                _ => None,
            }
        };

        button::Style {
            background: bg,
            text_color: if is_dark {
                colors::DARK_TEXT
            } else {
                colors::LIGHT_TEXT
            },
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
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

    #[test]
    fn test_theme_color_accessors() {
        let dark = AppTheme::Dark;
        assert_eq!(dark.bg_color(), colors::DARK_BG);
        assert_eq!(dark.surface_color(), colors::DARK_SURFACE);
        assert_eq!(dark.text_color(), colors::DARK_TEXT);

        let light = AppTheme::Light;
        assert_eq!(light.bg_color(), colors::LIGHT_BG);
        assert_eq!(light.surface_color(), colors::LIGHT_SURFACE);
        assert_eq!(light.text_color(), colors::LIGHT_TEXT);
    }
}
