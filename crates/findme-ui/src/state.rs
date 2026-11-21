//! Application state and model definitions

use crate::message::{ColumnId, ScreenId};
use crate::theme::{AppTheme, ThemePreference};
use findme_core::SearchResultItem;

/// Central state model for Find-Me desktop UI
#[derive(Debug, Clone)]
pub struct AppState {
    /// Active screen view
    pub active_screen: ScreenId,
    /// Whether sidebar is expanded (true = 220px, false = 68px compact)
    pub sidebar_expanded: bool,
    /// Current window width in pixels
    pub window_width: f32,
    /// Current window height in pixels
    pub window_height: f32,
    /// Active notification toast / banner text
    pub notification: Option<String>,
    /// Current search query in input box
    pub query: String,
    /// Live search results
    pub results: Vec<SearchResultItem>,
    /// Selected row index in the results table
    pub selected_row: Option<usize>,
    /// Active sort column
    pub sort_column: ColumnId,
    /// Sort direction (ascending if true, descending if false)
    pub sort_ascending: bool,
    /// User's persistent theme preference (System, Dark, Light)
    pub theme_pref: ThemePreference,
    /// Active theme (resolved from preference or system)
    pub theme: AppTheme,
    /// Total files indexed by background engine
    pub indexed_files_count: usize,
    /// Total folders indexed by background engine
    pub indexed_folders_count: usize,
    /// Duration of last search query in milliseconds
    pub last_search_duration_ms: u64,
    /// Whether a search is actively processing
    pub is_searching: bool,
}

impl Default for AppState {
    fn default() -> Self {
        let theme_pref =
            ThemePreference::load_from_path(&ThemePreference::config_path()).unwrap_or_default();
        let theme = theme_pref.resolve();

        Self {
            active_screen: ScreenId::Search,
            sidebar_expanded: true,
            window_width: 1200.0,
            window_height: 800.0,
            notification: None,
            query: String::new(),
            results: Vec::new(),
            selected_row: None,
            sort_column: ColumnId::Name,
            sort_ascending: true,
            theme_pref,
            theme,
            indexed_files_count: 0,
            indexed_folders_count: 0,
            last_search_duration_ms: 0,
            is_searching: false,
        }
    }
}
