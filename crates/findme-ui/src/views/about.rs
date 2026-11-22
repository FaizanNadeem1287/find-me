//! About screen displaying application branding, technical architecture, and system details

use crate::icons::{self, Icon};
use crate::message::Message;
use crate::state::AppState;
use crate::theme::{self, colors};
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Color, Element, Length};

/// Renders the About screen
pub fn view(state: &AppState) -> Element<'_, Message> {
    let header = column![
        row![
            icons::icon(Icon::Info, 18.0, colors::LIGHT_ACCENT),
            text("About Find-Me").size(18),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        text("Instant, offline-first Linux desktop file search utility").size(12),
    ]
    .spacing(4);

    let version_str = env!("CARGO_PKG_VERSION");

    // 1. Hero Card
    let monogram = container(text("FM").size(16).color(Color::WHITE))
        .padding([6, 10])
        .style(theme::pill_badge_style(colors::DARK_ACCENT, Color::WHITE));

    let hero_card = container(
        column![
            row![
                monogram,
                text("Find-Me").size(22),
                Space::new().width(Length::Fixed(6.0)),
                container(text(format!("v{version_str}")).size(12))
                    .padding([3, 8])
                    .style(theme::pill_badge_style(
                        if state.theme.is_dark() {
                            colors::DARK_SELECTED
                        } else {
                            colors::LIGHT_SELECTED
                        },
                        if state.theme.is_dark() {
                            colors::DARK_ACCENT
                        } else {
                            colors::LIGHT_ACCENT
                        },
                    )),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            text("Offline-first, high-performance desktop file search engine for Linux inspired by Windows Everything.")
                .size(13),
        ]
        .spacing(12),
    )
    .padding(18)
    .style(theme::card_style(&state.theme))
    .width(Length::Fill);

    // 2. Architectural Highlights Card
    let tech_card = container(
        column![
            text("Engineering & Architectural Highlights").size(14),
            text("• 100% Pure Rust Stack: Zero web technologies (no Electron, no Tauri, no WebViews)").size(12),
            text("• Iced Elm-Architecture GUI: Hardware & software accelerated rendering, non-blocking 60 FPS").size(12),
            text("• Compact Memory Arena: 32-byte FileRecord layout with StringPool & hierarchical DirectoryTable").size(12),
            text("• High-Performance Fuzzy Engine: Nucleo-matcher Smith-Waterman SIMD substring scoring").size(12),
            text("• Real-Time Filesystem Monitoring: Direct Linux inotify kernel events with queue overflow recovery").size(12),
            text("• Zero-Copy Index Persistence: Memory-mapped binary disk cache with SIMD CRC32 validation").size(12),
            text("• Latency Budget: Sub-5 millisecond query evaluation across 1,000,000+ files").size(12),
        ]
        .spacing(8),
    )
    .padding(16)
    .style(theme::card_style(&state.theme))
    .width(Length::Fill);

    // 3. System Environment Card
    let stats = &state.index_mgmt.stats;
    let env_card = container(
        column![
            text("System & Host Environment").size(14),
            row![
                text("Operating System:")
                    .size(12)
                    .width(Length::Fixed(180.0)),
                text(format!("Linux ({})", std::env::consts::OS)).size(12),
            ],
            row![
                text("Host Architecture:")
                    .size(12)
                    .width(Length::Fixed(180.0)),
                text(std::env::consts::ARCH).size(12),
            ],
            row![
                text("Security & Privileges:")
                    .size(12)
                    .width(Length::Fixed(180.0)),
                text("100% Unprivileged User Space (No root/sudo required)").size(12),
            ],
            row![
                text("Active Database Memory:")
                    .size(12)
                    .width(Length::Fixed(180.0)),
                text(stats.formatted_memory()).size(12),
            ],
        ]
        .spacing(8),
    )
    .padding(16)
    .style(theme::card_style(&state.theme))
    .width(Length::Fill);

    // 4. License & Repository Card
    let license_card = container(
        column![
            text("License & Open Source").size(14),
            text("Licensed under dual MIT License OR Apache License 2.0.").size(12),
            row![
                text("Repository: https://github.com/Faizan-Nadeem/Find-Me").size(12),
                Space::new().width(Length::Fill),
                button(
                    row![
                        icons::icon(Icon::Copy, 11.0, state.theme.text_color()),
                        text("Copy URL").size(11),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .padding([5, 12])
                .style(theme::secondary_btn_style(&state.theme))
                .on_press(Message::CopyPath(
                    "https://github.com/Faizan-Nadeem/Find-Me".to_string(),
                )),
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(8),
    )
    .padding(16)
    .style(theme::card_style(&state.theme))
    .width(Length::Fill);

    let content = column![header, hero_card, tech_card, env_card, license_card,]
        .spacing(14)
        .padding(16);

    scrollable(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
