//! Detailed file properties modal dialog
//!
//! Visualizes POSIX permissions, octal mode, inode, size, timestamps, and full paths.

use crate::desktop::FileProperties;
use crate::icons::{self, Icon};
use crate::message::Message;
use crate::theme::colors;
use iced::widget::{button, column, container, row, rule, text, Space};
use iced::{Alignment, Element, Length};

/// Renders the file properties modal dialog
pub fn view<'a>(props: &'a FileProperties) -> Element<'a, Message> {
    // Header
    let icon_type = if props.is_dir {
        Icon::Folder
    } else {
        Icon::FileText
    };
    let title = row![
        icons::icon(icon_type, 20.0, colors::LIGHT_ACCENT),
        text(&props.name).size(18),
        Space::new().width(Length::Fill),
        button(icons::icon(Icon::Cross, 14.0, colors::DARK_MUTED))
            .padding([4, 8])
            .on_press(Message::CloseProperties),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    // General Info Rows
    let type_label = if props.is_dir {
        "Directory / Folder"
    } else {
        "File"
    };

    let size_display = if props.is_dir {
        "— (Directory)".to_string()
    } else {
        format!(
            "{} ({} bytes)",
            props.formatted_size,
            crate::state::format_number(props.size_bytes)
        )
    };

    let general_section = column![
        text("General Information").size(14),
        row![
            text("Type:").size(12).width(Length::Fixed(120.0)),
            text(type_label).size(12),
        ],
        row![
            text("Size:").size(12).width(Length::Fixed(120.0)),
            text(size_display).size(12),
        ],
        row![
            text("Location:").size(12).width(Length::Fixed(120.0)),
            text(
                props
                    .path
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "/".to_string())
            )
            .size(12),
        ],
        row![
            text("Full Path:").size(12).width(Length::Fixed(120.0)),
            text(props.path.to_string_lossy().to_string()).size(12),
        ],
    ]
    .spacing(6);

    // POSIX Metadata Section
    let posix_section = column![
        text("Linux POSIX Metadata").size(14),
        row![
            text("Permissions:").size(12).width(Length::Fixed(120.0)),
            text(&props.mode_string).size(12),
        ],
        row![
            text("Inode Number:").size(12).width(Length::Fixed(120.0)),
            text(props.inode.to_string()).size(12),
        ],
        row![
            text("Device ID:").size(12).width(Length::Fixed(120.0)),
            text(props.device_id.to_string()).size(12),
        ],
        row![
            text("Attributes:").size(12).width(Length::Fixed(120.0)),
            text(&props.flags_description).size(12),
        ],
    ]
    .spacing(6);

    // Timestamps Section
    let timestamp_section = column![
        text("Timestamps").size(14),
        row![
            text("Modified:").size(12).width(Length::Fixed(120.0)),
            text(&props.mtime_formatted).size(12),
        ],
    ]
    .spacing(6);

    // Footer actions
    let copy_path_btn = button(
        row![
            icons::icon(Icon::Copy, 14.0, colors::LIGHT_ACCENT),
            text("Copy Path").size(12),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .padding([6, 12])
    .on_press(Message::CopyPath(props.path.to_string_lossy().to_string()));

    let close_btn = button(text("Close").size(12))
        .padding([6, 16])
        .on_press(Message::CloseProperties);

    let footer = row![copy_path_btn, Space::new().width(Length::Fill), close_btn,]
        .align_y(Alignment::Center);

    let modal_card = column![
        title,
        rule::horizontal(1),
        general_section,
        rule::horizontal(1),
        posix_section,
        rule::horizontal(1),
        timestamp_section,
        rule::horizontal(1),
        footer,
    ]
    .spacing(12)
    .padding(20)
    .width(Length::Fixed(520.0));

    container(modal_card).padding(4).into()
}
