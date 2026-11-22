//! Find-Me Desktop User Interface (Pure Rust / Iced)
//!
//! Provides the Elm-architecture model, views, navigation, themes,
//! and virtualized table components for the Find-Me Linux application.

pub mod advanced;
pub mod app;
pub mod desktop;
pub mod engine;
pub mod history;
pub mod index_mgmt;
pub mod message;
pub mod settings;
pub mod state;
pub mod theme;
pub mod views;

pub use advanced::{AdvancedFilterState, DatePreset, PresetFilter, SizeUnit};
pub use app::{window_settings, FindMeApp};
pub use desktop::{
    copy_file_uri, copy_to_clipboard, get_file_properties, show_in_file_manager, FileProperties,
};
pub use engine::{engine_subscription, EngineBridge, EngineEvent};
pub use history::{HistoryEntry, SearchHistoryState};
pub use index_mgmt::{
    ExcludedPattern, IndexManagementState, IndexStats, IndexedRoot, IndexingState, RootStatus,
};
pub use message::{
    AdvancedSearchMessage, ColumnId, HistoryMessage, IndexManagementMessage, Message, ScreenId,
    SettingsMessage,
};
pub use settings::AppSettings;
pub use state::{
    compute_virtual_range, format_number, is_heavy_query, search_input_id, table_scrollable_id,
    AppState, SEARCH_INPUT_ID, TABLE_SCROLLABLE_ID,
};
pub use theme::{
    card_style, chip_btn_style, colors, contrast_ratio, custom_dark_theme, custom_light_theme,
    danger_btn_style, detect_system_theme, hero_card_style, kbd_badge_style, pill_badge_style,
    primary_btn_style, search_input_style, secondary_btn_style, sidebar_item_style,
    subtle_card_style, table_row_btn_style, AppTheme, ThemePreference,
};
pub use views::table::{item_type_str, HEADER_HEIGHT, OVERSCAN_ROWS, ROW_HEIGHT};
