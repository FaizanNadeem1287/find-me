//! Application state and model definitions

use crate::advanced::AdvancedFilterState;
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
    /// Monotonically increasing generation counter to invalidate stale debounced queries
    pub search_generation: u64,
    /// Current vertical scroll position in results table (pixels)
    pub scroll_y: f32,
    /// Active viewport height of the results table (pixels)
    pub table_viewport_height: f32,
    /// Active state of visual query builder in Advanced Search screen
    pub advanced_filters: AdvancedFilterState,
    /// State of index management and custom exclusion rules
    pub index_mgmt: crate::index_mgmt::IndexManagementState,
    /// Persistent user settings & preferences
    pub settings: crate::settings::AppSettings,
    /// Search query history log & starred bookmarks
    pub history: crate::history::SearchHistoryState,
    /// Active state of right-click file context menu
    pub context_menu: ContextMenuState,
    /// Active file properties dialog modal (if open)
    pub properties_modal: Option<crate::desktop::FileProperties>,
    /// Set of expanded folder paths in the sidebar directory tree
    pub expanded_folders: std::collections::HashSet<String>,
    /// Active directory or folder filter applied from sidebar/hero cards
    pub active_folder_filter: Option<String>,
}

/// State of the right-click file context menu
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ContextMenuState {
    pub is_open: bool,
    pub target_item: Option<SearchResultItem>,
    pub position: (f32, f32),
}

/// Constant ID string for the main search input field
pub const SEARCH_INPUT_ID: &str = "main_search_input";

/// Constant ID string for the results table scrollable container
pub const TABLE_SCROLLABLE_ID: &str = "results_table_scrollable";

/// Returns the widget ID for the main search text input
pub fn search_input_id() -> iced::widget::Id {
    iced::widget::Id::new(SEARCH_INPUT_ID)
}

/// Returns the widget ID for the results table scrollable container
pub fn table_scrollable_id() -> iced::widget::Id {
    iced::widget::Id::new(TABLE_SCROLLABLE_ID)
}

/// Computes the visible window (start_index, end_index, top_spacer_px, bottom_spacer_px)
/// for high-performance virtualized tabular rendering.
pub fn compute_virtual_range(
    total_rows: usize,
    scroll_y: f32,
    viewport_height: f32,
    row_height: f32,
    overscan: usize,
) -> (usize, usize, f32, f32) {
    if total_rows == 0 || row_height <= 0.0 {
        return (0, 0, 0.0, 0.0);
    }

    let y = scroll_y.max(0.0);
    let vh = viewport_height.max(100.0);

    let raw_start = (y / row_height).floor() as usize;
    let start = raw_start.saturating_sub(overscan).min(total_rows);

    let visible_count = (vh / row_height).ceil() as usize + (overscan * 2);
    let end = (start + visible_count).min(total_rows);

    let top_spacer = start as f32 * row_height;
    let bottom_spacer = (total_rows.saturating_sub(end)) as f32 * row_height;

    (start, end, top_spacer, bottom_spacer)
}

/// Determines whether a search query is "heavy" and requires debouncing
pub fn is_heavy_query(query: &str) -> bool {
    let q = query.trim();
    if q.is_empty() {
        return false;
    }

    // Regex mode queries are computationally intensive
    if q.contains("regex:") || q.starts_with('/') {
        return true;
    }

    // Heavy wildcards requiring combinatorial backtracking
    if q.contains('*') || q.contains('?') {
        return true;
    }

    // Structured metadata filter prefixes
    if q.contains("size:")
        || q.contains("dm:")
        || q.contains("attrib:")
        || q.contains("path:")
        || q.contains("ext:")
    {
        return true;
    }

    // Complex boolean grouping expressions
    if q.contains('|') || q.contains('!') || q.contains('(') || q.contains(')') {
        return true;
    }

    // Long query strings with more than 3 distinct whitespace terms
    if q.split_whitespace().count() > 3 {
        return true;
    }

    false
}

/// Formats integer with comma separators for clean UI presentation (e.g. 1,234,567)
pub fn format_number(n: u64) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len + len / 3);
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(b as char);
    }
    result
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
            search_generation: 0,
            scroll_y: 0.0,
            table_viewport_height: 600.0,
            advanced_filters: AdvancedFilterState::default(),
            index_mgmt: {
                let s = crate::settings::AppSettings::load_from_path(
                    &crate::settings::AppSettings::config_path(),
                )
                .unwrap_or_default();
                let mut im = crate::index_mgmt::IndexManagementState::default();
                im.sync_from_settings(&s);
                im
            },
            settings: crate::settings::AppSettings::load_from_path(
                &crate::settings::AppSettings::config_path(),
            )
            .unwrap_or_default(),
            history: crate::history::SearchHistoryState::with_default_history(),
            context_menu: ContextMenuState::default(),
            properties_modal: None,
            expanded_folders: {
                let mut set = std::collections::HashSet::new();
                set.insert("General Knowledge".to_string());
                set
            },
            active_folder_filter: None,
        }
    }
}
