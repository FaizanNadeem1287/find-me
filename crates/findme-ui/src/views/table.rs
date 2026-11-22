//! High-performance virtualized 60 FPS tabular results view and column header sorting
//! Revamped with modern Pinterest / Notion aesthetic, generous row padding, file icons, and metadata badges.

use crate::icons::{self, Icon};
use crate::message::{ColumnId, Message};
use crate::state::{compute_virtual_range, table_scrollable_id, AppState};
use crate::theme::{self, colors, pill_badge_style, sidebar_item_style, table_row_btn_style};
use findme_core::SearchResultItem;
use iced::widget::{button, column, container, mouse_area, row, scrollable, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length};

/// Row height in pixels (generous row padding for modern Notion / Pinterest aesthetic)
pub const ROW_HEIGHT: f32 = 36.0;

/// Header height in pixels
pub const HEADER_HEIGHT: f32 = 32.0;

/// Number of buffer rows rendered above and below viewport to prevent flickering
pub const OVERSCAN_ROWS: usize = 5;

/// Extracts extension or general category string from a search result item
pub fn item_type_str(item: &SearchResultItem) -> &'static str {
    if item.is_dir {
        "Folder"
    } else if let Some(idx) = item.name.rfind('.') {
        if idx > 0 && idx + 1 < item.name.len() {
            match item.name[idx + 1..].to_ascii_lowercase().as_str() {
                "rs" => "Rust",
                "toml" => "TOML",
                "json" => "JSON",
                "md" => "Markdown",
                "txt" => "Text",
                "pdf" => "PDF",
                "png" => "PNG Image",
                "jpg" | "jpeg" => "JPEG Image",
                "zip" | "tar" | "gz" | "xz" => "Archive",
                "sh" => "Shell Script",
                "deb" => "Debian Pkg",
                _ => "File",
            }
        } else {
            "File"
        }
    } else {
        "File"
    }
}

/// Returns modern vector icon and category display string for a search result item
pub fn item_vector_icon(item: &SearchResultItem) -> (Icon, &'static str) {
    if item.is_dir {
        (Icon::Folder, "Folder")
    } else if let Some(idx) = item.name.rfind('.') {
        if idx > 0 && idx + 1 < item.name.len() {
            match item.name[idx + 1..].to_ascii_lowercase().as_str() {
                "rs" => (Icon::FileCode, "Rust"),
                "py" => (Icon::FileCode, "Python"),
                "js" | "ts" | "jsx" | "tsx" => (Icon::FileCode, "Web"),
                "c" | "cpp" | "h" | "hpp" => (Icon::FileCode, "C/C++"),
                "toml" => (Icon::FileCode, "TOML"),
                "yaml" | "yml" => (Icon::FileCode, "YAML"),
                "json" => (Icon::FileCode, "JSON"),
                "md" => (Icon::FileText, "Markdown"),
                "txt" => (Icon::FileText, "Text"),
                "pdf" => (Icon::FileText, "PDF"),
                "png" => (Icon::FileImage, "PNG Image"),
                "jpg" | "jpeg" => (Icon::FileImage, "JPEG Image"),
                "gif" | "svg" | "webp" => (Icon::FileImage, "Image"),
                "mp3" | "wav" | "flac" | "ogg" => (Icon::FileAudio, "Audio"),
                "mp4" | "mkv" | "avi" | "mov" => (Icon::FileVideo, "Video"),
                "zip" | "tar" | "gz" | "xz" => (Icon::FileArchive, "Archive"),
                "sh" => (Icon::FileCode, "Shell Script"),
                "deb" => (Icon::FileArchive, "Debian Pkg"),
                _ => (Icon::File, "File"),
            }
        } else {
            (Icon::File, "File")
        }
    } else {
        (Icon::File, "File")
    }
}

/// Renders the clickable sort header column button with modern indicator
fn column_header<'a>(
    theme: &'a crate::theme::AppTheme,
    title: &'static str,
    col_id: ColumnId,
    active_col: ColumnId,
    ascending: bool,
    fill_portion: u16,
) -> Element<'a, Message> {
    let arrow = if active_col == col_id {
        if ascending {
            " ▲"
        } else {
            " ▼"
        }
    } else {
        ""
    };

    let label = format!("{title}{arrow}");
    button(text(label).size(11).color(if active_col == col_id {
        if theme.is_dark() {
            colors::DARK_TEXT
        } else {
            colors::LIGHT_ACCENT
        }
    } else {
        theme.muted_color()
    }))
    .padding([6, 10])
    .width(Length::FillPortion(fill_portion))
    .style(sidebar_item_style(theme, false))
    .on_press(Message::SortByColumn(col_id))
    .into()
}

/// Renders the complete virtualized tabular results component
pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let is_dark = state.theme.is_dark();
    let asc = state.sort_ascending;
    let active_col = state.sort_column;

    // 1. Column header row with sort direction indicators
    let header = row![
        column_header(&state.theme, "Name", ColumnId::Name, active_col, asc, 4),
        column_header(&state.theme, "Path", ColumnId::Path, active_col, asc, 5),
        column_header(&state.theme, "Size", ColumnId::Size, active_col, asc, 2),
        column_header(
            &state.theme,
            "Date Modified",
            ColumnId::Modified,
            active_col,
            asc,
            2
        ),
        column_header(&state.theme, "Type", ColumnId::Type, active_col, asc, 1),
        Space::new().width(Length::Fixed(28.0)),
    ]
    .spacing(4)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let header_container = container(header).padding([2, 4]).width(Length::Fill);

    // 2. Results table body (virtualized or empty placeholder)
    let body: Element<'a, Message> = if state.results.is_empty() {
        let empty_msg = if state.query.is_empty() {
            "Index is ready. Type in search bar to find files instantaneously."
        } else {
            "No matching files found. Check filter syntax (e.g. ext:rs, size:>10mb, dm:today) or use fuzzy: prefix."
        };

        container(
            column![
                row![
                    icons::icon(Icon::Search, 16.0, state.theme.muted_color()),
                    text("No Results Found").size(15),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                text(empty_msg).size(12).color(state.theme.muted_color()),
            ]
            .spacing(6)
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    } else {
        let total_count = state.results.len();
        let (start_idx, end_idx, top_spacer, bottom_spacer) = compute_virtual_range(
            total_count,
            state.scroll_y,
            state.table_viewport_height,
            ROW_HEIGHT,
            OVERSCAN_ROWS,
        );

        let mut rows = column![].spacing(2).width(Length::Fill);

        if top_spacer > 0.0 {
            rows = rows.push(Space::new().height(top_spacer));
        }

        for (offset, item) in state.results[start_idx..end_idx].iter().enumerate() {
            let row_idx = start_idx + offset;
            let is_selected = state.selected_row == Some(row_idx);

            let (vec_icon, type_label) = item_vector_icon(item);
            let size_text = item.formatted_size();
            let mtime_text = item.formatted_mtime();

            let semantic_hit = state
                .semantic_results
                .iter()
                .find(|h| h.path.to_string_lossy() == item.path);

            let type_badge = if let Some(hit) = semantic_hit {
                let (badge_text, badge_color) = match hit.match_kind {
                    findme_semantic::MatchKind::SemanticMatch => (
                        format!("✨ {:.0}% Sem", hit.score * 100.0),
                        colors::LIGHT_ACCENT,
                    ),
                    findme_semantic::MatchKind::ExactContent => {
                        ("📄 Content".to_string(), colors::TINT_BLUE)
                    }
                    findme_semantic::MatchKind::AudioTranscript => {
                        ("🎙️ Audio".to_string(), colors::TINT_AMBER)
                    }
                    findme_semantic::MatchKind::VisionMatch => {
                        ("🖼️ Vision".to_string(), colors::TINT_PURPLE)
                    }
                    findme_semantic::MatchKind::HybridFusion => {
                        ("⚡ Hybrid".to_string(), colors::LIGHT_ACCENT)
                    }
                };
                container(text(badge_text).size(10).color(Color::WHITE))
                    .padding([1, 6])
                    .style(pill_badge_style(badge_color, Color::WHITE))
            } else {
                container(text(type_label).size(10).color(state.theme.muted_color()))
                    .padding([1, 6])
                    .style(pill_badge_style(
                        state.theme.hover_color(),
                        state.theme.muted_color(),
                    ))
            };

            let size_badge = container(text(size_text).size(10).color(state.theme.muted_color()))
                .padding([1, 6])
                .style(pill_badge_style(
                    state.theme.hover_color(),
                    state.theme.muted_color(),
                ));

            let menu_btn = button(text("⋮").size(13).color(state.theme.muted_color()))
                .padding([2, 6])
                .style(sidebar_item_style(&state.theme, false))
                .on_press(Message::OpenContextMenu {
                    item: item.clone(),
                    x: 0.0,
                    y: 0.0,
                });

            let name_cell = row![
                icons::icon(
                    vec_icon,
                    13.0,
                    if is_selected {
                        if is_dark {
                            colors::DARK_TEXT
                        } else {
                            colors::LIGHT_ACCENT
                        }
                    } else {
                        state.theme.muted_color()
                    }
                ),
                text(&item.name).size(12).color(if is_selected {
                    if is_dark {
                        colors::DARK_TEXT
                    } else {
                        colors::LIGHT_ACCENT
                    }
                } else {
                    state.theme.text_color()
                }),
            ]
            .spacing(6)
            .align_y(Alignment::Center)
            .width(Length::FillPortion(4));

            let path_cell = text(&item.path)
                .size(11)
                .color(state.theme.muted_color())
                .width(Length::FillPortion(5));

            let size_cell = container(size_badge)
                .width(Length::FillPortion(2))
                .align_x(Alignment::Start);

            let mtime_cell = text(mtime_text)
                .size(11)
                .color(state.theme.subtext_color())
                .width(Length::FillPortion(2));

            let type_cell = container(type_badge)
                .width(Length::FillPortion(1))
                .align_x(Alignment::Start);

            let row_cells = row![name_cell, path_cell, size_cell, mtime_cell, type_cell, menu_btn,]
                .spacing(6)
                .align_y(Alignment::Center)
                .width(Length::Fill);

            let mut row_column = column![row_cells].spacing(4).width(Length::Fill);

            if let Some(hit) = semantic_hit {
                if state.expanded_snippets.contains(&row_idx) || is_selected {
                    let mut snippet_row = row![
                        text(format!("↳ {}:", hit.location))
                            .size(11)
                            .color(colors::LIGHT_ACCENT),
                        text(&hit.best_snippet)
                            .size(11)
                            .color(state.theme.subtext_color()),
                        Space::new().width(Length::Fill),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center);

                    if let Some(ts) = hit.audio_timestamp {
                        let mins = (ts / 60.0).floor() as u64;
                        let secs = (ts % 60.0).floor() as u64;
                        let play_btn =
                            button(text(format!("▶ Play from {:02}:{:02}", mins, secs)).size(10))
                                .padding([2, 6])
                                .style(theme::primary_btn_style(&state.theme))
                                .on_press(Message::JumpAudioTimestamp {
                                    path: item.path.clone(),
                                    timestamp_secs: ts,
                                });
                        snippet_row = snippet_row.push(play_btn);
                    }

                    let snippet_container =
                        container(snippet_row).padding([3, 8]).style(move |_theme| {
                            container::Style {
                                background: Some(Background::Color(if is_dark {
                                    colors::DARK_SURFACE
                                } else {
                                    colors::LIGHT_SURFACE
                                })),
                                border: Border {
                                    radius: 4.0.into(),
                                    color: state.theme.border_color(),
                                    width: 1.0,
                                },
                                ..Default::default()
                            }
                        });

                    row_column = row_column.push(snippet_container);
                }
            }

            let row_btn = button(row_column)
                .padding([5, 8])
                .width(Length::Fill)
                .style(table_row_btn_style(&state.theme, is_selected))
                .on_press(Message::SelectRow(Some(row_idx)));

            let row_container =
                container(row_btn)
                    .width(Length::Fill)
                    .style(move |_theme| container::Style {
                        border: Border {
                            color: if is_selected {
                                if is_dark {
                                    colors::DARK_ACCENT
                                } else {
                                    colors::LIGHT_ACCENT
                                }
                            } else if is_dark {
                                colors::DARK_BORDER
                            } else {
                                colors::LIGHT_BORDER
                            },
                            width: if is_selected { 1.5 } else { 1.0 },
                            radius: 6.0.into(),
                        },
                        ..Default::default()
                    });

            let row_area = mouse_area(row_container)
                .on_right_press(Message::OpenContextMenu {
                    item: item.clone(),
                    x: 0.0,
                    y: 0.0,
                })
                .on_double_click(Message::OpenFile(item.path.clone()));

            rows = rows.push(row_area);
        }

        if bottom_spacer > 0.0 {
            rows = rows.push(Space::new().height(bottom_spacer));
        }

        scrollable(rows)
            .id(table_scrollable_id())
            .on_scroll(|viewport| Message::TableScrolled {
                scroll_y: viewport.absolute_offset().y,
                viewport_height: viewport.bounds().height,
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    };

    column![header_container, body]
        .spacing(4)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
