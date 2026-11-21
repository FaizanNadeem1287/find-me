//! UI Message enum representing all user and background events

use findme_core::SearchResultItem;

/// Screen identifier for sidebar navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScreenId {
    #[default]
    Search,
    AdvancedSearch,
    IndexManagement,
    SearchHistory,
    Settings,
    About,
}

/// Column identifier for sorting the search results table
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColumnId {
    #[default]
    Name,
    Path,
    Type,
    Size,
    Modified,
}

/// All user interactions and asynchronous events
#[derive(Debug, Clone)]
pub enum Message {
    /// User typed into the main search input
    SearchInputChanged(String),
    /// User clicked "Clear" or pressed Escape
    ClearSearch,
    /// User clicked sidebar navigation item
    SwitchScreen(ScreenId),
    /// User clicked column header to sort
    SortByColumn(ColumnId),
    /// User selected a row in the results table
    SelectRow(Option<usize>),
    /// User requested to open selected file
    OpenFile(String),
    /// User requested to open enclosing folder in file manager
    OpenContainingFolder(String),
    /// User requested to copy file path to clipboard
    CopyPath(String),
    /// Background engine sent search results
    ResultsReceived {
        results: Vec<SearchResultItem>,
        search_duration_ms: u64,
        total_matches: usize,
    },
    /// Background engine sent indexing progress
    IndexingProgress {
        indexed_files: usize,
        indexed_folders: usize,
    },
    /// User toggled theme (Light / Dark)
    ToggleTheme,
}
