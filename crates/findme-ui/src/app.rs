//! Top-level FindMeApp Elm-architecture loop

use crate::message::{ColumnId, Message};
use crate::state::AppState;
use iced::widget::{column, container, text};
use iced::{Element, Length, Task, Theme};

/// Main application controller
#[derive(Debug, Default)]
pub struct FindMeApp {
    pub state: AppState,
}

impl FindMeApp {
    pub fn new() -> Self {
        Self {
            state: AppState::default(),
        }
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

    /// Renders the UI widget tree
    pub fn view(&self) -> Element<'_, Message> {
        let content = column![
            text(format!(
                "Find-Me - Active Screen: {:?}",
                self.state.active_screen
            ))
            .size(18),
            text(format!(
                "Indexed Files: {} | Folders: {}",
                self.state.indexed_files_count, self.state.indexed_folders_count
            ))
            .size(13),
            text(format!(
                "Query: '{}' | Matches: {}",
                self.state.query,
                self.state.results.len()
            ))
            .size(14),
        ]
        .spacing(12);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(16)
            .into()
    }

    /// Resolves theme for Iced engine
    pub fn theme(&self) -> Theme {
        if self.state.theme.is_dark() {
            Theme::Dark
        } else {
            Theme::Light
        }
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
    }

    #[test]
    fn test_theme_toggle() {
        let mut app = FindMeApp::new();
        assert!(app.state.theme.is_dark());

        let _ = app.update(Message::ToggleTheme);
        assert!(!app.state.theme.is_dark());
    }
}
