//! Modern Search view layout with Pinterest / Notion-style Breadcrumbs, Folders Hero Showcase, and Files Table

use crate::icons::{self, Icon};
use crate::message::Message;
use crate::state::{format_number, search_input_id, AppState};
use crate::theme::{colors, hero_card_style, pill_badge_style, search_input_style};
use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow};

/// Renders the revamped primary Search screen
pub fn view(state: &AppState) -> Element<'_, Message> {
    let is_dark = state.theme.is_dark();

    // -------------------------------------------------------------------------
    // 1. Breadcrumbs Bar
    // -------------------------------------------------------------------------
    let mut breadcrumbs_row = row![
        button(
            text("Index Roots")
                .size(12)
                .color(state.theme.subtext_color())
        )
        .style(move |_theme, _status| button::Style::default())
        .on_press(Message::SelectFolderFilter(None)),
        text("›").size(12).color(state.theme.muted_color()),
        button(
            text("General Knowledge")
                .size(12)
                .color(state.theme.subtext_color())
        )
        .style(move |_theme, _status| button::Style::default())
        .on_press(Message::SelectFolderFilter(None)),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    if let Some(ref active_folder) = state.active_folder_filter {
        breadcrumbs_row = breadcrumbs_row.push(text("›").size(12).color(state.theme.muted_color()));
        breadcrumbs_row = breadcrumbs_row.push(text(active_folder).size(12).color(if is_dark {
            colors::DARK_TEXT
        } else {
            colors::LIGHT_TEXT
        }));
    } else {
        breadcrumbs_row = breadcrumbs_row.push(text("›").size(12).color(state.theme.muted_color()));
        breadcrumbs_row = breadcrumbs_row.push(text("All Files").size(12).color(if is_dark {
            colors::DARK_TEXT
        } else {
            colors::LIGHT_TEXT
        }));
    }

    let breadcrumbs_bar = row![
        breadcrumbs_row,
        Space::new().width(Length::Fill),
        text(format!(
            "Indexed: {} files",
            format_number(state.indexed_files_count as u64)
        ))
        .size(11)
        .color(state.theme.muted_color()),
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill);

    // -------------------------------------------------------------------------
    // 2. Search Mode Pills ([Everything], [Content], [Semantic], [Hybrid])
    // -------------------------------------------------------------------------
    let make_mode_pill = |label: &'static str,
                          mode: findme_semantic::SearchMode,
                          active_mode: findme_semantic::SearchMode,
                          icon: Icon| {
        let is_active = mode == active_mode;
        button(
            row![
                icons::icon(
                    icon,
                    12.0,
                    if is_active {
                        Color::WHITE
                    } else {
                        state.theme.subtext_color()
                    },
                ),
                text(label).size(12).color(if is_active {
                    Color::WHITE
                } else {
                    state.theme.subtext_color()
                }),
            ]
            .spacing(5)
            .align_y(Alignment::Center),
        )
        .padding([4, 12])
        .style(move |_theme, status| {
            let bg = if is_active {
                Some(Background::Color(colors::LIGHT_ACCENT))
            } else {
                match status {
                    button::Status::Hovered | button::Status::Pressed => {
                        Some(Background::Color(state.theme.hover_color()))
                    }
                    _ => Some(Background::Color(state.theme.surface_color())),
                }
            };
            button::Style {
                background: bg,
                border: Border {
                    radius: 14.0.into(),
                    width: if is_active { 0.0 } else { 1.0 },
                    color: state.theme.border_color(),
                },
                ..Default::default()
            }
        })
        .on_press(Message::SetSearchMode(mode))
    };

    let mode_pills_row = row![
        make_mode_pill(
            "Everything",
            findme_semantic::SearchMode::Everything,
            state.search_mode,
            Icon::Search
        ),
        make_mode_pill(
            "Content",
            findme_semantic::SearchMode::Content,
            state.search_mode,
            Icon::FileText
        ),
        make_mode_pill(
            "Semantic",
            findme_semantic::SearchMode::Semantic,
            state.search_mode,
            Icon::Sliders
        ),
        make_mode_pill(
            "Hybrid",
            findme_semantic::SearchMode::Hybrid,
            state.search_mode,
            Icon::Refresh
        ),
        Space::new().width(Length::Fill),
        if state.search_mode != findme_semantic::SearchMode::Everything {
            text("✨ Local Quantized Vector AI Active (all-MiniLM-L6-v2)")
                .size(11)
                .color(colors::LIGHT_ACCENT)
        } else {
            text("⚡ Sub-5ms Exact Filename Index")
                .size(11)
                .color(state.theme.muted_color())
        }
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    // -------------------------------------------------------------------------
    // 3. Search Input Bar
    // -------------------------------------------------------------------------
    let search_input = text_input(
        "Search files, paths, or operators (e.g. ext:rs size:>1mb)...",
        &state.query,
    )
    .id(search_input_id())
    .on_input(Message::SearchInputChanged)
    .style(search_input_style(&state.theme))
    .padding([10, 16])
    .size(14)
    .width(Length::Fill);

    let mut search_row = row![search_input]
        .spacing(8)
        .align_y(Alignment::Center)
        .width(Length::Fill);

    if !state.query.is_empty() {
        let clear_btn = button(icons::icon(Icon::Cross, 12.0, state.theme.subtext_color()))
            .padding([8, 12])
            .style(move |_theme, status| {
                let bg = match status {
                    button::Status::Hovered | button::Status::Pressed => {
                        Some(Background::Color(state.theme.hover_color()))
                    }
                    _ => None,
                };
                button::Style {
                    background: bg,
                    border: Border {
                        radius: 20.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
            .on_press(Message::ClearSearch);

        search_row = search_row.push(clear_btn);
    }

    // -------------------------------------------------------------------------
    // 3. Quick-Access "Folders" Showcase Hero Grid
    // -------------------------------------------------------------------------
    let folders_title = text("Folders").size(18).color(state.theme.text_color());

    let find_root_count = |target: &str| -> Option<u64> {
        state
            .index_mgmt
            .roots
            .iter()
            .find(|r| {
                r.path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.eq_ignore_ascii_case(target))
                    .unwrap_or(false)
            })
            .map(|r| r.file_count)
    };

    let docs_subtitle = match find_root_count("Documents") {
        Some(c) => format!("{} files • PDF & Notes", crate::state::format_number(c)),
        None => "PDF & Notes".to_string(),
    };
    let dev_subtitle = match find_root_count("Projects") {
        Some(c) => format!("{} files • Rust & Python", crate::state::format_number(c)),
        None => "Rust & Python".to_string(),
    };
    let downloads_subtitle = match find_root_count("Downloads") {
        Some(c) => format!("{} files • Media & Zip", crate::state::format_number(c)),
        None => "Media & Zip".to_string(),
    };

    // Hero Card 1: Documents (Amber)
    let card1 = render_hero_card(
        &state.theme,
        "Documents",
        docs_subtitle,
        "Documents",
        state.theme.category_amber(),
        state.theme.tint_amber(),
        &["K", "M", "A"],
        "Docs",
        state.active_folder_filter.as_deref() == Some("Documents"),
    );

    // Hero Card 2: Code & Dev (Blue)
    let card2 = render_hero_card(
        &state.theme,
        "Code & Dev",
        dev_subtitle,
        "Projects",
        state.theme.category_blue(),
        state.theme.tint_blue(),
        &["D", "S"],
        "Dev",
        state.active_folder_filter.as_deref() == Some("Projects"),
    );

    // Hero Card 3: Downloads & Media (Purple)
    let card3 = render_hero_card(
        &state.theme,
        "Downloads & Media",
        downloads_subtitle,
        "Downloads",
        state.theme.category_purple(),
        state.theme.tint_purple(),
        &["M", "A"],
        "Media",
        state.active_folder_filter.as_deref() == Some("Downloads"),
    );

    let hero_grid = row![card1, card2, card3].spacing(16).width(Length::Fill);

    let folders_section = column![folders_title, hero_grid].spacing(10);

    // -------------------------------------------------------------------------
    // 4. "Files" Results Table Header & Stats
    // -------------------------------------------------------------------------
    let files_title = text("Files").size(18).color(state.theme.text_color());

    let matches_text = if state.query.is_empty() {
        format!(
            "Showing {} files",
            format_number(state.results.len() as u64)
        )
    } else {
        format!(
            "\"{}\" • {} matches ({} ms)",
            state.query,
            format_number(state.results.len() as u64),
            state.last_search_duration_ms
        )
    };

    let files_header = row![
        files_title,
        Space::new().width(Length::Fixed(12.0)),
        text(matches_text).size(12).color(state.theme.muted_color()),
        Space::new().width(Length::Fill),
        row![
            if state.is_searching {
                icons::icon(Icon::Refresh, 12.0, colors::LIGHT_ACCENT)
            } else {
                icons::icon(Icon::Folder, 12.0, state.theme.muted_color())
            },
            text(if state.is_searching {
                "Searching index..."
            } else {
                "Ready"
            })
            .size(11)
            .color(if state.is_searching {
                colors::LIGHT_ACCENT
            } else {
                state.theme.muted_color()
            }),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill);

    // Results table view (virtualized 60 FPS table with sort headers)
    let results_table = crate::views::table::view(state);

    // -------------------------------------------------------------------------
    // 5. Minimal Bottom Footer Status Bar
    // -------------------------------------------------------------------------
    let status_bar = row![
        text("Shortcuts: Ctrl+K / Ctrl+F Search • Ctrl+B Sidebar • Ctrl+T Theme • Enter Open • Alt+Enter Properties")
            .size(11)
            .color(state.theme.muted_color()),
        Space::new().width(Length::Fill),
        text("Find-Me Linux • Pure Rust Offline Engine")
            .size(11)
            .color(state.theme.muted_color()),
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill);

    column![
        breadcrumbs_bar,
        mode_pills_row,
        search_row,
        folders_section,
        files_header,
        results_table,
        status_bar,
    ]
    .spacing(12)
    .padding(20)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Helper function to render a 3D-styled Folder Hero Showcase Card
#[allow(clippy::too_many_arguments)]
fn render_hero_card<'a>(
    theme: &'a crate::theme::AppTheme,
    title: &'static str,
    subtitle: String,
    filter_target: &'static str,
    accent_color: Color,
    tint_color: Color,
    avatars: &[&'static str],
    tag: &'static str,
    is_active: bool,
) -> Element<'a, Message> {
    let is_dark = theme.is_dark();

    // Top illustration container with layered 3D folder visual
    let tag_badge = container(text(tag).size(10).color(Color::WHITE))
        .padding([2, 6])
        .style(pill_badge_style(
            accent_color.scale_alpha(0.85),
            Color::WHITE,
        ));

    let top_row = row![Space::new().width(Length::Fill), tag_badge,].align_y(Alignment::Center);

    // Layered 3D folder center graphic
    let center_graphic = row![
        icons::icon(Icon::FolderOpen, 36.0, accent_color),
        Space::new().width(Length::Fixed(6.0)),
        icons::icon(Icon::FileText, 24.0, accent_color.scale_alpha(0.7)),
    ]
    .align_y(Alignment::Center);

    // Avatar circles row
    let mut avatars_row = row![].spacing(4).align_y(Alignment::Center);
    for avatar in avatars {
        let avatar_circle = container(text(*avatar).size(9).color(Color::WHITE))
            .width(Length::Fixed(20.0))
            .height(Length::Fixed(20.0))
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .style(move |_theme| container::Style {
                background: Some(Background::Color(accent_color)),
                border: Border {
                    color: Color::WHITE,
                    width: 1.0,
                    radius: 10.0.into(),
                },
                ..Default::default()
            });

        avatars_row = avatars_row.push(avatar_circle);
    }

    let illustration_box = container(
        column![
            top_row,
            container(center_graphic)
                .width(Length::Fill)
                .align_x(Alignment::Center),
            avatars_row,
        ]
        .spacing(4)
        .padding(10)
        .height(Length::Fixed(92.0)),
    )
    .width(Length::Fill)
    .style(hero_card_style(theme, tint_color));

    let card_labels = column![
        text(title).size(14).color(if is_active {
            accent_color
        } else {
            theme.text_color()
        }),
        text(subtitle).size(11).color(theme.muted_color()),
    ]
    .spacing(2)
    .padding([4, 2]);

    let card_content = column![illustration_box, card_labels].spacing(6);

    let next_filter = if is_active {
        None
    } else {
        Some(filter_target.to_string())
    };

    button(card_content)
        .width(Length::FillPortion(1))
        .padding(8)
        .style(move |_theme, status| {
            let border_color = if is_active {
                accent_color
            } else {
                match status {
                    button::Status::Hovered | button::Status::Pressed => {
                        accent_color.scale_alpha(0.5)
                    }
                    _ => Color::TRANSPARENT,
                }
            };

            button::Style {
                background: Some(Background::Color(if is_dark {
                    tint_color
                } else {
                    colors::LIGHT_SURFACE
                })),
                border: Border {
                    color: border_color,
                    width: if is_active { 1.5 } else { 1.0 },
                    radius: 14.0.into(),
                },
                shadow: Shadow::default(),
                ..Default::default()
            }
        })
        .on_press(Message::SelectFolderFilter(next_filter))
        .into()
}
