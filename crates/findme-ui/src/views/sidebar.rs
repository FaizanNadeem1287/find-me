//! Modern responsive sidebar navigation component inspired by Notion / Pinterest file workspace aesthetic

use crate::icons::{self, Icon};
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

        let chevron = icons::icon(Icon::ChevronVertical, 11.0, state.theme.muted_color());

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
        let search_icon = icons::icon(Icon::Search, 12.0, state.theme.muted_color());
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
            container(icons::icon(Icon::Search, 14.0, state.theme.text_color()))
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
        let icon_col = if is_active {
            if is_dark {
                colors::DARK_TEXT
            } else {
                colors::LIGHT_ACCENT
            }
        } else {
            state.theme.muted_color()
        };

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
                        icons::icon(screen.vector_icon(), 14.0, icon_col),
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
                        icons::icon(screen.vector_icon(), 14.0, icon_col),
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
                    icons::icon(screen.vector_icon(), 14.0, icon_col),
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
                    container(icons::icon(Icon::Sparkles, 11.0, colors::ACCENT_AMBER))
                        .padding([1, 4]),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                _ => row![
                    icons::icon(screen.vector_icon(), 14.0, icon_col),
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
                container(icons::icon(screen.vector_icon(), 16.0, icon_col))
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
    // 4. Dynamic Indexed Directory Roots (When Expanded)
    // -------------------------------------------------------------------------
    let folder_tree_section: Element<'_, Message> = if state.sidebar_expanded {
        let tree_header = row![
            text("INDEXED DIRECTORIES")
                .size(10)
                .color(state.theme.muted_color()),
            Space::new().width(Length::Fill),
            button(
                row![
                    icons::icon(Icon::Plus, 10.0, state.theme.muted_color()),
                    text("Add").size(10).color(state.theme.muted_color())
                ]
                .spacing(3)
                .align_y(Alignment::Center)
            )
            .padding([2, 6])
            .style(sidebar_item_style(&state.theme, false))
            .on_press(Message::SwitchScreen(ScreenId::IndexManagement)),
        ]
        .align_y(Alignment::Center);

        let mut tree_col = column![tree_header].spacing(3);

        if let Some(ref active_filter) = state.active_folder_filter {
            let clear_btn = button(
                row![
                    icons::icon(Icon::Cross, 9.0, colors::ACCENT_AMBER),
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

        let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
        let accent_colors = [
            colors::ACCENT_PURPLE,
            colors::ACCENT_BLUE,
            colors::ACCENT_EMERALD,
            colors::ACCENT_AMBER,
            colors::ACCENT_ROSE,
        ];

        let mut roots_sub_col = column![].spacing(2);

        for (idx, root) in state.index_mgmt.roots.iter().enumerate() {
            let path_str = root.display_path();
            let is_selected = state.active_folder_filter.as_deref() == Some(&path_str);
            let accent_color = accent_colors[idx % accent_colors.len()];

            let display_name = if path_str == home_dir {
                "Home (~)".to_string()
            } else if path_str == "/" {
                "Root (/)".to_string()
            } else {
                root.path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| path_str.clone())
            };

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

            let count_label = if root.file_count > 0 {
                crate::state::format_number(root.file_count)
            } else {
                "0".to_string()
            };

            let badge = container(text(count_label).size(10).color(state.theme.muted_color()))
                .padding([1, 6])
                .style(pill_badge_style(
                    state.theme.hover_color(),
                    state.theme.muted_color(),
                ));

            let item_row = row![
                Space::new().width(Length::Fixed(6.0)),
                color_indicator,
                icons::icon(Icon::Folder, 12.0, accent_color),
                text(display_name).size(12).color(if is_selected {
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
            .spacing(8)
            .align_y(Alignment::Center);

            let next_filter = if is_selected {
                None
            } else {
                Some(path_str.clone())
            };

            let item_btn = button(item_row)
                .width(Length::Fill)
                .padding([5, 8])
                .style(sidebar_item_style(&state.theme, is_selected))
                .on_press(Message::SelectFolderFilter(next_filter));

            roots_sub_col = roots_sub_col.push(item_btn);
        }

        let manage_folders_btn = button(
            row![
                icons::icon(Icon::Folder, 11.0, state.theme.muted_color()),
                text("Manage Folders...")
                    .size(11)
                    .color(state.theme.muted_color()),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([4, 8])
        .style(sidebar_item_style(&state.theme, false))
        .on_press(Message::SwitchScreen(ScreenId::IndexManagement));

        roots_sub_col = roots_sub_col.push(manage_folders_btn);

        tree_col = tree_col.push(roots_sub_col);
        container(tree_col).padding([8, 0]).into()
    } else {
        button(
            container(icons::icon(Icon::Folder, 15.0, state.theme.muted_color()))
                .width(Length::Fill)
                .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([8, 0])
        .style(sidebar_item_style(&state.theme, false))
        .on_press(Message::SwitchScreen(ScreenId::IndexManagement))
        .into()
    };

    let scrollable_content =
        scrollable(column![nav_items, folder_tree_section].spacing(10)).height(Length::Fill);

    // -------------------------------------------------------------------------
    // 5. Footer Actions: Collapse & Theming
    // -------------------------------------------------------------------------
    let collapse_btn = if state.sidebar_expanded {
        button(
            row![
                icons::icon(Icon::Collapse, 13.0, state.theme.subtext_color()),
                text("Collapse").size(12).color(state.theme.subtext_color()),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([6, 10])
        .style(sidebar_item_style(&state.theme, false))
        .on_press(Message::ToggleSidebar)
    } else {
        button(
            container(icons::icon(Icon::Expand, 14.0, state.theme.subtext_color()))
                .width(Length::Fill)
                .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([6, 0])
        .style(sidebar_item_style(&state.theme, false))
        .on_press(Message::ToggleSidebar)
    };

    let theme_icon = if is_dark { Icon::Moon } else { Icon::Sun };
    let theme_label_text = if is_dark { "Dark Mode" } else { "Light Mode" };

    let theme_btn = if state.sidebar_expanded {
        button(
            row![
                icons::icon(theme_icon, 13.0, state.theme.subtext_color()),
                text(theme_label_text)
                    .size(12)
                    .color(state.theme.subtext_color()),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([6, 10])
        .style(sidebar_item_style(&state.theme, false))
        .on_press(Message::ToggleTheme)
    } else {
        button(
            container(icons::icon(theme_icon, 14.0, state.theme.subtext_color()))
                .width(Length::Fill)
                .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([6, 0])
        .style(sidebar_item_style(&state.theme, false))
        .on_press(Message::ToggleTheme)
    };

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
