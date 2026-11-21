//! UI Message enum representing all user and background events

use crate::advanced::{DatePreset, PresetFilter, SizeUnit};
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

impl ScreenId {
    /// Returns the user-facing display title for the screen
    pub fn title(&self) -> &'static str {
        match self {
            ScreenId::Search => "Search",
            ScreenId::AdvancedSearch => "Advanced Search",
            ScreenId::IndexManagement => "Index Management",
            ScreenId::SearchHistory => "Search History",
            ScreenId::Settings => "Settings",
            ScreenId::About => "About",
        }
    }

    /// Returns the icon symbol for the screen
    pub fn icon(&self) -> &'static str {
        match self {
            ScreenId::Search => "🔍",
            ScreenId::AdvancedSearch => "⚡",
            ScreenId::IndexManagement => "📁",
            ScreenId::SearchHistory => "🕒",
            ScreenId::Settings => "⚙️",
            ScreenId::About => "ℹ️",
        }
    }

    /// All available navigation screens in order
    pub const ALL: [ScreenId; 6] = [
        ScreenId::Search,
        ScreenId::AdvancedSearch,
        ScreenId::IndexManagement,
        ScreenId::SearchHistory,
        ScreenId::Settings,
        ScreenId::About,
    ];
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
    /// Focus the primary search input
    FocusSearchInput,
    /// Debounced search trigger with generation counter
    DebouncedSearch { query: String, generation: u64 },
    /// Execute search query immediately
    ExecuteSearch(String),
    /// Global keyboard event
    KeyboardEvent(iced::keyboard::Event),
    /// User clicked sidebar navigation item
    SwitchScreen(ScreenId),
    /// User toggled sidebar expand/collapse
    ToggleSidebar,
    /// Window was resized
    WindowResized { width: f32, height: f32 },
    /// Set a status toast / notification banner
    ShowNotification(String),
    /// Dismiss the active notification banner
    DismissNotification,
    /// Run a search query (e.g. from history or advanced search)
    RunQuery(String),
    /// User clicked column header to sort
    SortByColumn(ColumnId),
    /// User selected a row in the results table
    SelectRow(Option<usize>),
    /// Results table scrolled with vertical offset and viewport height
    TableScrolled { scroll_y: f32, viewport_height: f32 },
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
    /// Interaction inside the Advanced Search screen
    AdvancedSearch(AdvancedSearchMessage),
    /// Interaction inside the Index Management screen
    IndexManagement(IndexManagementMessage),
    /// Interaction inside the Search History screen
    History(HistoryMessage),
    /// Interaction inside the Settings screen
    Settings(SettingsMessage),
    /// Background engine event
    Engine(crate::engine::EngineEvent),
}

/// Messages emitted by controls inside the Search History screen
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryMessage {
    ExecuteQuery(String),
    ToggleBookmark(u64),
    RemoveEntry(u64),
    ClearHistory,
    ToggleFilterBookmarked(bool),
    UpdateSearchFilter(String),
}

/// Messages emitted by controls inside the Settings screen
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsMessage {
    ToggleAutostart(bool),
    ToggleCloseToTray(bool),
    ToggleDefaultRegex(bool),
    ToggleDefaultCase(bool),
    SetMaxResults(usize),
    ToggleExcludeHidden(bool),
    ToggleExcludeSystem(bool),
    ResetToDefaults,
    SaveSettings,
}

/// Messages emitted by controls inside the Index Management screen
#[derive(Debug, Clone, PartialEq)]
pub enum IndexManagementMessage {
    UpdateNewRootInput(String),
    AddRootFromInput,
    AddRoot(std::path::PathBuf),
    RemoveRoot(std::path::PathBuf),
    UpdateNewExclusionInput(String),
    AddExclusionFromInput,
    AddExclusion(String),
    RemoveExclusion(String),
    AddDefaultExclusions,
    TriggerRebuildIndex,
    TriggerRefreshIndex,
    ClearIndex,
    UpdateProgress {
        current_path: String,
        files_found: u64,
        percent: f32,
    },
    RebuildFinished(crate::index_mgmt::IndexStats),
}

/// Messages emitted by form controls inside the Advanced Search screen
#[derive(Debug, Clone)]
pub enum AdvancedSearchMessage {
    UpdateAllWords(String),
    UpdateExactPhrase(String),
    UpdateAnyWords(String),
    UpdateNoneWords(String),
    ToggleExtDocuments(bool),
    ToggleExtCode(bool),
    ToggleExtImages(bool),
    ToggleExtAudio(bool),
    ToggleExtVideo(bool),
    ToggleExtArchives(bool),
    UpdateCustomExt(String),
    UpdateSizeMin(String),
    SelectSizeMinUnit(SizeUnit),
    UpdateSizeMax(String),
    SelectSizeMaxUnit(SizeUnit),
    SelectDatePreset(DatePreset),
    UpdateSearchPath(String),
    UpdateExcludePath(String),
    ToggleMatchCase(bool),
    ToggleMatchWholeWord(bool),
    ToggleRegexMode(bool),
    ApplyPreset(PresetFilter),
    ResetFilters,
    SubmitSearch,
}
