//! Responsive sidebar navigation component

use crate::message::{Message, ScreenId};
use crate::state::AppState;
use iced::widget::{button, column, container, scrollable, text};
use iced::{Alignment, Element, Length};

/// Renders the sidebar navigation panel
pub fn view(state: &AppState) -> Element<'_, Message> {
    let width = if state.sidebar_expanded {
        Length::Fixed(220.0)
    } else {
        Length::Fixed(68.0)
    };

    // Header branding
    let brand_title = if state.sidebar_expanded {
        column![text("Find-Me").size(18), text("v0.1.0 • Desktop").size(11),].spacing(2)
    } else {
        column![text("FM").size(16), text("0.1").size(10)].spacing(2)
    };

    let brand_section = container(brand_title).padding(12);

    // Navigation links
    let mut nav_items = column![].spacing(6);

    for screen in ScreenId::ALL {
        let is_active = state.active_screen == screen;
        let nav_btn = if state.sidebar_expanded {
            let label_text = if is_active {
                format!("▶ {}  {}", screen.icon(), screen.title())
            } else {
                format!("  {}  {}", screen.icon(), screen.title())
            };
            button(text(label_text).size(13))
                .width(Length::Fill)
                .padding([8, 12])
                .on_press(Message::SwitchScreen(screen))
        } else {
            button(
                container(text(screen.icon()).size(16))
                    .width(Length::Fill)
                    .align_x(Alignment::Center),
            )
            .width(Length::Fill)
            .padding([8, 4])
            .on_press(Message::SwitchScreen(screen))
        };

        nav_items = nav_items.push(nav_btn);
    }

    let nav_scroll = scrollable(nav_items).height(Length::Fill);

    // Footer actions
    let collapse_label = if state.sidebar_expanded {
        "⇤ Collapse"
    } else {
        "⇥"
    };

    let theme_label = if state.sidebar_expanded {
        if state.theme.is_dark() {
            "🌙 Dark Mode"
        } else {
            "☀️ Light Mode"
        }
    } else if state.theme.is_dark() {
        "🌙"
    } else {
        "☀️"
    };

    let collapse_btn = button(text(collapse_label).size(12))
        .width(Length::Fill)
        .padding([6, 10])
        .on_press(Message::ToggleSidebar);

    let theme_btn = button(text(theme_label).size(12))
        .width(Length::Fill)
        .padding([6, 10])
        .on_press(Message::ToggleTheme);

    let footer_section = column![collapse_btn, theme_btn].spacing(6).padding(12);

    let sidebar_layout = column![brand_section, nav_scroll, footer_section]
        .width(width)
        .height(Length::Fill);

    container(sidebar_layout)
        .width(width)
        .height(Length::Fill)
        .into()
}
