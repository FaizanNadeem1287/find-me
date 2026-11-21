//! Index management and custom exclusion rules screen

use crate::message::Message;
use crate::state::AppState;
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length};

/// Renders the Index Management screen
pub fn view(state: &AppState) -> Element<'_, Message> {
    let header = column![
        text("📁 Index Management & Paths").size(18),
        text("Monitor indexed directories, configure custom exclusions, and maintain database")
            .size(12),
    ]
    .spacing(4);

    // Stats Grid
    let stats_card = container(
        column![
            text("System Storage & Index Statistics").size(14),
            row![
                column![
                    text("Total Indexed Files").size(11),
                    text(format!("{}", state.indexed_files_count)).size(16),
                ]
                .spacing(2),
                Space::new().width(Length::Fixed(32.0)),
                column![
                    text("Total Directories").size(11),
                    text(format!("{}", state.indexed_folders_count)).size(16),
                ]
                .spacing(2),
                Space::new().width(Length::Fixed(32.0)),
                column![
                    text("Linux Watcher Status").size(11),
                    text("Active (inotify)").size(16),
                ]
                .spacing(2),
            ]
            .spacing(12),
        ]
        .spacing(8),
    )
    .padding(12)
    .width(Length::Fill);

    // Actions Card
    let actions_card = container(
        column![
            text("Maintenance Actions").size(14),
            row![
                button(text("🔄 Refresh Index (Check Changes)").size(12)).on_press(
                    Message::ShowNotification("Incremental inotify refresh initiated".into())
                ),
                button(text("⚡ Force Full Rebuild").size(12)).on_press(Message::ShowNotification(
                    "Full background rescan started".into()
                )),
            ]
            .spacing(10),
        ]
        .spacing(8),
    )
    .padding(12)
    .width(Length::Fill);

    // Default Exclusions Card
    let exclusions_card = container(
        column![
            text("Active Exclusions (Virtual Filesystems & Caches)").size(14),
            text("• /proc, /sys, /dev (Linux kernel & virtual devices)").size(12),
            text("• /tmp, /run (Temporary volatile runtimes)").size(12),
            text("• $XDG_CACHE_HOME (Application cache directories)").size(12),
            text("• target/, node_modules/, .git/ (Build artifacts & version control)").size(12),
        ]
        .spacing(6),
    )
    .padding(12)
    .width(Length::Fill);

    let content = column![header, stats_card, actions_card, exclusions_card].spacing(14);

    scrollable(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
