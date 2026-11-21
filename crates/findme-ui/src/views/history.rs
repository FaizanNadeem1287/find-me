//! Search history and saved searches screen

use crate::message::Message;
use crate::state::AppState;
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};

/// Renders the Search History screen
pub fn view(_state: &AppState) -> Element<'_, Message> {
    let header = column![
        text("🕒 Search History & Starred Queries").size(18),
        text("Quick access to recent and frequently used search expressions").size(12),
    ]
    .spacing(4);

    let recent_queries = [
        ("ext:rs size:>100kb", "Source code files over 100 KB"),
        ("path:~/Documents dm:today", "Documents modified today"),
        ("Cargo.toml", "Rust package manifests"),
        ("regex:^lib.*\\.so$", "Dynamic shared library objects"),
    ];

    let mut query_list = column![text("Recent Searches").size(14)].spacing(8);

    for (query, desc) in recent_queries {
        let q_row = row![
            text("🔍").size(12),
            text(query).size(13),
            text(format!("— {desc}")).size(11),
            Space::new().width(Length::Fill),
            button(text("Search").size(11)).on_press(Message::RunQuery(query.into())),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        query_list = query_list.push(q_row);
    }

    let history_card = container(query_list).padding(12).width(Length::Fill);

    let content = column![header, history_card].spacing(14);

    scrollable(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
