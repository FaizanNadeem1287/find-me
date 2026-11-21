//! Top-level FindMeApp Elm-architecture loop

use crate::engine::{engine_subscription, EngineBridge, EngineEvent};
use crate::message::{
    AdvancedSearchMessage, ColumnId, HistoryMessage, IndexManagementMessage, Message, ScreenId,
    SettingsMessage,
};
use crate::state::{is_heavy_query, search_input_id, AppState};
use crate::views;
use iced::widget::{column, operation, row};
use iced::{Element, Length, Size, Subscription, Task, Theme};
use std::sync::Arc;

/// Configures window settings conforming to the Find-Me desktop specification
///
/// Sets minimum dimensions of 800x600 and a comfortable default launch size of 1200x800.
pub fn window_settings() -> iced::window::Settings {
    iced::window::Settings {
        size: Size::new(1200.0, 800.0),
        min_size: Some(Size::new(800.0, 600.0)),
        resizable: true,
        decorations: true,
        ..Default::default()
    }
}

/// Main application controller
#[derive(Debug)]
pub struct FindMeApp {
    pub state: AppState,
    pub engine: Arc<EngineBridge>,
}

impl Default for FindMeApp {
    fn default() -> Self {
        Self::new()
    }
}

impl FindMeApp {
    /// Creates a new default instance of FindMeApp with engine bridge
    pub fn new() -> Self {
        Self::with_engine(Arc::new(EngineBridge::new()))
    }

    /// Creates a FindMeApp instance with an existing engine bridge
    pub fn with_engine(engine: Arc<EngineBridge>) -> Self {
        let (initial_files, initial_folders, memory_bytes) = {
            let arena = engine.arena();
            let lock = arena.read();
            (
                lock.len(),
                lock.dirs().len(),
                lock.memory_usage_bytes() as u64,
            )
        };

        let mut index_mgmt = crate::index_mgmt::IndexManagementState::default();
        index_mgmt.stats.total_files = initial_files as u64;
        index_mgmt.stats.total_folders = initial_folders as u64;
        index_mgmt.stats.memory_bytes = memory_bytes;

        let state = AppState {
            indexed_files_count: initial_files,
            indexed_folders_count: initial_folders,
            index_mgmt,
            ..Default::default()
        };

        Self { state, engine }
    }

    /// Boots the application and initializes primary focus on the search input
    pub fn boot() -> (Self, Task<Message>) {
        let app = Self::new();
        let focus_task = operation::focus(search_input_id());
        (app, focus_task)
    }

    /// Application window title
    pub fn title(&self) -> String {
        format!("Find-Me — {}", self.state.active_screen.title())
    }

    /// Handles messages and performs state transitions
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SearchInputChanged(query) => {
                self.state.query = query;
                self.state.search_generation = self.state.search_generation.wrapping_add(1);
                let gen = self.state.search_generation;

                if self.state.query.is_empty() {
                    self.state.results.clear();
                    self.state.is_searching = false;
                    self.state.selected_row = None;
                    Task::none()
                } else {
                    self.state.is_searching = true;
                    let q = self.state.query.clone();
                    if is_heavy_query(&q) {
                        Task::future(async move {
                            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                            Message::DebouncedSearch {
                                query: q,
                                generation: gen,
                            }
                        })
                    } else {
                        Task::done(Message::DebouncedSearch {
                            query: q,
                            generation: gen,
                        })
                    }
                }
            }
            Message::ClearSearch => {
                self.state.query.clear();
                self.state.results.clear();
                self.state.is_searching = false;
                self.state.selected_row = None;
                self.state.scroll_y = 0.0;
                operation::focus(search_input_id())
            }
            Message::FocusSearchInput => operation::focus(search_input_id()),
            Message::DebouncedSearch { query, generation } => {
                if generation == self.state.search_generation {
                    Task::done(Message::ExecuteSearch(query))
                } else {
                    Task::none()
                }
            }
            Message::ExecuteSearch(query) => {
                self.state.is_searching = true;
                let engine = Arc::clone(&self.engine);
                let max_results = self.state.settings.max_results;
                let sort_col = self.state.sort_column;
                let sort_asc = self.state.sort_ascending;
                Task::future(async move {
                    let (results, duration_ms, total_matches) =
                        engine.search(&query, max_results, sort_col, sort_asc).await;
                    Message::ResultsReceived {
                        results,
                        search_duration_ms: duration_ms,
                        total_matches,
                    }
                })
            }
            Message::KeyboardEvent(event) => {
                if let iced::keyboard::Event::KeyPressed { key, modifiers, .. } = event {
                    if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) {
                        if !self.state.query.is_empty() {
                            self.state.query.clear();
                            self.state.results.clear();
                            self.state.is_searching = false;
                            self.state.selected_row = None;
                            self.state.scroll_y = 0.0;
                            return operation::focus(search_input_id());
                        } else if self.state.notification.is_some() {
                            self.state.notification = None;
                        }
                    } else if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowUp)
                    {
                        if !self.state.results.is_empty() {
                            let current = self.state.selected_row.unwrap_or(0);
                            let next = current.saturating_sub(1);
                            self.state.selected_row = Some(next);
                            return self.scroll_to_selected_row(next);
                        }
                    } else if key
                        == iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowDown)
                    {
                        if !self.state.results.is_empty() {
                            let max_idx = self.state.results.len().saturating_sub(1);
                            let next = match self.state.selected_row {
                                None => 0,
                                Some(curr) => (curr + 1).min(max_idx),
                            };
                            self.state.selected_row = Some(next);
                            return self.scroll_to_selected_row(next);
                        }
                    } else if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::PageUp)
                    {
                        if !self.state.results.is_empty() {
                            let current = self.state.selected_row.unwrap_or(0);
                            let next = current.saturating_sub(15);
                            self.state.selected_row = Some(next);
                            return self.scroll_to_selected_row(next);
                        }
                    } else if key
                        == iced::keyboard::Key::Named(iced::keyboard::key::Named::PageDown)
                    {
                        if !self.state.results.is_empty() {
                            let max_idx = self.state.results.len().saturating_sub(1);
                            let current = self.state.selected_row.unwrap_or(0);
                            let next = (current + 15).min(max_idx);
                            self.state.selected_row = Some(next);
                            return self.scroll_to_selected_row(next);
                        }
                    } else if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Home) {
                        if !self.state.results.is_empty() {
                            self.state.selected_row = Some(0);
                            return self.scroll_to_selected_row(0);
                        }
                    } else if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::End) {
                        if !self.state.results.is_empty() {
                            let last = self.state.results.len().saturating_sub(1);
                            self.state.selected_row = Some(last);
                            return self.scroll_to_selected_row(last);
                        }
                    } else if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter) {
                        if let Some(idx) = self.state.selected_row {
                            if let Some(item) = self.state.results.get(idx) {
                                let path = item.path.clone();
                                if modifiers.alt() {
                                    return self.update(Message::OpenContainingFolder(path));
                                } else {
                                    return self.update(Message::OpenFile(path));
                                }
                            }
                        }
                    }

                    let is_ctrl_f = modifiers.control()
                        && matches!(key, iced::keyboard::Key::Character(ref s) if s.eq_ignore_ascii_case("f"));
                    let is_slash = key == iced::keyboard::Key::Character("/".into())
                        && !modifiers.control()
                        && !modifiers.alt()
                        && self.state.active_screen != ScreenId::Search;

                    if is_ctrl_f || is_slash {
                        self.state.active_screen = ScreenId::Search;
                        return operation::focus(search_input_id());
                    }
                }
                Task::none()
            }
            Message::SwitchScreen(screen) => {
                self.state.active_screen = screen;
                Task::none()
            }
            Message::ToggleSidebar => {
                self.state.sidebar_expanded = !self.state.sidebar_expanded;
                Task::none()
            }
            Message::WindowResized { width, height } => {
                self.state.window_width = width;
                self.state.window_height = height;
                Task::none()
            }
            Message::ShowNotification(msg) => {
                self.state.notification = Some(msg);
                Task::none()
            }
            Message::DismissNotification => {
                self.state.notification = None;
                Task::none()
            }
            Message::RunQuery(query) => {
                self.state.query = query.clone();
                self.state.active_screen = ScreenId::Search;
                self.state.is_searching = true;
                self.update(Message::ExecuteSearch(query))
            }
            Message::SortByColumn(col) => {
                if self.state.sort_column == col {
                    self.state.sort_ascending = !self.state.sort_ascending;
                } else {
                    self.state.sort_column = col;
                    self.state.sort_ascending = true;
                }
                self.sort_results();
                Task::none()
            }
            Message::SelectRow(row_idx) => {
                self.state.selected_row = row_idx;
                Task::none()
            }
            Message::OpenFile(path) => {
                let _ = open::that_detached(path);
                Task::none()
            }
            Message::OpenContainingFolder(path) => {
                if let Some(parent) = std::path::Path::new(&path).parent() {
                    let _ = open::that_detached(parent);
                }
                Task::none()
            }
            Message::CopyPath(path) => {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(path);
                }
                Task::none()
            }
            Message::ResultsReceived {
                results,
                search_duration_ms,
                total_matches,
            } => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                self.state.history.record_query(
                    &self.state.query,
                    total_matches,
                    search_duration_ms,
                    now,
                );
                self.state.results = results;
                self.state.last_search_duration_ms = search_duration_ms;
                self.state.is_searching = false;
                self.state.scroll_y = 0.0;
                self.state.selected_row = if self.state.results.is_empty() {
                    None
                } else {
                    Some(0)
                };
                self.sort_results();
                Task::none()
            }
            Message::TableScrolled {
                scroll_y,
                viewport_height,
            } => {
                self.state.scroll_y = scroll_y;
                self.state.table_viewport_height = viewport_height;
                Task::none()
            }
            Message::IndexingProgress {
                indexed_files,
                indexed_folders,
            } => {
                self.state.indexed_files_count = indexed_files;
                self.state.indexed_folders_count = indexed_folders;
                Task::none()
            }
            Message::ToggleTheme => {
                self.state.theme = self.state.theme.toggle();
                self.state.theme_pref = match self.state.theme {
                    crate::theme::AppTheme::Dark => crate::theme::ThemePreference::Dark,
                    crate::theme::AppTheme::Light => crate::theme::ThemePreference::Light,
                };
                let _ = self
                    .state
                    .theme_pref
                    .save_to_path(&crate::theme::ThemePreference::config_path());
                Task::none()
            }
            Message::AdvancedSearch(adv_msg) => {
                match adv_msg {
                    AdvancedSearchMessage::UpdateAllWords(w) => {
                        self.state.advanced_filters.all_words = w;
                    }
                    AdvancedSearchMessage::UpdateExactPhrase(w) => {
                        self.state.advanced_filters.exact_phrase = w;
                    }
                    AdvancedSearchMessage::UpdateAnyWords(w) => {
                        self.state.advanced_filters.any_words = w;
                    }
                    AdvancedSearchMessage::UpdateNoneWords(w) => {
                        self.state.advanced_filters.none_words = w;
                    }
                    AdvancedSearchMessage::ToggleExtDocuments(b) => {
                        self.state.advanced_filters.ext_documents = b;
                    }
                    AdvancedSearchMessage::ToggleExtCode(b) => {
                        self.state.advanced_filters.ext_code = b;
                    }
                    AdvancedSearchMessage::ToggleExtImages(b) => {
                        self.state.advanced_filters.ext_images = b;
                    }
                    AdvancedSearchMessage::ToggleExtAudio(b) => {
                        self.state.advanced_filters.ext_audio = b;
                    }
                    AdvancedSearchMessage::ToggleExtVideo(b) => {
                        self.state.advanced_filters.ext_video = b;
                    }
                    AdvancedSearchMessage::ToggleExtArchives(b) => {
                        self.state.advanced_filters.ext_archives = b;
                    }
                    AdvancedSearchMessage::UpdateCustomExt(w) => {
                        self.state.advanced_filters.custom_ext = w;
                    }
                    AdvancedSearchMessage::UpdateSizeMin(s) => {
                        self.state.advanced_filters.size_min = s;
                    }
                    AdvancedSearchMessage::SelectSizeMinUnit(u) => {
                        self.state.advanced_filters.size_min_unit = u;
                    }
                    AdvancedSearchMessage::UpdateSizeMax(s) => {
                        self.state.advanced_filters.size_max = s;
                    }
                    AdvancedSearchMessage::SelectSizeMaxUnit(u) => {
                        self.state.advanced_filters.size_max_unit = u;
                    }
                    AdvancedSearchMessage::SelectDatePreset(p) => {
                        self.state.advanced_filters.date_preset = p;
                    }
                    AdvancedSearchMessage::UpdateSearchPath(p) => {
                        self.state.advanced_filters.search_path = p;
                    }
                    AdvancedSearchMessage::UpdateExcludePath(p) => {
                        self.state.advanced_filters.exclude_path = p;
                    }
                    AdvancedSearchMessage::ToggleMatchCase(b) => {
                        self.state.advanced_filters.match_case = b;
                    }
                    AdvancedSearchMessage::ToggleMatchWholeWord(b) => {
                        self.state.advanced_filters.match_whole_word = b;
                    }
                    AdvancedSearchMessage::ToggleRegexMode(b) => {
                        self.state.advanced_filters.regex_mode = b;
                    }
                    AdvancedSearchMessage::ApplyPreset(preset) => {
                        self.state.advanced_filters.apply_preset(preset);
                    }
                    AdvancedSearchMessage::ResetFilters => {
                        self.state.advanced_filters =
                            crate::advanced::AdvancedFilterState::default();
                    }
                    AdvancedSearchMessage::SubmitSearch => {
                        let synthesized = self.state.advanced_filters.synthesize_query();
                        return self.update(Message::RunQuery(synthesized));
                    }
                }
                Task::none()
            }
            Message::IndexManagement(idx_msg) => {
                match idx_msg {
                    IndexManagementMessage::UpdateNewRootInput(input) => {
                        self.state.index_mgmt.new_root_input = input;
                    }
                    IndexManagementMessage::AddRootFromInput => {
                        let trimmed = self.state.index_mgmt.new_root_input.trim();
                        if !trimmed.is_empty() {
                            let path = std::path::PathBuf::from(trimmed);
                            if self.state.index_mgmt.add_root(path) {
                                self.state.index_mgmt.new_root_input.clear();
                                self.state.notification =
                                    Some("Added indexed root directory".to_string());
                            } else {
                                self.state.notification =
                                    Some("Directory already in indexed list".to_string());
                            }
                        }
                    }
                    IndexManagementMessage::AddRoot(path) => {
                        self.state.index_mgmt.add_root(path);
                    }
                    IndexManagementMessage::RemoveRoot(path) => {
                        if self.state.index_mgmt.remove_root(&path) {
                            self.state.notification =
                                Some("Removed indexed directory root".to_string());
                        }
                    }
                    IndexManagementMessage::UpdateNewExclusionInput(input) => {
                        self.state.index_mgmt.new_exclusion_input = input;
                    }
                    IndexManagementMessage::AddExclusionFromInput => {
                        let trimmed = self.state.index_mgmt.new_exclusion_input.trim().to_string();
                        if !trimmed.is_empty() {
                            if self.state.index_mgmt.add_exclusion(trimmed) {
                                self.state.index_mgmt.new_exclusion_input.clear();
                                self.state.notification = Some("Added exclusion rule".to_string());
                            } else {
                                self.state.notification =
                                    Some("Exclusion pattern already exists".to_string());
                            }
                        }
                    }
                    IndexManagementMessage::AddExclusion(pat) => {
                        self.state.index_mgmt.add_exclusion(pat);
                    }
                    IndexManagementMessage::RemoveExclusion(pat) => {
                        if self.state.index_mgmt.remove_exclusion(&pat) {
                            self.state.notification = Some("Removed exclusion pattern".to_string());
                        }
                    }
                    IndexManagementMessage::AddDefaultExclusions => {
                        self.state.index_mgmt.add_default_exclusions();
                        self.state.notification = Some("Default exclusion rules added".to_string());
                    }
                    IndexManagementMessage::TriggerRebuildIndex => {
                        self.state.index_mgmt.indexing_state =
                            crate::index_mgmt::IndexingState::Scanning {
                                current_path: "Initializing crawl...".to_string(),
                                files_found: 0,
                                percent: 0.0,
                            };
                        self.state.notification =
                            Some("Starting full index rebuild...".to_string());
                        let roots: Vec<std::path::PathBuf> = self
                            .state
                            .index_mgmt
                            .roots
                            .iter()
                            .map(|r| r.path.clone())
                            .collect();
                        let exclusions: Vec<std::path::PathBuf> = self
                            .state
                            .index_mgmt
                            .exclusions
                            .iter()
                            .filter(|e| e.pattern.starts_with('/'))
                            .map(|e| std::path::PathBuf::from(&e.pattern))
                            .collect();
                        let excluded_names: Vec<String> = self
                            .state
                            .index_mgmt
                            .exclusions
                            .iter()
                            .filter(|e| !e.pattern.starts_with('/'))
                            .map(|e| e.pattern.clone())
                            .collect();
                        self.engine.rebuild_index(roots, exclusions, excluded_names);
                    }
                    IndexManagementMessage::TriggerRefreshIndex => {
                        self.state.notification =
                            Some("Incremental refresh completed (inotify synced)".to_string());
                        let q = self.state.query.clone();
                        return self.update(Message::ExecuteSearch(q));
                    }
                    IndexManagementMessage::ClearIndex => {
                        self.engine.clear_index();
                        self.state.index_mgmt.stats.total_files = 0;
                        self.state.index_mgmt.stats.total_folders = 0;
                        self.state.index_mgmt.stats.memory_bytes = 0;
                        self.state.index_mgmt.indexing_state =
                            crate::index_mgmt::IndexingState::Idle;
                        for root in &mut self.state.index_mgmt.roots {
                            root.file_count = 0;
                            root.folder_count = 0;
                        }
                        self.state.indexed_files_count = 0;
                        self.state.indexed_folders_count = 0;
                        self.state.results.clear();
                        self.state.notification = Some("Index database cleared".to_string());
                    }
                    IndexManagementMessage::UpdateProgress {
                        current_path,
                        files_found,
                        percent,
                    } => {
                        self.state.index_mgmt.indexing_state =
                            crate::index_mgmt::IndexingState::Scanning {
                                current_path,
                                files_found,
                                percent,
                            };
                    }
                    IndexManagementMessage::RebuildFinished(stats) => {
                        self.state.indexed_files_count = stats.total_files as usize;
                        self.state.indexed_folders_count = stats.total_folders as usize;
                        self.state.index_mgmt.stats = stats;
                        self.state.index_mgmt.indexing_state =
                            crate::index_mgmt::IndexingState::Idle;
                        self.state.notification =
                            Some("Index rebuild completed successfully".to_string());
                    }
                }
                Task::none()
            }
            Message::History(hist_msg) => {
                match hist_msg {
                    HistoryMessage::ExecuteQuery(query) => {
                        self.state.query = query.clone();
                        self.state.active_screen = ScreenId::Search;
                        return self.update(Message::RunQuery(query));
                    }
                    HistoryMessage::ToggleBookmark(id) => {
                        self.state.history.toggle_bookmark(id);
                    }
                    HistoryMessage::RemoveEntry(id) => {
                        if self.state.history.remove_entry(id) {
                            self.state.notification =
                                Some("Removed query from history".to_string());
                        }
                    }
                    HistoryMessage::ClearHistory => {
                        self.state.history.clear();
                        self.state.notification = Some("Search history cleared".to_string());
                    }
                    HistoryMessage::ToggleFilterBookmarked(b) => {
                        self.state.history.filter_bookmarked_only = b;
                    }
                    HistoryMessage::UpdateSearchFilter(filter) => {
                        self.state.history.search_filter = filter;
                    }
                }
                Task::none()
            }
            Message::Settings(settings_msg) => {
                match settings_msg {
                    SettingsMessage::ToggleAutostart(b) => {
                        self.state.settings.autostart = b;
                    }
                    SettingsMessage::ToggleCloseToTray(b) => {
                        self.state.settings.close_to_tray = b;
                    }
                    SettingsMessage::ToggleDefaultRegex(b) => {
                        self.state.settings.default_regex = b;
                    }
                    SettingsMessage::ToggleDefaultCase(b) => {
                        self.state.settings.default_case_sensitive = b;
                    }
                    SettingsMessage::SetMaxResults(limit) => {
                        self.state.settings.max_results = limit;
                    }
                    SettingsMessage::ToggleExcludeHidden(b) => {
                        self.state.settings.exclude_hidden_files = b;
                    }
                    SettingsMessage::ToggleExcludeSystem(b) => {
                        self.state.settings.exclude_system_paths = b;
                    }
                    SettingsMessage::ResetToDefaults => {
                        self.state.settings = crate::settings::AppSettings::default();
                        self.state.notification = Some("Settings restored to defaults".to_string());
                    }
                    SettingsMessage::SaveSettings => {
                        let path = crate::settings::AppSettings::config_path();
                        match self.state.settings.save_to_path(&path) {
                            Ok(()) => {
                                self.state.notification =
                                    Some("Preferences saved successfully".to_string());
                            }
                            Err(e) => {
                                self.state.notification =
                                    Some(format!("Error saving preferences: {e}"));
                            }
                        }
                    }
                }
                Task::none()
            }
            Message::Engine(engine_event) => {
                match engine_event {
                    EngineEvent::Started => {
                        self.state.index_mgmt.indexing_state =
                            crate::index_mgmt::IndexingState::Scanning {
                                current_path: "Crawling filesystem roots...".to_string(),
                                files_found: 0,
                                percent: 0.0,
                            };
                    }
                    EngineEvent::Progress {
                        files_found,
                        dirs_scanned,
                        current_path,
                        percent,
                    } => {
                        self.state.indexed_files_count = files_found as usize;
                        self.state.indexed_folders_count = dirs_scanned as usize;
                        self.state.index_mgmt.indexing_state =
                            crate::index_mgmt::IndexingState::Scanning {
                                current_path,
                                files_found,
                                percent,
                            };
                    }
                    EngineEvent::IndexReady {
                        total_files,
                        total_folders,
                        memory_bytes,
                        duration_ms,
                    } => {
                        self.state.indexed_files_count = total_files as usize;
                        self.state.indexed_folders_count = total_folders as usize;
                        self.state.index_mgmt.stats.total_files = total_files;
                        self.state.index_mgmt.stats.total_folders = total_folders;
                        self.state.index_mgmt.stats.memory_bytes = memory_bytes;
                        self.state.index_mgmt.stats.last_scan_secs =
                            Some((duration_ms / 1000).max(1));
                        self.state.index_mgmt.indexing_state =
                            crate::index_mgmt::IndexingState::Idle;
                        self.state.notification = Some(format!(
                            "Index ready: {} files in {} folders ({:.2} MB in {} ms)",
                            crate::state::format_number(total_files),
                            crate::state::format_number(total_folders),
                            (memory_bytes as f64) / (1024.0 * 1024.0),
                            duration_ms
                        ));
                        let q = self.state.query.clone();
                        return self.update(Message::ExecuteSearch(q));
                    }
                    EngineEvent::FsChange { path, kind } => {
                        tracing::debug!("Filesystem change detected: {:?} ({})", path, kind);
                        if self.state.active_screen == ScreenId::Search {
                            let q = self.state.query.clone();
                            return self.update(Message::ExecuteSearch(q));
                        }
                    }
                    EngineEvent::Error(err_msg) => {
                        self.state.index_mgmt.indexing_state =
                            crate::index_mgmt::IndexingState::Idle;
                        self.state.notification = Some(format!("Engine error: {err_msg}"));
                    }
                }
                Task::none()
            }
        }
    }

    /// Automatically scrolls the virtualized table so the selected row is visible
    fn scroll_to_selected_row(&mut self, row_idx: usize) -> Task<Message> {
        let row_top = row_idx as f32 * crate::views::table::ROW_HEIGHT;
        let row_bottom = row_top + crate::views::table::ROW_HEIGHT;
        let view_top = self.state.scroll_y;
        let view_bottom = view_top + self.state.table_viewport_height;

        if row_top < view_top {
            self.state.scroll_y = row_top;
            operation::scroll_to(
                crate::state::table_scrollable_id(),
                operation::AbsoluteOffset {
                    x: None,
                    y: Some(row_top),
                },
            )
        } else if row_bottom > view_bottom {
            let target = (row_bottom - self.state.table_viewport_height).max(0.0);
            self.state.scroll_y = target;
            operation::scroll_to(
                crate::state::table_scrollable_id(),
                operation::AbsoluteOffset {
                    x: None,
                    y: Some(target),
                },
            )
        } else {
            Task::none()
        }
    }

    /// Internal helper to sort current results based on active column and direction
    fn sort_results(&mut self) {
        let asc = self.state.sort_ascending;
        match self.state.sort_column {
            ColumnId::Name => {
                self.state.results.sort_by(|a, b| {
                    if asc {
                        a.name.cmp(&b.name).then_with(|| a.path.cmp(&b.path))
                    } else {
                        b.name.cmp(&a.name).then_with(|| b.path.cmp(&a.path))
                    }
                });
            }
            ColumnId::Path => {
                self.state.results.sort_by(|a, b| {
                    if asc {
                        a.path.cmp(&b.path).then_with(|| a.name.cmp(&b.name))
                    } else {
                        b.path.cmp(&a.path).then_with(|| b.name.cmp(&a.name))
                    }
                });
            }
            ColumnId::Type => {
                self.state.results.sort_by(|a, b| {
                    let type_a = crate::views::table::item_type_str(a);
                    let type_b = crate::views::table::item_type_str(b);
                    if asc {
                        type_a.cmp(type_b).then_with(|| a.name.cmp(&b.name))
                    } else {
                        type_b.cmp(type_a).then_with(|| b.name.cmp(&a.name))
                    }
                });
            }
            ColumnId::Size => {
                self.state.results.sort_by(|a, b| {
                    if asc {
                        a.size_bytes
                            .cmp(&b.size_bytes)
                            .then_with(|| a.name.cmp(&b.name))
                    } else {
                        b.size_bytes
                            .cmp(&a.size_bytes)
                            .then_with(|| b.name.cmp(&a.name))
                    }
                });
            }
            ColumnId::Modified => {
                self.state.results.sort_by(|a, b| {
                    if asc {
                        a.mtime_secs
                            .cmp(&b.mtime_secs)
                            .then_with(|| a.name.cmp(&b.name))
                    } else {
                        b.mtime_secs
                            .cmp(&a.mtime_secs)
                            .then_with(|| b.name.cmp(&a.name))
                    }
                });
            }
        }
    }

    /// Renders the responsive desktop shell with sidebar navigation and active workspace screen
    pub fn view(&self) -> Element<'_, Message> {
        let sidebar = views::sidebar::view(&self.state);

        let screen_content: Element<'_, Message> = match self.state.active_screen {
            ScreenId::Search => views::search::view(&self.state),
            ScreenId::AdvancedSearch => views::advanced::view(&self.state),
            ScreenId::IndexManagement => views::index_mgmt::view(&self.state),
            ScreenId::SearchHistory => views::history::view(&self.state),
            ScreenId::Settings => views::settings::view(&self.state),
            ScreenId::About => views::about::view(&self.state),
        };

        let workspace_content = if let Some(ref notif) = self.state.notification {
            let banner = views::notification_banner(notif);
            column![banner, screen_content]
                .width(Length::Fill)
                .height(Length::Fill)
        } else {
            column![screen_content]
                .width(Length::Fill)
                .height(Length::Fill)
        };

        row![sidebar, workspace_content]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Resolves theme for Iced engine
    pub fn theme(&self) -> Theme {
        self.state.theme.as_iced_theme()
    }

    /// Background event subscriptions (e.g. engine channels, keyboard shortcuts)
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::keyboard::listen().map(Message::KeyboardEvent),
            engine_subscription().map(Message::Engine),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::ScreenId;

    #[test]
    fn test_search_input_state_transition() {
        let mut app = FindMeApp::new();
        assert_eq!(app.state.query, "");

        let _ = app.update(Message::SearchInputChanged("rust_code".into()));
        assert_eq!(app.state.query, "rust_code");
        assert!(app.state.is_searching);

        let _ = app.update(Message::ClearSearch);
        assert_eq!(app.state.query, "");
        assert!(!app.state.is_searching);
    }

    #[test]
    fn test_screen_switch() {
        let mut app = FindMeApp::new();
        assert_eq!(app.state.active_screen, ScreenId::Search);

        let _ = app.update(Message::SwitchScreen(ScreenId::Settings));
        assert_eq!(app.state.active_screen, ScreenId::Settings);
        assert_eq!(app.title(), "Find-Me — Settings");
    }

    #[test]
    fn test_theme_toggle() {
        let mut app = FindMeApp::new();
        let initial_is_dark = app.state.theme.is_dark();

        let _ = app.update(Message::ToggleTheme);
        assert_eq!(app.state.theme.is_dark(), !initial_is_dark);

        let _ = app.update(Message::ToggleTheme);
        assert_eq!(app.state.theme.is_dark(), initial_is_dark);
    }

    #[test]
    fn test_sidebar_toggle() {
        let mut app = FindMeApp::new();
        assert!(app.state.sidebar_expanded);

        let _ = app.update(Message::ToggleSidebar);
        assert!(!app.state.sidebar_expanded);

        let _ = app.update(Message::ToggleSidebar);
        assert!(app.state.sidebar_expanded);
    }

    #[test]
    fn test_window_resized_and_notification() {
        let mut app = FindMeApp::new();
        assert_eq!(app.state.window_width, 1200.0);

        let _ = app.update(Message::WindowResized {
            width: 1400.0,
            height: 900.0,
        });
        assert_eq!(app.state.window_width, 1400.0);
        assert_eq!(app.state.window_height, 900.0);

        let _ = app.update(Message::ShowNotification("Test notification".into()));
        assert_eq!(app.state.notification.as_deref(), Some("Test notification"));

        let _ = app.update(Message::DismissNotification);
        assert_eq!(app.state.notification, None);
    }
}
