//! Modular screen views and layout containers for Find-Me

pub mod about;
pub mod advanced;
pub mod context_menu;
pub mod history;
pub mod index_mgmt;
pub mod properties;
pub mod search;
pub mod settings;
pub mod sidebar;
pub mod table;

use crate::icons::{self, Icon};
use crate::message::Message;
use crate::theme::colors;
use iced::widget::{button, container, row, text, Space};
use iced::{Element, Length};

/// Renders a top notification banner when active
pub fn notification_banner<'a>(msg: &'a str) -> Element<'a, Message> {
    let banner_content = row![
        icons::icon(Icon::Info, 16.0, colors::LIGHT_ACCENT),
        text(msg).size(13),
        Space::new().width(Length::Fill),
        button(icons::icon(Icon::Cross, 12.0, colors::DARK_MUTED))
            .padding([4, 8])
            .on_press(Message::DismissNotification),
    ]
    .spacing(12)
    .align_y(iced::Alignment::Center);

    container(banner_content)
        .width(Length::Fill)
        .padding(10)
        .into()
}
