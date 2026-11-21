//! High-performance virtualized 60 FPS tabular results view and column header sorting

use crate::message::{ColumnId, Message};
use crate::state::{compute_virtual_range, table_scrollable_id, AppState};
use crate::theme::colors;
use findme_core::SearchResultItem;
use iced::widget::{button, column, container, mouse_area, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};

/// Row height in pixels
pub const ROW_HEIGHT: f32 = 30.0;

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
            // Check well-known extensions
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

/// Renders the clickable sort header column button
fn column_header<'a>(
    title: &'static str,
    col_id: ColumnId,
    active_col: ColumnId,
    ascending: bool,
    fill_portion: u16,
) -> Element<'a, Message> {
    let arrow = if active_col == col_id {
        if ascending {
            "▲ "
        } else {
            "▼ "
        }
    } else {
        "   "
    };

    let label = format!("{arrow}{title}");
    button(text(label).size(12))
        .padding([4, 8])
        .width(Length::FillPortion(fill_portion))
        .on_press(Message::SortByColumn(col_id))
        .into()
}

/// Renders the complete virtualized tabular results component
pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let asc = state.sort_ascending;
    let active_col = state.sort_column;

    // 1. Column header row with sort direction indicators
    let header = row![
        column_header("Name", ColumnId::Name, active_col, asc, 4),
        column_header("Path", ColumnId::Path, active_col, asc, 5),
        column_header("Size", ColumnId::Size, active_col, asc, 2),
        column_header("Date Modified", ColumnId::Modified, active_col, asc, 2),
        column_header("Type", ColumnId::Type, active_col, asc, 1),
        Space::new().width(Length::Fixed(28.0)),
    ]
    .spacing(4)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let header_container = container(header).padding([4, 6]).width(Length::Fill);

    // 2. Results table body (virtualized or empty placeholder)
    let body: Element<'a, Message> = if state.results.is_empty() {
        let empty_msg = if state.query.is_empty() {
            "Index is ready. Type in search bar to find files instantaneously."
        } else {
            "No matching files found. Check filter syntax (e.g. ext:rs, size:>10mb, dm:today)."
        };

        container(
            column![text("🔍 No Results").size(15), text(empty_msg).size(12),]
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

        let mut rows = column![].spacing(1).width(Length::Fill);

        if top_spacer > 0.0 {
            rows = rows.push(Space::new().height(top_spacer));
        }

        for (offset, item) in state.results[start_idx..end_idx].iter().enumerate() {
            let row_idx = start_idx + offset;
            let is_selected = state.selected_row == Some(row_idx);

            let icon = if item.is_dir { "📁 " } else { "📄 " };
            let name_text = format!("{icon}{}", item.name);
            let size_text = item.formatted_size();
            let mtime_text = item.formatted_mtime();
            let type_text = item_type_str(item);

            let menu_btn =
                button(text("⋮").size(13))
                    .padding([2, 6])
                    .on_press(Message::OpenContextMenu {
                        item: item.clone(),
                        x: 0.0,
                        y: 0.0,
                    });

            let row_cells = row![
                text(name_text).size(12).width(Length::FillPortion(4)),
                text(&item.path).size(11).width(Length::FillPortion(5)),
                text(size_text).size(11).width(Length::FillPortion(2)),
                text(mtime_text).size(11).width(Length::FillPortion(2)),
                text(type_text).size(11).width(Length::FillPortion(1)),
                menu_btn,
            ]
            .spacing(4)
            .align_y(Alignment::Center)
            .width(Length::Fill);

            let row_btn = button(row_cells)
                .padding([4, 6])
                .width(Length::Fill)
                .on_press(Message::SelectRow(Some(row_idx)));

            let styled_row = if is_selected {
                container(row_btn)
                    .width(Length::Fill)
                    .style(|_theme| container::Style {
                        background: Some(iced::Background::Color(
                            colors::DARK_ACCENT.scale_alpha(0.25),
                        )),
                        border: iced::border::rounded(3),
                        ..Default::default()
                    })
            } else {
                container(row_btn).width(Length::Fill)
            };

            let row_area = mouse_area(styled_row)
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
