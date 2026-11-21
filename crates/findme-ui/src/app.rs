//! Top-level FindMeApp Elm-architecture loop

use crate::message::{ColumnId, Message, ScreenId};
use crate::state::AppState;
use crate::views;
use iced::widget::{column, row};
use iced::{Element, Length, Size, Subscription, Task, Theme};

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
#[derive(Debug, Default)]
pub struct FindMeApp {
    pub state: AppState,
}

impl FindMeApp {
    /// Creates a new default instance of FindMeApp
    pub fn new() -> Self {
        Self {
            state: AppState::default(),
        }
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
                self.state.is_searching = !self.state.query.is_empty();
                Task::none()
            }
            Message::ClearSearch => {
                self.state.query.clear();
                self.state.results.clear();
                self.state.is_searching = false;
                self.state.selected_row = None;
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
                self.state.query = query;
                self.state.active_screen = ScreenId::Search;
                self.state.is_searching = true;
                Task::none()
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
                total_matches: _,
            } => {
                self.state.results = results;
                self.state.last_search_duration_ms = search_duration_ms;
                self.state.is_searching = false;
                self.sort_results();
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
        }
    }

    /// Internal helper to sort current results based on active column and direction
    fn sort_results(&mut self) {
        let asc = self.state.sort_ascending;
        match self.state.sort_column {
            ColumnId::Name => {
                self.state.results.sort_by(|a, b| {
                    if asc {
                        a.name.cmp(&b.name)
                    } else {
                        b.name.cmp(&a.name)
                    }
                });
            }
            ColumnId::Path => {
                self.state.results.sort_by(|a, b| {
                    if asc {
                        a.path.cmp(&b.path)
                    } else {
                        b.path.cmp(&a.path)
                    }
                });
            }
            ColumnId::Type => {
                self.state.results.sort_by(|a, b| {
                    if asc {
                        a.is_dir.cmp(&b.is_dir)
                    } else {
                        b.is_dir.cmp(&a.is_dir)
                    }
                });
            }
            ColumnId::Size => {
                self.state.results.sort_by(|a, b| {
                    if asc {
                        a.size_bytes.cmp(&b.size_bytes)
                    } else {
                        b.size_bytes.cmp(&a.size_bytes)
                    }
                });
            }
            ColumnId::Modified => {
                self.state.results.sort_by(|a, b| {
                    if asc {
                        a.mtime_secs.cmp(&b.mtime_secs)
                    } else {
                        b.mtime_secs.cmp(&a.mtime_secs)
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

    /// Background event subscriptions (e.g. engine channels, window resizing)
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
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
