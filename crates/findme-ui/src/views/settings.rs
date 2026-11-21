//! Settings and user preferences screen

use crate::message::Message;
use crate::state::AppState;
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};

/// Renders the Settings screen
pub fn view(state: &AppState) -> Element<'_, Message> {
    let header = column![
        text("⚙️ Application Settings").size(18),
        text("Customize appearance, search behavior, and system integration").size(12),
    ]
    .spacing(4);

    // Appearance Card
    let theme_text = if state.theme.is_dark() {
        "Dark Theme (Active)"
    } else {
        "Light Theme (Active)"
    };

    let appearance_card = container(
        column![
            text("Appearance & Theme").size(14),
            row![
                text(theme_text).size(12),
                Space::new().width(Length::Fill),
                button(text("Toggle Theme").size(12)).on_press(Message::ToggleTheme),
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(8),
    )
    .padding(12)
    .width(Length::Fill);

    // Search Defaults Card
    let search_defaults_card = container(
        column![
            text("Search Behavior & Defaults").size(14),
            text("• Match Case: Off (Case-insensitive matching by default)").size(12),
            text("• Fuzzy Search: On (Nucleo-Matcher Smith-Waterman engine)").size(12),
            text("• Inotify Debounce: 150 ms (Batch disk bursts)").size(12),
            text("• Results Limit: 1,000 top items").size(12),
        ]
        .spacing(6),
    )
    .padding(12)
    .width(Length::Fill);

    // Hotkeys Card
    let hotkeys_card = container(
        column![
            text("Desktop Shortcuts Cheat Sheet").size(14),
            text("• Ctrl + F  or  / : Focus search input bar").size(12),
            text("• Esc : Clear current search query").size(12),
            text("• Up / Down : Navigate search result rows").size(12),
            text("• Enter : Open selected file with default desktop app").size(12),
            text("• Alt + Enter : Reveal file in default file manager").size(12),
            text("• Ctrl + Shift + C : Copy absolute file path to clipboard").size(12),
        ]
        .spacing(6),
    )
    .padding(12)
    .width(Length::Fill);

    let content = column![header, appearance_card, search_defaults_card, hotkeys_card].spacing(14);

    scrollable(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
