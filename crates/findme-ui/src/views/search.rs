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
        format!("Showing all files ({})", format_number(state.results.len()))
    } else {
        format!(
            "Query: \"{}\" • {} matches ({} ms)",
            state.query,
            format_number(state.results.len()),
            state.last_search_duration_ms
        )
    };

    let indexed_text = format!(
        "Indexed: {} files in {} folders",
        format_number(state.indexed_files_count),
        format_number(state.indexed_folders_count)
    );

    let stats_header = row![
        text(matches_text).size(12),
        Space::new().width(Length::Fill),
        text(indexed_text).size(12),
    ]
    .align_y(Alignment::Center);

    // Results placeholder area (to be upgraded in Task 5.2 to virtualized table)
    let results_content = if state.results.is_empty() {
        column![
            text("No matching files found.").size(14),
            text("Try broader search terms, or check filter syntax like 'ext:rs', 'size:>10mb', or 'dm:today'").size(12),
        ]
        .spacing(6)
        .align_x(Alignment::Center)
    } else {
        let sample_count = state.results.len().min(10);
        let mut sample_items = column![].spacing(4);
        for item in &state.results[0..sample_count] {
            let row_item = row![
                text(if item.is_dir { "📁" } else { "📄" }).size(12),
                text(&item.name).size(13),
                text(format!("({})", item.path)).size(11),
                Space::new().width(Length::Fill),
                text(item.formatted_size()).size(12),
            ]
            .spacing(8)
            .align_y(Alignment::Center);
            sample_items = sample_items.push(row_item);
        }
        sample_items
    };

    let results_panel = container(results_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(16);

    // Bottom footer status bar
    let status_bar = row![
        text(if state.is_searching {
            "⚡ Searching index..."
        } else {
            "✓ Ready"
        })
        .size(11),
        Space::new().width(Length::Fill),
        text("Find-Me Linux • Offline Engine Active").size(11),
    ]
    .align_y(Alignment::Center);

    column![search_bar, stats_header, results_panel, status_bar]
        .spacing(12)
        .padding(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
