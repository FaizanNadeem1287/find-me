//! Right-click file context menu component
//!
//! Provides desktop actions (Open, Show in Folder, Copy Path, Copy Name, Copy URI, Properties).

use crate::icons::{self, Icon};
use crate::message::Message;
use findme_core::SearchResultItem;
use iced::widget::{button, column, container, row, rule, text};
use iced::{Alignment, Element, Length};

/// Renders the floating context menu for a targeted search result item
pub fn view<'a>(item: &'a SearchResultItem) -> Element<'a, Message> {
    let action_item = |icon_name: Icon, label: &'static str| -> Element<'a, Message> {
        row![
            icons::icon(icon_name, 13.0, iced::Color::from_rgb(0.5, 0.5, 0.5)),
            text(label).size(13),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
    };

    let open_btn = button(action_item(Icon::ExternalLink, "Open"))
        .width(Length::Fill)
        .padding([6, 12])
        .on_press(Message::OpenFile(item.path.clone()));

    let show_folder_btn = button(action_item(Icon::FolderOpen, "Show in Containing Folder"))
        .width(Length::Fill)
        .padding([6, 12])
        .on_press(Message::OpenContainingFolder(item.path.clone()));

    let copy_path_btn = button(action_item(Icon::Clipboard, "Copy Full Path"))
        .width(Length::Fill)
        .padding([6, 12])
        .on_press(Message::CopyPath(item.path.clone()));

    let copy_name_btn = button(action_item(Icon::Copy, "Copy File Name"))
        .width(Length::Fill)
        .padding([6, 12])
        .on_press(Message::CopyFileName(item.name.clone()));

    let copy_uri_btn = button(action_item(Icon::ExternalLink, "Copy File URL"))
        .width(Length::Fill)
        .padding([6, 12])
        .on_press(Message::CopyFileUri(item.path.clone()));

    let properties_btn = button(action_item(Icon::Info, "Properties..."))
        .width(Length::Fill)
        .padding([6, 12])
        .on_press(Message::ShowProperties(item.path.clone()));

    let cancel_btn = button(action_item(Icon::Cross, "Cancel"))
        .width(Length::Fill)
        .padding([4, 12])
        .on_press(Message::CloseContextMenu);

    let menu_content = column![
        open_btn,
        show_folder_btn,
        rule::horizontal(1),
        copy_path_btn,
        copy_name_btn,
        copy_uri_btn,
        rule::horizontal(1),
        properties_btn,
        rule::horizontal(1),
        cancel_btn,
    ]
    .spacing(4)
    .width(Length::Fixed(240.0));

    container(menu_content).padding(6).into()
}
