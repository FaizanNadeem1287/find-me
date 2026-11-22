//! Top-level FindMeApp Elm-architecture loop

use crate::engine::{engine_subscription, EngineBridge, EngineEvent};
use crate::message::{
    AdvancedSearchMessage, ColumnId, HistoryMessage, IndexManagementMessage, Message, ScreenId,
    SettingsMessage,
};
use crate::state::{is_heavy_query, search_input_id, AppState};
use crate::views;
use findme_core::SearchResultItem;
use iced::widget::{column, container, operation, row};
use iced::{Background, Element, Length, Size, Subscription, Task, Theme};
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

        let mut app = Self { state, engine };
        app.refresh_root_counts();
        app
    }

    /// Boots the application and initializes primary focus on the search input
    pub fn boot() -> (Self, Task<Message>) {
        Self::boot_with_query(None)
    }

    /// Boots the application with an optional initial search query
    pub fn boot_with_query(initial_query: Option<String>) -> (Self, Task<Message>) {
        Self::boot_with_engine_and_query(Arc::new(EngineBridge::new()), initial_query)
    }

    /// Boots the application with an existing engine bridge and optional initial search query
    pub fn boot_with_engine_and_query(
        engine: Arc<EngineBridge>,
        initial_query: Option<String>,
    ) -> (Self, Task<Message>) {
        let mut app = Self::with_engine(engine);
        let focus_task = operation::focus(search_input_id());

        if let Some(query) = initial_query {
            let trimmed = query.trim();
            if !trimmed.is_empty() {
                app.state.query = trimmed.to_string();
                app.state.is_searching = true;
                let search_task = Task::done(Message::ExecuteSearch(trimmed.to_string()));
                return (app, Task::batch(vec![focus_task, search_task]));
            }
        }

        (app, focus_task)
    }

    /// Application window title
    pub fn title(&self) -> String {
        format!("Find-Me — {}", self.state.active_screen.title())
    }

    /// Recalculates live file and folder counts for all monitored roots from the active FileArena
    pub fn refresh_root_counts(&mut self) {
        let arena = self.engine.arena();
        let lock = arena.read();
        for root in &mut self.state.index_mgmt.roots {
            let (files, folders) = lock.count_under_path(&root.path);
            root.file_count = files;
            root.folder_count = folders;
        }
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
            Message::FocusSearchInput => {
                self.state.active_screen = ScreenId::Search;
                operation::focus(search_input_id())
            }
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
                let search_mode = self.state.search_mode;

                let is_semantic = search_mode != findme_semantic::SearchMode::Everything
                    || query.starts_with("semantic:")
                    || query.starts_with("sem:")
                    || query.starts_with("content:")
                    || query.starts_with("text:")
                    || query.starts_with("hybrid:");

                if is_semantic {
                    let mode = if search_mode != findme_semantic::SearchMode::Everything {
                        search_mode
                    } else {
                        findme_semantic::SearchMode::Hybrid
                    };
                    Task::future(async move {
                        let (hits, duration_ms) =
                            engine.search_semantic(query, mode, max_results).await;
                        Message::SemanticResultsReceived {
                            results: hits,
                            search_duration_ms: duration_ms,
                        }
                    })
                } else {
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
            }
            Message::KeyboardEvent(event) => {
                if let iced::keyboard::Event::KeyPressed { key, modifiers, .. } = event {
                    if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) {
                        if self.state.properties_modal.is_some() {
                            self.state.properties_modal = None;
                            return Task::none();
                        } else if self.state.context_menu.is_open {
                            self.state.context_menu.is_open = false;
                            return Task::none();
                        } else if !self.state.query.is_empty() {
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
                                    return self.update(Message::ShowProperties(path));
                                } else {
                                    return self.update(Message::OpenFile(path));
                                }
                            }
                        }
                    } else if key
                        == iced::keyboard::Key::Named(iced::keyboard::key::Named::ContextMenu)
                        || (modifiers.shift()
                            && key == iced::keyboard::Key::Named(iced::keyboard::key::Named::F10))
                    {
                        if let Some(idx) = self.state.selected_row {
                            if let Some(item) = self.state.results.get(idx) {
                                return self.update(Message::OpenContextMenu {
                                    item: item.clone(),
                                    x: 0.0,
                                    y: 0.0,
                                });
                            }
                        }
                    }

                    let is_ctrl_f = modifiers.control()
                        && !modifiers.shift()
                        && !modifiers.alt()
                        && matches!(key, iced::keyboard::Key::Character(ref s) if s.eq_ignore_ascii_case("f"));
                    let is_ctrl_k = modifiers.control()
                        && !modifiers.shift()
                        && !modifiers.alt()
                        && matches!(key, iced::keyboard::Key::Character(ref s) if s.eq_ignore_ascii_case("k"));
                    let is_slash = key == iced::keyboard::Key::Character("/".into())
                        && !modifiers.control()
                        && !modifiers.alt()
                        && self.state.active_screen != ScreenId::Search;

                    if is_ctrl_f || is_ctrl_k || is_slash {
                        self.state.active_screen = ScreenId::Search;
                        return operation::focus(search_input_id());
                    }

                    // Ctrl + 1 .. 6: Navigation screen switching
                    if modifiers.control() && !modifiers.shift() && !modifiers.alt() {
                        let target_screen = match key {
                            iced::keyboard::Key::Character(ref s) if s == "1" => {
                                Some(ScreenId::Search)
                            }
                            iced::keyboard::Key::Character(ref s) if s == "2" => {
                                Some(ScreenId::AdvancedSearch)
                            }
                            iced::keyboard::Key::Character(ref s) if s == "3" => {
                                Some(ScreenId::IndexManagement)
                            }
                            iced::keyboard::Key::Character(ref s) if s == "4" => {
                                Some(ScreenId::SearchHistory)
                            }
                            iced::keyboard::Key::Character(ref s) if s == "5" => {
                                Some(ScreenId::Settings)
                            }
                            iced::keyboard::Key::Character(ref s) if s == "6" => {
                                Some(ScreenId::About)
                            }
                            _ => None,
                        };
                        if let Some(screen) = target_screen {
                            return self.update(Message::SwitchScreen(screen));
                        }
                    }

                    // Ctrl + B: Toggle sidebar expansion
                    let is_ctrl_b = modifiers.control()
                        && !modifiers.shift()
                        && !modifiers.alt()
                        && matches!(key, iced::keyboard::Key::Character(ref s) if s.eq_ignore_ascii_case("b"));
                    if is_ctrl_b {
                        return self.update(Message::ToggleSidebar);
                    }

                    // Ctrl + T: Toggle visual theme (dark/light)
                    let is_ctrl_t = modifiers.control()
                        && !modifiers.shift()
                        && !modifiers.alt()
                        && matches!(key, iced::keyboard::Key::Character(ref s) if s.eq_ignore_ascii_case("t"));
                    if is_ctrl_t {
                        return self.update(Message::ToggleTheme);
                    }

                    // F5 or Ctrl + R: Refresh / re-execute current search query
                    let is_f5 = key == iced::keyboard::Key::Named(iced::keyboard::key::Named::F5);
                    let is_ctrl_r = modifiers.control()
                        && !modifiers.shift()
                        && !modifiers.alt()
                        && matches!(key, iced::keyboard::Key::Character(ref s) if s.eq_ignore_ascii_case("r"));
                    if is_f5 || is_ctrl_r {
                        let q = self.state.query.clone();
                        return self.update(Message::ExecuteSearch(q));
                    }

                    // Ctrl + Shift + C / N / U: Instant clipboard actions for selected row
                    if modifiers.control() && modifiers.shift() {
                        if let Some(idx) = self.state.selected_row {
                            if let Some(item) = self.state.results.get(idx) {
                                match key {
                                    iced::keyboard::Key::Character(ref s)
                                        if s.eq_ignore_ascii_case("c") =>
                                    {
                                        return self.update(Message::CopyPath(item.path.clone()));
                                    }
                                    iced::keyboard::Key::Character(ref s)
                                        if s.eq_ignore_ascii_case("n") =>
                                    {
                                        return self
                                            .update(Message::CopyFileName(item.name.clone()));
                                    }
                                    iced::keyboard::Key::Character(ref s)
                                        if s.eq_ignore_ascii_case("u") =>
                                    {
                                        return self
                                            .update(Message::CopyFileUri(item.path.clone()));
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    // Alt + 1 .. 5: Sort table by column
                    if modifiers.alt() && !modifiers.control() && !modifiers.shift() {
                        let col = match key {
                            iced::keyboard::Key::Character(ref s) if s == "1" => {
                                Some(ColumnId::Name)
                            }
                            iced::keyboard::Key::Character(ref s) if s == "2" => {
                                Some(ColumnId::Path)
                            }
                            iced::keyboard::Key::Character(ref s) if s == "3" => {
                                Some(ColumnId::Size)
                            }
                            iced::keyboard::Key::Character(ref s) if s == "4" => {
                                Some(ColumnId::Modified)
                            }
                            iced::keyboard::Key::Character(ref s) if s == "5" => {
                                Some(ColumnId::Type)
                            }
                            _ => None,
                        };
                        if let Some(col_id) = col {
                            return self.update(Message::SortByColumn(col_id));
                        }
                    }
                }
                Task::none()
            }
            Message::SwitchScreen(screen) => {
                self.state.active_screen = screen;
                self.state.context_menu.is_open = false;
                self.state.properties_modal = None;
                if screen == ScreenId::Search {
                    operation::focus(search_input_id())
                } else {
                    Task::none()
                }
            }
            Message::ToggleSidebar => {
                self.state.sidebar_expanded = !self.state.sidebar_expanded;
                Task::none()
            }
            Message::ToggleFolderTreeNode(folder) => {
                if self.state.expanded_folders.contains(&folder) {
                    self.state.expanded_folders.remove(&folder);
                } else {
                    self.state.expanded_folders.insert(folder);
                }
                Task::none()
            }
            Message::SelectFolderFilter(opt_filter) => {
                self.state.active_folder_filter = opt_filter.clone();
                self.state.active_screen = ScreenId::Search;
                let query = match opt_filter {
                    Some(f) => format!("path:\"{}\"", f),
                    None => String::new(),
                };
                self.update(Message::SearchInputChanged(query))
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
                self.state.context_menu.is_open = false;
                let _ = open::that_detached(path);
                Task::none()
            }
            Message::OpenContainingFolder(path) => {
                self.state.context_menu.is_open = false;
                match crate::desktop::show_in_file_manager(std::path::Path::new(&path)) {
                    Ok(()) => {
                        self.state.notification = Some("Highlighted in file manager".to_string())
                    }
                    Err(e) => self.state.notification = Some(format!("Failed to open folder: {e}")),
                }
                Task::none()
            }
            Message::CopyPath(path) => {
                self.state.context_menu.is_open = false;
                match crate::desktop::copy_to_clipboard(&path) {
                    Ok(()) => {
                        self.state.notification = Some("Copied full path to clipboard".to_string())
                    }
                    Err(e) => self.state.notification = Some(format!("Failed to copy path: {e}")),
                }
                Task::none()
            }
            Message::CopyFileName(name) => {
                self.state.context_menu.is_open = false;
                match crate::desktop::copy_to_clipboard(&name) {
                    Ok(()) => {
                        self.state.notification =
                            Some(format!("Copied file name to clipboard: {name}"))
                    }
                    Err(e) => {
                        self.state.notification = Some(format!("Failed to copy file name: {e}"))
                    }
                }
                Task::none()
            }
            Message::CopyFileUri(path) => {
                self.state.context_menu.is_open = false;
                match crate::desktop::copy_file_uri(std::path::Path::new(&path)) {
                    Ok(uri) => {
                        self.state.notification =
                            Some(format!("Copied file URL to clipboard: {uri}"))
                    }
                    Err(e) => {
                        self.state.notification = Some(format!("Failed to copy file URL: {e}"))
                    }
                }
                Task::none()
            }
            Message::OpenContextMenu { item, x, y } => {
                self.state.context_menu = crate::state::ContextMenuState {
                    is_open: true,
                    target_item: Some(item),
                    position: (x, y),
                };
                Task::none()
            }
            Message::CloseContextMenu => {
                self.state.context_menu.is_open = false;
                Task::none()
            }
            Message::ShowProperties(path) => {
                self.state.context_menu.is_open = false;
                match crate::desktop::get_file_properties(std::path::Path::new(&path)) {
                    Ok(props) => self.state.properties_modal = Some(props),
                    Err(e) => {
                        self.state.notification =
                            Some(format!("Could not read file properties: {e}"))
                    }
                }
                Task::none()
            }
            Message::CloseProperties => {
                self.state.properties_modal = None;
                Task::none()
            }
            Message::ExportResults(format, path) => {
                match findme_core::export_to_file(format, &self.state.results, &path) {
                    Ok(count) => {
                        self.state.notification =
                            Some(format!("Exported {} results to {}", count, path.display()));
                    }
                    Err(e) => {
                        self.state.notification = Some(format!("Export failed: {}", e));
                    }
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
                self.state.semantic_results.clear();
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
            Message::SetSearchMode(mode) => {
                self.state.search_mode = mode;
                let q = self.state.query.clone();
                if !q.is_empty() {
                    return self.update(Message::ExecuteSearch(q));
                }
                Task::none()
            }
            Message::ToggleSnippetExpansion(row_idx) => {
                if self.state.expanded_snippets.contains(&row_idx) {
                    self.state.expanded_snippets.remove(&row_idx);
                } else {
                    self.state.expanded_snippets.insert(row_idx);
                }
                Task::none()
            }
            Message::JumpAudioTimestamp {
                path,
                timestamp_secs,
            } => {
                let _ = open::that_detached(&path);
                let mins = (timestamp_secs / 60.0).floor() as u64;
                let secs = (timestamp_secs % 60.0).floor() as u64;
                self.state.notification = Some(format!(
                    "Playing audio file from {:02}:{:02} ({path})",
                    mins, secs
                ));
                Task::none()
            }
            Message::SemanticResultsReceived {
                results,
                search_duration_ms,
            } => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                self.state.history.record_query(
                    &self.state.query,
                    results.len(),
                    search_duration_ms,
                    now,
                );
                self.state.semantic_results = results.clone();
                self.state.last_search_duration_ms = search_duration_ms;
                self.state.is_searching = false;
                self.state.scroll_y = 0.0;

                let mut converted = Vec::with_capacity(results.len());
                for (idx, hit) in results.into_iter().enumerate() {
                    let name = hit
                        .path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("document")
                        .to_string();
                    converted.push(SearchResultItem {
                        id: hit.vector_id.max(idx as u64),
                        name,
                        path: hit.path.to_string_lossy().to_string(),
                        size_bytes: hit.size_bytes,
                        mtime_secs: hit.mtime,
                        is_dir: false,
                    });
                }
                self.state.results = converted;
                self.state.selected_row = if self.state.results.is_empty() {
                    None
                } else {
                    Some(0)
                };
                Task::none()
            }
            Message::SetThrottleMode(mode) => {
                self.state.throttle_mode = mode;
                self.engine.set_throttle_mode(mode);
                Task::none()
            }
            Message::SemanticProgressReceived(progress) => {
                self.state.semantic_progress = progress;
                Task::none()
            }
            Message::TriggerSemanticRebuild => {
                let roots: Vec<std::path::PathBuf> = self
                    .state
                    .index_mgmt
                    .roots
                    .iter()
                    .map(|r| r.path.clone())
                    .collect();
                self.state.semantic_progress.is_indexing = true;
                self.engine.start_semantic_indexing(roots);
                self.state.notification =
                    Some("Semantic vector indexing started in background".to_string());
                Task::none()
            }
            Message::ToggleSemanticPause => {
                if self.state.semantic_progress.is_paused {
                    self.engine.resume_semantic_indexing();
                    self.state.semantic_progress.is_paused = false;
                    self.state.notification = Some("Semantic indexing resumed".to_string());
                } else {
                    self.engine.pause_semantic_indexing();
                    self.state.semantic_progress.is_paused = true;
                    self.state.notification = Some("Semantic indexing paused".to_string());
                }
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
                    AdvancedSearchMessage::SelectExtCategory(cat) => {
                        self.state.advanced_filters.active_ext_category = cat;
                    }
                    AdvancedSearchMessage::ToggleExtension(ext) => {
                        self.state.advanced_filters.toggle_extension(&ext);
                    }
                    AdvancedSearchMessage::SelectAllInCategory(cat) => {
                        self.state.advanced_filters.select_all_in_category(cat);
                    }
                    AdvancedSearchMessage::ClearCategory(cat) => {
                        self.state.advanced_filters.clear_category(cat);
                    }
                    AdvancedSearchMessage::ClearAllExtensions => {
                        self.state.advanced_filters.clear_all_extensions();
                    }
                    AdvancedSearchMessage::UpdateCustomExtInput(s) => {
                        self.state.advanced_filters.custom_ext_input = s;
                    }
                    AdvancedSearchMessage::AddCustomExtFromInput => {
                        let trimmed = self
                            .state
                            .advanced_filters
                            .custom_ext_input
                            .trim()
                            .to_string();
                        if !trimmed.is_empty() {
                            self.state.advanced_filters.add_custom_extension(&trimmed);
                            self.state.advanced_filters.custom_ext_input.clear();
                        }
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
                            let home =
                                std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
                            let path = if let Some(stripped) = trimmed.strip_prefix("~/") {
                                std::path::PathBuf::from(&home).join(stripped)
                            } else if trimmed == "~" {
                                std::path::PathBuf::from(&home)
                            } else {
                                std::path::PathBuf::from(trimmed)
                            };
                            if self.state.index_mgmt.add_root(path) {
                                self.state.index_mgmt.new_root_input.clear();
                                self.refresh_root_counts();
                                self.state
                                    .index_mgmt
                                    .sync_to_settings(&mut self.state.settings);
                                let _ = self.state.settings.save_default();
                                self.state.notification =
                                    Some("Added indexed root directory".to_string());
                            } else {
                                self.state.notification =
                                    Some("Directory already in indexed list".to_string());
                            }
                        }
                    }
                    IndexManagementMessage::AddRoot(path) => {
                        if self.state.index_mgmt.add_root(path) {
                            self.refresh_root_counts();
                            self.state
                                .index_mgmt
                                .sync_to_settings(&mut self.state.settings);
                            let _ = self.state.settings.save_default();
                        }
                    }
                    IndexManagementMessage::RemoveRoot(path) => {
                        if self.state.index_mgmt.remove_root(&path) {
                            self.refresh_root_counts();
                            self.state
                                .index_mgmt
                                .sync_to_settings(&mut self.state.settings);
                            let _ = self.state.settings.save_default();
                            self.state.notification =
                                Some("Removed indexed directory root".to_string());
                        }
                    }
                    IndexManagementMessage::AddQuickRoot(path_str) => {
                        let home =
                            std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
                        let resolved_path = if let Some(stripped) = path_str.strip_prefix("~/") {
                            std::path::PathBuf::from(&home).join(stripped)
                        } else if path_str == "~" {
                            std::path::PathBuf::from(&home)
                        } else {
                            std::path::PathBuf::from(&path_str)
                        };
                        if self.state.index_mgmt.add_root(resolved_path) {
                            self.refresh_root_counts();
                            self.state
                                .index_mgmt
                                .sync_to_settings(&mut self.state.settings);
                            let _ = self.state.settings.save_default();
                            self.state.notification = Some(format!("Added {}", path_str));
                        } else {
                            self.state.notification =
                                Some(format!("{} is already indexed", path_str));
                        }
                    }
                    IndexManagementMessage::UpdateNewExclusionInput(input) => {
                        self.state.index_mgmt.new_exclusion_input = input;
                    }
                    IndexManagementMessage::AddExclusionFromInput => {
                        let trimmed = self.state.index_mgmt.new_exclusion_input.trim().to_string();
                        if !trimmed.is_empty() {
                            if self.state.index_mgmt.add_smart_rule(&trimmed) {
                                self.state.index_mgmt.new_exclusion_input.clear();
                                self.state
                                    .index_mgmt
                                    .sync_to_settings(&mut self.state.settings);
                                let _ = self.state.settings.save_default();
                                self.purge_excluded_from_arena();
                                self.state.notification = Some("Added exclusion rule".to_string());
                            } else {
                                self.state.notification =
                                    Some("Exclusion pattern already exists".to_string());
                            }
                        }
                    }
                    IndexManagementMessage::AddExclusion(pat) => {
                        if self.state.index_mgmt.add_smart_rule(&pat) {
                            self.state
                                .index_mgmt
                                .sync_to_settings(&mut self.state.settings);
                            let _ = self.state.settings.save_default();
                        }
                    }
                    IndexManagementMessage::RemoveExclusion(pat) => {
                        if self.state.index_mgmt.remove_exclusion(&pat) {
                            self.state
                                .index_mgmt
                                .sync_to_settings(&mut self.state.settings);
                            let _ = self.state.settings.save_default();
                            self.state.notification = Some("Removed exclusion pattern".to_string());
                        }
                    }
                    IndexManagementMessage::AddDefaultExclusions => {
                        self.state.index_mgmt.add_default_exclusions();
                        self.state
                            .index_mgmt
                            .sync_to_settings(&mut self.state.settings);
                        let _ = self.state.settings.save_default();
                        self.state.notification = Some("Default exclusion rules added".to_string());
                    }
                    IndexManagementMessage::TogglePreset(preset) => {
                        self.state.index_mgmt.toggle_preset(preset);
                        self.state
                            .index_mgmt
                            .sync_to_settings(&mut self.state.settings);
                        let _ = self.state.settings.save_default();
                        self.purge_excluded_from_arena();
                        self.state.notification =
                            Some(format!("Toggled preset {}", preset.label()));
                    }
                    IndexManagementMessage::AddExcludedExtension(ext) => {
                        if self.state.index_mgmt.add_excluded_extension(&ext) {
                            self.state
                                .index_mgmt
                                .sync_to_settings(&mut self.state.settings);
                            let _ = self.state.settings.save_default();
                            self.purge_excluded_from_arena();
                            self.state.notification = Some(format!("Excluded extension *.{}", ext));
                        }
                    }
                    IndexManagementMessage::RemoveExcludedExtension(ext) => {
                        if self.state.index_mgmt.remove_excluded_extension(&ext) {
                            self.state
                                .index_mgmt
                                .sync_to_settings(&mut self.state.settings);
                            let _ = self.state.settings.save_default();
                            self.state.notification = Some(format!("Removed extension *.{}", ext));
                        }
                    }
                    IndexManagementMessage::AddExcludedFolder(folder) => {
                        if self.state.index_mgmt.add_excluded_folder(&folder) {
                            self.state
                                .index_mgmt
                                .sync_to_settings(&mut self.state.settings);
                            let _ = self.state.settings.save_default();
                            self.purge_excluded_from_arena();
                            self.state.notification = Some(format!("Excluded folder {}/", folder));
                        }
                    }
                    IndexManagementMessage::RemoveExcludedFolder(folder) => {
                        if self.state.index_mgmt.remove_excluded_folder(&folder) {
                            self.state
                                .index_mgmt
                                .sync_to_settings(&mut self.state.settings);
                            let _ = self.state.settings.save_default();
                            self.state.notification = Some(format!("Removed folder {}/", folder));
                        }
                    }
                    IndexManagementMessage::AddExcludedPath(path) => {
                        if self.state.index_mgmt.add_excluded_path(&path) {
                            self.state
                                .index_mgmt
                                .sync_to_settings(&mut self.state.settings);
                            let _ = self.state.settings.save_default();
                            self.state.notification = Some(format!("Excluded path {}", path));
                        }
                    }
                    IndexManagementMessage::RemoveExcludedPath(path) => {
                        if self.state.index_mgmt.remove_excluded_path(&path) {
                            self.state
                                .index_mgmt
                                .sync_to_settings(&mut self.state.settings);
                            let _ = self.state.settings.save_default();
                            self.state.notification = Some(format!("Removed path {}", path));
                        }
                    }
                    IndexManagementMessage::ToggleGitignoreRawView => {
                        self.state.index_mgmt.view_gitignore_raw =
                            !self.state.index_mgmt.view_gitignore_raw;
                    }
                    IndexManagementMessage::UpdateNewExtensionInput(input) => {
                        self.state.index_mgmt.new_extension_input = input;
                    }
                    IndexManagementMessage::UpdateNewFolderInput(input) => {
                        self.state.index_mgmt.new_folder_input = input;
                    }
                    IndexManagementMessage::AddExtensionFromInput => {
                        let trimmed = self.state.index_mgmt.new_extension_input.trim().to_string();
                        if !trimmed.is_empty() {
                            if self.state.index_mgmt.add_excluded_extension(&trimmed) {
                                self.state.index_mgmt.new_extension_input.clear();
                                self.state
                                    .index_mgmt
                                    .sync_to_settings(&mut self.state.settings);
                                let _ = self.state.settings.save_default();
                                self.state.notification =
                                    Some(format!("Excluded extension *.{}", trimmed));
                            } else {
                                self.state.notification =
                                    Some("Extension already excluded".to_string());
                            }
                        }
                    }
                    IndexManagementMessage::AddFolderFromInput => {
                        let trimmed = self.state.index_mgmt.new_folder_input.trim().to_string();
                        if !trimmed.is_empty() {
                            if self.state.index_mgmt.add_excluded_folder(&trimmed) {
                                self.state.index_mgmt.new_folder_input.clear();
                                self.state
                                    .index_mgmt
                                    .sync_to_settings(&mut self.state.settings);
                                let _ = self.state.settings.save_default();
                                self.state.notification =
                                    Some(format!("Excluded folder {}/", trimmed));
                            } else {
                                self.state.notification =
                                    Some("Folder already excluded".to_string());
                            }
                        }
                    }
                    IndexManagementMessage::CopyGitignoreRules => {
                        let rules = self.state.index_mgmt.to_gitignore_string();
                        let _ = crate::desktop::copy_to_clipboard(&rules);
                        self.state.notification =
                            Some("Copied .gitignore rules to clipboard".to_string());
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
                            .excluded_paths
                            .iter()
                            .map(std::path::PathBuf::from)
                            .collect();
                        let excluded_names = self.state.index_mgmt.excluded_folders.clone();
                        let excluded_extensions = self.state.index_mgmt.excluded_extensions.clone();
                        let excluded_globs: Vec<String> = self
                            .state
                            .index_mgmt
                            .exclusions
                            .iter()
                            .filter(|e| e.pattern.contains('*'))
                            .map(|e| e.pattern.clone())
                            .collect();
                        self.engine.rebuild_index_with_rules(
                            roots,
                            exclusions,
                            excluded_names,
                            excluded_extensions,
                            excluded_globs,
                        );
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
                        self.refresh_root_counts();
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
                        self.refresh_root_counts();
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
                    EngineEvent::SemanticProgress(progress) => {
                        self.state.semantic_progress = progress;
                    }
                    EngineEvent::SemanticIndexReady {
                        indexed_chunks,
                        total_files,
                    } => {
                        self.state.semantic_progress.is_indexing = false;
                        self.state.semantic_chunks_count = indexed_chunks;
                        self.state.notification = Some(format!(
                            "Semantic vector indexing complete: {} files indexed ({} vector chunks)",
                            crate::state::format_number(total_files as u64),
                            crate::state::format_number(indexed_chunks as u64)
                        ));
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

    /// Instant in-memory purge of files matching active exclusion rules (<2ms)
    pub fn purge_excluded_from_arena(&mut self) -> usize {
        let excluded_folders = self.state.index_mgmt.excluded_folders.clone();
        let excluded_exts = self.state.index_mgmt.excluded_extensions.clone();
        let excluded_paths = self.state.index_mgmt.excluded_paths.clone();

        self.engine.purge_matching(move |name, path, is_dir| {
            if is_dir
                && excluded_folders
                    .iter()
                    .any(|f| f.eq_ignore_ascii_case(name))
            {
                return true;
            }
            for part in path.components() {
                if let std::path::Component::Normal(comp) = part {
                    let comp_str = comp.to_string_lossy();
                    if excluded_folders
                        .iter()
                        .any(|f| f.eq_ignore_ascii_case(&comp_str))
                    {
                        return true;
                    }
                }
            }
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if excluded_exts.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
                    return true;
                }
            }
            let path_str = path.to_string_lossy();
            for p in &excluded_paths {
                if path_str.starts_with(p) {
                    return true;
                }
            }
            false
        })
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

        let workspace_container = container(workspace_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_theme| container::Style {
                background: Some(Background::Color(self.state.theme.bg_color())),
                ..Default::default()
            });

        let base_layout: Element<'_, Message> = container(
            row![sidebar, workspace_container]
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(Background::Color(self.state.theme.bg_color())),
            ..Default::default()
        })
        .into();

        if self.state.properties_modal.is_some() || self.state.context_menu.is_open {
            let mut layers: Vec<Element<'_, Message>> = vec![base_layout];

            if self.state.context_menu.is_open {
                if let Some(ref item) = self.state.context_menu.target_item {
                    let menu_elem = views::context_menu::view(item);
                    let menu_overlay = container(menu_elem)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_x(Length::Fill)
                        .center_y(Length::Fill);

                    layers.push(menu_overlay.into());
                }
            }

            if let Some(ref props) = self.state.properties_modal {
                let modal_elem = views::properties::view(props);
                let modal_overlay = container(modal_elem)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill);

                layers.push(modal_overlay.into());
            }

            iced::widget::Stack::from_vec(layers)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            base_layout
        }
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
