//! Application state and model definitions

use crate::message::{ColumnId, ScreenId};
use crate::theme::AppTheme;
use findme_core::SearchResultItem;

/// Central state model for Find-Me desktop UI
#[derive(Debug, Clone)]
pub struct AppState {
    /// Active screen view
    pub active_screen: ScreenId,
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
    /// Active theme (Dark by default)
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
        Self {
            active_screen: ScreenId::Search,
            query: String::new(),
            results: Vec::new(),
            selected_row: None,
            sort_column: ColumnId::Name,
            sort_ascending: true,
            theme: AppTheme::Dark,
            indexed_files_count: 0,
            indexed_folders_count: 0,
            last_search_duration_ms: 0,
            is_searching: false,
        }
    }
}
