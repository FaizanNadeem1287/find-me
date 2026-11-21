//! Custom themes and color palettes for Find-Me (Dark and Light)

/// Application Theme identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppTheme {
    #[default]
    Dark,
    Light,
}

impl AppTheme {
    pub fn is_dark(&self) -> bool {
        matches!(self, AppTheme::Dark)
    }

    pub fn toggle(&self) -> Self {
        match self {
            AppTheme::Dark => AppTheme::Light,
            AppTheme::Light => AppTheme::Dark,
        }
    }
}

/// Palette color constants conforming to the minimal Linux desktop design system
pub mod palette {
    pub const DARK_BG: &str = "#181825";
    pub const DARK_SURFACE: &str = "#1E1E2E";
    pub const DARK_TEXT: &str = "#CDD6F4";
    pub const DARK_SUBTEXT: &str = "#A6ADC8";
    pub const DARK_BORDER: &str = "#313244";
    pub const DARK_ACCENT: &str = "#89B4FA";

    pub const LIGHT_BG: &str = "#EFF1F5";
    pub const LIGHT_SURFACE: &str = "#E6E9EF";
    pub const LIGHT_TEXT: &str = "#4C4F69";
    pub const LIGHT_SUBTEXT: &str = "#6C6F85";
    pub const LIGHT_BORDER: &str = "#CCD0DA";
    pub const LIGHT_ACCENT: &str = "#1E66F5";
}
