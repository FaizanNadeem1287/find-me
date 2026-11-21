//! About screen displaying application branding and version details

use crate::message::Message;
use crate::state::AppState;
use iced::widget::{column, container, scrollable, text, Space};
use iced::{Element, Length};

/// Renders the About screen
pub fn view(_state: &AppState) -> Element<'_, Message> {
    let header = column![
        text("ℹ️ About Find-Me").size(18),
        text("Instant, offline-first Linux desktop file search").size(12),
    ]
    .spacing(4);

    let about_card = container(
        column![
            text("Find-Me Desktop v0.1.0").size(16),
            text("A high-performance file search utility inspired by Windows Everything, written in 100% pure Rust with an Iced GUI.").size(13),
            Space::new().height(Length::Fixed(8.0)),
            text("Architecture & Technologies:").size(13),
            text("• GUI: Iced 0.14 Elm-Architecture (tiny-skia software rendering)").size(12),
            text("• Memory Arena: Compact 32-byte FileRecord with StringPool & DirectoryTable").size(12),
            text("• Search Engine: Two-stage filter pipeline with nucleo-matcher fuzzy scoring").size(12),
            text("• Real-Time Monitoring: Linux inotify event synchronization").size(12),
            text("• Storage: Zero-copy binary persistence via memmap2 and SIMD CRC32").size(12),
            Space::new().height(Length::Fixed(8.0)),
            text("License: MIT OR Apache-2.0").size(12),
            text("Repository: https://github.com/Faizan-Nadeem/Find-Me").size(12),
        ]
        .spacing(6),
    )
    .padding(14)
    .width(Length::Fill);

    let content = column![header, about_card].spacing(14);

    scrollable(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
