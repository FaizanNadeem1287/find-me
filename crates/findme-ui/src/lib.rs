//! Find-Me Desktop User Interface (Pure Rust / Iced)
//!
//! Provides the Elm-architecture model, views, navigation, themes,
//! and virtualized table components for the Find-Me Linux application.

pub mod advanced;
pub mod app;
pub mod message;
pub mod state;
pub mod theme;
pub mod views;

pub use advanced::{AdvancedFilterState, DatePreset, PresetFilter, SizeUnit};
pub use app::{window_settings, FindMeApp};
pub use message::{AdvancedSearchMessage, ColumnId, Message, ScreenId};
pub use state::{
    compute_virtual_range, format_number, is_heavy_query, search_input_id, table_scrollable_id,
    AppState, SEARCH_INPUT_ID, TABLE_SCROLLABLE_ID,
};
pub use theme::{
    colors, contrast_ratio, custom_dark_theme, custom_light_theme, detect_system_theme, AppTheme,
    ThemePreference,
};
pub use views::table::{item_type_str, HEADER_HEIGHT, OVERSCAN_ROWS, ROW_HEIGHT};
