//! Modern responsive sidebar navigation component inspired by Notion / Pinterest file workspace aesthetic

use crate::message::{Message, ScreenId};
use crate::state::AppState;
use crate::theme::{colors, pill_badge_style, sidebar_item_style};
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow};

/// Renders the revamped modern sidebar navigation panel
pub fn view(state: &AppState) -> Element<'_, Message> {
    let is_dark = state.theme.is_dark();
    let width = if state.sidebar_expanded {
        Length::Fixed(240.0)
    } else {
        Length::Fixed(68.0)
    };

    // -------------------------------------------------------------------------
    // 1. Workspace Monogram Header
    // -------------------------------------------------------------------------
    let brand_section: Element<'_, Message> = if state.sidebar_expanded {
        // Monogram circular badge "FM" in vibrant indigo accent
        let monogram = container(text("FM").size(11).color(Color::WHITE))
            .width(Length::Fixed(32.0))
            .height(Length::Fixed(32.0))
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .style(|_theme| container::Style {
                background: Some(Background::Color(colors::ACCENT_PURPLE)),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 10.0.into(),
                },
                shadow: Shadow::default(),
                ..Default::default()
            });

        let ws_title = column![
            text("Find-Me Core")
                .size(13)
                .color(state.theme.text_color()),
            text("Indexed Files • Live Inotify")
                .size(10)
                .color(state.theme.muted_color()),
        ]
        .spacing(1);

        let chevron = text("↕").size(12).color(state.theme.muted_color());

        let header_row = row![
            monogram,
            ws_title,
            Space::new().width(Length::Fill),
            chevron
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        button(header_row)
            .width(Length::Fill)
            .padding([8, 10])
            .style(sidebar_item_style(&state.theme, false))
            .on_press(Message::SwitchScreen(ScreenId::Search))
            .into()
    } else {
        let monogram = container(text("FM").size(11).color(Color::WHITE))
            .width(Length::Fixed(34.0))
            .height(Length::Fixed(34.0))
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .style(|_theme| container::Style {
                background: Some(Background::Color(colors::ACCENT_PURPLE)),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 10.0.into(),
                },
                shadow: Shadow::default(),
                ..Default::default()
            });

        button(
            container(monogram)
                .width(Length::Fill)
                .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([8, 0])
        .style(sidebar_item_style(&state.theme, false))
        .on_press(Message::ToggleSidebar)
        .into()
    };

    // -------------------------------------------------------------------------
    // 2. Quick Search Pill
    // -------------------------------------------------------------------------
    let search_pill_section: Element<'_, Message> = if state.sidebar_expanded {
        let search_icon = text("🔍").size(12).color(state.theme.muted_color());
        let placeholder = text("Search files & folders...")
            .size(12)
            .color(state.theme.muted_color());

        let shortcut_pill = container(text("Ctrl+K").size(9).color(state.theme.subtext_color()))
            .padding([2, 5])
            .style(move |_theme| container::Style {
                background: Some(Background::Color(if is_dark {
                    colors::DARK_HOVER
                } else {
                    colors::LIGHT_HOVER
                })),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            });

        let pill_inner = row![
            search_icon,
            placeholder,
            Space::new().width(Length::Fill),
            shortcut_pill
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        button(pill_inner)
            .width(Length::Fill)
            .padding([7, 10])
            .style(move |_theme, status| {
                let border_col = match status {
                    button::Status::Hovered | button::Status::Pressed => {
                        if is_dark {
                            colors::DARK_ACCENT
                        } else {
                            colors::LIGHT_ACCENT
                        }
                    }
                    _ => {
                        if is_dark {
                            colors::DARK_BORDER
                        } else {
                            colors::LIGHT_BORDER
                        }
                    }
                };
                button::Style {
                    background: Some(Background::Color(if is_dark {
                        colors::DARK_SURFACE
                    } else {
                        colors::LIGHT_SURFACE
                    })),
                    border: Border {
                        color: border_col,
                        width: 1.0,
                        radius: 20.0.into(),
                    },
                    shadow: Shadow::default(),
                    ..Default::default()
                }
            })
            .on_press(Message::FocusSearchInput)
            .into()
    } else {
        button(
            container(text("🔍").size(14).color(state.theme.text_color()))
                .width(Length::Fill)
                .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([8, 0])
        .style(sidebar_item_style(&state.theme, false))
        .on_press(Message::FocusSearchInput)
        .into()
    };

    // -------------------------------------------------------------------------
    // 3. Primary Navigation Links
    // -------------------------------------------------------------------------
    let mut nav_items = column![].spacing(3);

    for screen in ScreenId::ALL {
        let is_active = state.active_screen == screen;

        let nav_btn = if state.sidebar_expanded {
            let label_row = match screen {
                ScreenId::IndexManagement => {
                    let roots_count = state.index_mgmt.roots.len();
                    let count_badge =
                        container(text(roots_count.to_string()).size(10).color(if is_active {
                            colors::LIGHT_ACCENT
                        } else {
                            state.theme.muted_color()
                        }))
                        .padding([1, 6])
                        .style(pill_badge_style(
                            if is_active {
                                colors::LIGHT_SELECTED
                            } else {
                                state.theme.hover_color()
                            },
                            colors::LIGHT_ACCENT,
                        ));

                    row![
                        text(screen.icon()).size(14),
                        text(screen.title()).size(13).color(if is_active {
                            if is_dark {
                                colors::DARK_TEXT
                            } else {
                                colors::LIGHT_ACCENT
                            }
                        } else {
                            state.theme.text_color()
                        }),
                        Space::new().width(Length::Fill),
                        count_badge,
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center)
                }
                ScreenId::SearchHistory => {
                    let history_count = state.history.entries.len();
                    let count_badge = if history_count > 0 {
                        Some(
                            container(
                                text(history_count.to_string())
                                    .size(10)
                                    .color(state.theme.muted_color()),
                            )
                            .padding([1, 6])
                            .style(pill_badge_style(
                                state.theme.hover_color(),
                                state.theme.muted_color(),
                            )),
                        )
                    } else {
                        None
                    };

                    let mut r = row![
                        text(screen.icon()).size(14),
                        text(screen.title()).size(13).color(if is_active {
                            if is_dark {
                                colors::DARK_TEXT
                            } else {
                                colors::LIGHT_ACCENT
                            }
                        } else {
                            state.theme.text_color()
                        }),
                        Space::new().width(Length::Fill),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center);

                    if let Some(badge) = count_badge {
                        r = r.push(badge);
                    }
                    r
                }
                ScreenId::AdvancedSearch => row![
                    text(screen.icon()).size(14),
                    text(screen.title()).size(13).color(if is_active {
                        if is_dark {
                            colors::DARK_TEXT
                        } else {
                            colors::LIGHT_ACCENT
                        }
                    } else {
                        state.theme.text_color()
                    }),
                    Space::new().width(Length::Fill),
                    container(text("⚡").size(10).color(colors::ACCENT_AMBER)).padding([1, 4]),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                _ => row![
                    text(screen.icon()).size(14),
                    text(screen.title()).size(13).color(if is_active {
                        if is_dark {
                            colors::DARK_TEXT
                        } else {
                            colors::LIGHT_ACCENT
                        }
                    } else {
                        state.theme.text_color()
                    }),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            };

            button(label_row)
                .width(Length::Fill)
                .padding([7, 10])
                .style(sidebar_item_style(&state.theme, is_active))
                .on_press(Message::SwitchScreen(screen))
        } else {
            button(
                container(text(screen.icon()).size(15))
                    .width(Length::Fill)
                    .align_x(Alignment::Center),
            )
            .width(Length::Fill)
            .padding([8, 0])
            .style(sidebar_item_style(&state.theme, is_active))
            .on_press(Message::SwitchScreen(screen))
        };

        nav_items = nav_items.push(nav_btn);
    }

    // -------------------------------------------------------------------------
    // 4. Hierarchical Directory Tree (When Expanded)
    // -------------------------------------------------------------------------
    let folder_tree_section: Element<'_, Message> = if state.sidebar_expanded {
        let tree_header = row![
            text("INDEX ROOTS & CATEGORIES")
                .size(10)
                .color(state.theme.muted_color()),
            Space::new().width(Length::Fill),
            button(text("+").size(12).color(state.theme.muted_color()))
                .padding([1, 6])
                .style(sidebar_item_style(&state.theme, false))
                .on_press(Message::SwitchScreen(ScreenId::IndexManagement)),
        ]
        .align_y(Alignment::Center);

        let root_expanded = state.expanded_folders.contains("General Knowledge");
        let chevron_symbol = if root_expanded { "▾" } else { "▸" };

        let root_row = row![
            button(
                text(chevron_symbol)
                    .size(11)
                    .color(state.theme.muted_color())
            )
            .padding([2, 4])
            .style(sidebar_item_style(&state.theme, false))
            .on_press(Message::ToggleFolderTreeNode(
                "General Knowledge".to_string()
            )),
            text("📁").size(13),
            text("General Knowledge")
                .size(12)
                .color(state.theme.text_color()),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        let root_btn = button(root_row)
            .width(Length::Fill)
            .padding([5, 8])
            .style(sidebar_item_style(&state.theme, false))
            .on_press(Message::SelectFolderFilter(None));

        let mut tree_col = column![tree_header, root_btn].spacing(3);

        if let Some(ref active_filter) = state.active_folder_filter {
            let clear_btn = button(
                row![
                    text("✕").size(10).color(colors::ACCENT_AMBER),
                    text(format!("Filter: {active_filter}"))
                        .size(10)
                        .color(state.theme.muted_color()),
                ]
                .spacing(4)
                .align_y(Alignment::Center),
            )
            .padding([2, 6])
            .style(sidebar_item_style(&state.theme, false))
            .on_press(Message::SelectFolderFilter(None));

            tree_col = tree_col.push(clear_btn);
        }

        if root_expanded {
            // Child categories with colored folder icons (Purple, Blue, Amber, Emerald) and file count pills
            let subfolders = [
                ("Documents", state.theme.category_amber(), "Documents", "15"),
                ("Projects", state.theme.category_blue(), "Projects", "22"),
                (
                    "Downloads",
                    state.theme.category_emerald(),
                    "Downloads",
                    "10",
                ),
                (
                    "Media Assets",
                    state.theme.category_purple(),
                    "Pictures",
                    "8",
                ),
            ];

            let mut sub_col = column![].spacing(2);

            for (title, accent_color, path_filter, count_str) in subfolders {
                let is_selected = state.active_folder_filter.as_deref() == Some(path_filter);

                let color_indicator = container(Space::new())
                    .width(Length::Fixed(8.0))
                    .height(Length::Fixed(8.0))
                    .style(move |_theme| container::Style {
                        background: Some(Background::Color(accent_color)),
                        border: Border {
                            color: Color::TRANSPARENT,
                            width: 0.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    });

                let badge = container(text(count_str).size(10).color(state.theme.muted_color()))
                    .padding([1, 6])
                    .style(pill_badge_style(
                        state.theme.hover_color(),
                        state.theme.muted_color(),
                    ));

                let item_row = row![
                    Space::new().width(Length::Fixed(10.0)),
                    color_indicator,
                    text("📁").size(12),
                    text(title).size(12).color(if is_selected {
                        if is_dark {
                            colors::DARK_TEXT
                        } else {
                            colors::LIGHT_ACCENT
                        }
                    } else {
                        state.theme.text_color()
                    }),
                    Space::new().width(Length::Fill),
                    badge,
                ]
                .spacing(6)
                .align_y(Alignment::Center);

                let item_btn = button(item_row)
                    .width(Length::Fill)
                    .padding([5, 8])
                    .style(sidebar_item_style(&state.theme, is_selected))
                    .on_press(Message::SelectFolderFilter(Some(path_filter.to_string())));

                sub_col = sub_col.push(item_btn);
            }

            tree_col = tree_col.push(sub_col);
        }

        container(tree_col).padding([8, 0]).into()
    } else {
        Space::new().height(Length::Fixed(0.0)).into()
    };

    let scrollable_content =
        scrollable(column![nav_items, folder_tree_section].spacing(10)).height(Length::Fill);

    // -------------------------------------------------------------------------
    // 5. Footer Actions: Collapse & Theming
    // -------------------------------------------------------------------------
    let collapse_label = if state.sidebar_expanded {
        "⇤  Collapse"
    } else {
        "⇥"
    };

    let theme_label = if state.sidebar_expanded {
        if is_dark {
            "🌙  Dark Mode"
        } else {
            "☀️  Light Mode"
        }
    } else if is_dark {
        "🌙"
    } else {
        "☀️"
    };

    let collapse_btn = button(
        container(
            text(collapse_label)
                .size(12)
                .color(state.theme.subtext_color()),
        )
        .width(Length::Fill)
        .align_x(if state.sidebar_expanded {
            Alignment::Start
        } else {
            Alignment::Center
        }),
    )
    .width(Length::Fill)
    .padding([6, 10])
    .style(sidebar_item_style(&state.theme, false))
    .on_press(Message::ToggleSidebar);

    let theme_btn = button(
        container(
            text(theme_label)
                .size(12)
                .color(state.theme.subtext_color()),
        )
        .width(Length::Fill)
        .align_x(if state.sidebar_expanded {
            Alignment::Start
        } else {
            Alignment::Center
        }),
    )
    .width(Length::Fill)
    .padding([6, 10])
    .style(sidebar_item_style(&state.theme, false))
    .on_press(Message::ToggleTheme);

    let footer_section = column![collapse_btn, theme_btn].spacing(4);

    let sidebar_layout = column![
        brand_section,
        search_pill_section,
        scrollable_content,
        footer_section
    ]
    .spacing(12)
    .padding(12)
    .width(width)
    .height(Length::Fill);

    container(sidebar_layout)
        .width(width)
        .height(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(Background::Color(state.theme.sidebar_color())),
            border: Border {
                color: state.theme.border_color(),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}
