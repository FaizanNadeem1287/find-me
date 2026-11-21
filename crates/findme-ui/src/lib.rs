//! Find-Me Desktop User Interface (Pure Rust / Iced)
//!
//! Provides the Elm-architecture model, views, navigation, themes,
//! and virtualized table components for the Find-Me Linux application.

pub mod app;
pub mod message;
pub mod state;
pub mod theme;
pub mod views;

pub use app::{window_settings, FindMeApp};
pub use message::{ColumnId, Message, ScreenId};
pub use state::{format_number, is_heavy_query, search_input_id, AppState, SEARCH_INPUT_ID};
pub use theme::{
    colors, contrast_ratio, custom_dark_theme, custom_light_theme, detect_system_theme, AppTheme,
    ThemePreference,
};
