//! About screen displaying application branding, technical architecture, and system details

use crate::message::Message;
use crate::state::AppState;
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};

/// Renders the About screen
pub fn view(state: &AppState) -> Element<'_, Message> {
    let header = column![
        text("ℹ️ About Find-Me").size(18),
        text("Instant, offline-first Linux desktop file search utility").size(12),
    ]
    .spacing(4);

    let version_str = env!("CARGO_PKG_VERSION");

    // 1. Hero Card
    let hero_card = container(
        column![
            row![
                text("🔍⚡ Find-Me").size(20),
                Space::new().width(Length::Fixed(8.0)),
                container(text(format!("v{version_str}")).size(12))
                    .padding([2, 6]),
            ]
            .align_y(Alignment::Center),
            text("Offline-first, high-performance desktop file search engine for Linux inspired by Windows Everything.")
                .size(13),
        ]
        .spacing(8),
    )
    .padding(14)
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
        .spacing(6),
    )
    .padding(12)
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
        .spacing(6),
    )
    .padding(12)
    .width(Length::Fill);

    // 4. License & Repository Card
    let license_card = container(
        column![
            text("License & Open Source").size(14),
            text("Licensed under dual MIT License OR Apache License 2.0.").size(12),
            row![
                text("Repository: https://github.com/Faizan-Nadeem/Find-Me").size(12),
                Space::new().width(Length::Fill),
                button(text("📋 Copy URL").size(11))
                    .padding([3, 8])
                    .on_press(Message::CopyPath(
                        "https://github.com/Faizan-Nadeem/Find-Me".to_string(),
                    )),
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(6),
    )
    .padding(12)
    .width(Length::Fill);

    let content = column![header, hero_card, tech_card, env_card, license_card,]
        .spacing(12)
        .padding(16);

    scrollable(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
