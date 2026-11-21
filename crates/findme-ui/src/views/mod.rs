//! Modular screen views and layout containers for Find-Me

pub mod about;
pub mod advanced;
pub mod history;
pub mod index_mgmt;
pub mod search;
pub mod settings;
pub mod sidebar;

use crate::message::Message;
use iced::widget::{button, container, row, text, Space};
use iced::{Element, Length};

/// Renders a top notification banner when active
pub fn notification_banner<'a>(msg: &'a str) -> Element<'a, Message> {
    let banner_content = row![
        text(format!("ℹ️  {msg}")).size(13),
        Space::new().width(Length::Fill),
        button(text("✕").size(12)).on_press(Message::DismissNotification),
    ]
    .spacing(12)
    .align_y(iced::Alignment::Center);

    container(banner_content)
        .width(Length::Fill)
        .padding(10)
        .into()
}
