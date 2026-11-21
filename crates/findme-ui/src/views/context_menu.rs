//! Right-click file context menu component
//!
//! Provides desktop actions (Open, Show in Folder, Copy Path, Copy Name, Copy URI, Properties).

use crate::message::Message;
use findme_core::SearchResultItem;
use iced::widget::{button, column, container, rule, text};
use iced::{Element, Length};

/// Renders the floating context menu for a targeted search result item
pub fn view<'a>(item: &'a SearchResultItem) -> Element<'a, Message> {
    let open_btn = button(text("📂  Open").size(13))
        .width(Length::Fill)
        .padding([6, 12])
        .on_press(Message::OpenFile(item.path.clone()));

    let show_folder_btn = button(text("📁  Show in Containing Folder").size(13))
        .width(Length::Fill)
        .padding([6, 12])
        .on_press(Message::OpenContainingFolder(item.path.clone()));

    let copy_path_btn = button(text("📋  Copy Full Path").size(13))
        .width(Length::Fill)
        .padding([6, 12])
        .on_press(Message::CopyPath(item.path.clone()));

    let copy_name_btn = button(text("📝  Copy File Name").size(13))
        .width(Length::Fill)
        .padding([6, 12])
        .on_press(Message::CopyFileName(item.name.clone()));

    let copy_uri_btn = button(text("🔗  Copy File URL").size(13))
        .width(Length::Fill)
        .padding([6, 12])
        .on_press(Message::CopyFileUri(item.path.clone()));

    let properties_btn = button(text("ℹ️   Properties...").size(13))
        .width(Length::Fill)
        .padding([6, 12])
        .on_press(Message::ShowProperties(item.path.clone()));

    let cancel_btn = button(text("✕   Cancel").size(12))
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
