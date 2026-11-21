//! Main search view layout

use crate::message::Message;
use crate::state::{format_number, search_input_id, AppState};
use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Alignment, Element, Length};

/// Renders the primary Search screen
pub fn view(state: &AppState) -> Element<'_, Message> {
    // Top search bar
    let search_icon = text("🔍").size(16);

    let search_input = text_input(
        "Type to search files, paths, or operators (e.g. ext:rs size:>1mb)...",
        &state.query,
    )
    .id(search_input_id())
    .on_input(Message::SearchInputChanged)
    .padding(10)
    .size(15)
    .width(Length::Fill);

    let mut search_row = row![search_icon, search_input]
        .spacing(10)
        .align_y(Alignment::Center)
        .width(Length::Fill);

    // Dynamic clear button rendered only when query text is non-empty
    if !state.query.is_empty() {
        let clear_btn = button(text("✕").size(13))
            .padding([8, 12])
            .on_press(Message::ClearSearch);
        search_row = search_row.push(clear_btn);
    }

    let search_bar = container(search_row).padding(4).width(Length::Fill);

    // Results status & stats bar
    let matches_text = if state.query.is_empty() {
        format!(
            "Showing all files ({})",
            format_number(state.results.len() as u64)
        )
    } else {
        format!(
            "Query: \"{}\" • {} matches ({} ms)",
            state.query,
            format_number(state.results.len() as u64),
            state.last_search_duration_ms
        )
    };

    let indexed_text = format!(
        "Indexed: {} files in {} folders",
        format_number(state.indexed_files_count as u64),
        format_number(state.indexed_folders_count as u64)
    );

    let stats_header = row![
        text(matches_text).size(12),
        Space::new().width(Length::Fill),
        text(indexed_text).size(12),
    ]
    .align_y(Alignment::Center);

    // Results table view (virtualized 60 FPS table with sort headers)
    let results_table = crate::views::table::view(state);

    // Bottom footer status bar
    let status_bar = row![
        text(if state.is_searching {
            "⚡ Searching index..."
        } else {
            "✓ Ready"
        })
        .size(11),
        Space::new().width(Length::Fill),
        text("Shortcuts: Ctrl+1..6 Screens • Ctrl+F Search • Ctrl+B Sidebar • Ctrl+T Theme • Enter Open • Alt+Enter Properties")
            .size(11),
        Space::new().width(Length::Fill),
        text("Find-Me Linux • Offline Engine Active").size(11),
    ]
    .align_y(Alignment::Center);

    column![search_bar, stats_header, results_table, status_bar]
        .spacing(12)
        .padding(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
