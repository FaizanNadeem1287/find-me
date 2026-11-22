//! Search history and saved searches screen

use crate::message::{HistoryMessage, Message};
use crate::state::AppState;
use crate::theme::{self, colors};
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};

/// Renders the Search History screen
pub fn view(state: &AppState) -> Element<'_, Message> {
    let history = &state.history;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(1_700_000_000);

    // 1. Header & Filter Bar
    let header_title = column![
        text("🕒 Search History & Starred Bookmarks").size(18),
        text("Quick access to recent queries, execution latency, and saved bookmark expressions")
            .size(12),
    ]
    .spacing(4);

    let star_filter_label = if history.filter_bookmarked_only {
        "★ Showing Starred Only"
    } else {
        "☆ All Queries"
    };

    let controls_bar = row![
        text_input("Filter history queries...", &history.search_filter)
            .on_input(|s| Message::History(HistoryMessage::UpdateSearchFilter(s)))
            .style(theme::search_input_style(&state.theme))
            .padding(8)
            .size(13)
            .width(Length::Fill),
        button(text(star_filter_label).size(12))
            .padding([6, 12])
            .style(theme::chip_btn_style(
                &state.theme,
                history.filter_bookmarked_only
            ))
            .on_press(Message::History(HistoryMessage::ToggleFilterBookmarked(
                !history.filter_bookmarked_only,
            ))),
        button(text("🗑️ Clear History").size(12))
            .padding([6, 12])
            .style(theme::danger_btn_style(&state.theme))
            .on_press(Message::History(HistoryMessage::ClearHistory)),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let header_card = container(column![header_title, controls_bar,].spacing(14))
        .padding(16)
        .style(theme::card_style(&state.theme))
        .width(Length::Fill);

    // 2. Entries List
    let filtered = history.filtered_entries();

    let mut entries_col = column![].spacing(8);

    if filtered.is_empty() {
        let empty_card = container(
            column![
                text("🔍").size(24),
                text(if history.entries.is_empty() {
                    "No search history recorded yet. Executed queries will appear here automatically."
                } else {
                    "No search queries match the active filter criteria."
                })
                .size(13),
            ]
            .spacing(8)
            .align_x(Alignment::Center),
        )
        .padding(32)
        .style(theme::card_style(&state.theme))
        .center_x(Length::Fill)
        .width(Length::Fill);

        entries_col = entries_col.push(empty_card);
    } else {
        for entry in filtered {
            let star_label = if entry.is_bookmarked { "★" } else { "☆" };
            let relative_time = entry.formatted_relative_time(now);
            let meta_text = format!(
                "{}  •  {} ms  •  {}",
                entry.formatted_matches(),
                entry.duration_ms,
                relative_time
            );

            let entry_id = entry.id;
            let query_text = entry.query.clone();

            let entry_card = container(
                row![
                    button(text(star_label).size(15).color(if entry.is_bookmarked {
                        colors::ACCENT_AMBER
                    } else {
                        colors::DARK_MUTED
                    }))
                    .padding([4, 8])
                    .style(theme::secondary_btn_style(&state.theme))
                    .on_press(Message::History(HistoryMessage::ToggleBookmark(entry_id))),
                    column![text(query_text.clone()).size(13), text(meta_text).size(11),]
                        .spacing(2),
                    Space::new().width(Length::Fill),
                    button(text("🔍 Run Search").size(11))
                        .padding([5, 12])
                        .style(theme::secondary_btn_style(&state.theme))
                        .on_press(Message::History(HistoryMessage::ExecuteQuery(query_text,))),
                    button(text("✕").size(11))
                        .padding([5, 8])
                        .style(theme::danger_btn_style(&state.theme))
                        .on_press(Message::History(HistoryMessage::RemoveEntry(entry_id))),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            )
            .padding(12)
            .style(theme::card_style(&state.theme))
            .width(Length::Fill);

            entries_col = entries_col.push(entry_card);
        }
    }

    let main_content = column![header_card, entries_col].spacing(14).padding(16);

    scrollable(main_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
