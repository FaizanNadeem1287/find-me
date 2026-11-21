//! Main search view layout

use crate::message::Message;
use crate::state::AppState;
use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Alignment, Element, Length};

/// Renders the primary Search screen
pub fn view(state: &AppState) -> Element<'_, Message> {
    // Top search bar
    let search_input = text_input(
        "Type to search files, paths, or operators (e.g. ext:rs size:>1mb)...",
        &state.query,
    )
    .on_input(Message::SearchInputChanged)
    .padding(10)
    .size(14)
    .width(Length::Fill);

    let clear_btn = button(text("✕ Clear").size(13))
        .padding([8, 12])
        .on_press(Message::ClearSearch);

    let search_bar = row![search_input, clear_btn]
        .spacing(8)
        .align_y(Alignment::Center);

    // Results status & stats bar
    let query_summary = if state.query.is_empty() {
        format!(
            "Showing all top indexed files ({} items)",
            state.results.len()
        )
    } else {
        format!(
            "Query: '{}' • {} matches found in {} ms",
            state.query,
            state.results.len(),
            state.last_search_duration_ms
        )
    };

    let stats_header = row![
        text(query_summary).size(12),
        Space::new().width(Length::Fill),
        text(format!(
            "Database: {} files in {} folders",
            state.indexed_files_count, state.indexed_folders_count
        ))
        .size(12),
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
