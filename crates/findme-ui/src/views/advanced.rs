//! Advanced Search visual filter builder screen with granular extension matrix

use crate::advanced::{DatePreset, ExtensionCategory, SizeUnit};
use crate::icons::{self, Icon};
use crate::message::{AdvancedSearchMessage, Message};
use crate::state::AppState;
use crate::theme::{self, colors};
use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Color, Element, Length};

/// Renders the Advanced Search query builder
pub fn view(state: &AppState) -> Element<'_, Message> {
    let filters = &state.advanced_filters;
    let synthesized = filters.synthesize_query();

    let query_preview_text = if synthesized.is_empty() {
        "(No active filters — matches all files)".to_string()
    } else {
        synthesized.clone()
    };

    let query_preview_container = container(text(query_preview_text).size(13))
        .padding(10)
        .style(theme::subtle_card_style(&state.theme))
        .width(Length::Fill);

    let query_card = container(
        column![
            row![
                column![
                    row![
                        icons::icon(Icon::Sliders, 18.0, colors::ACCENT_AMBER),
                        text("Advanced Search & Visual Query Builder").size(18),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                    text("Construct precise multi-criteria file search queries without memorizing syntax")
                        .size(12),
                ]
                .spacing(4),
                Space::new().width(Length::Fill),
                button(
                    row![
                        icons::icon(Icon::Refresh, 12.0, state.theme.text_color()),
                        text("Reset Filters").size(12),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .padding([5, 12])
                .style(theme::secondary_btn_style(&state.theme))
                .on_press(Message::AdvancedSearch(AdvancedSearchMessage::ResetFilters)),
                button(
                    row![
                        icons::icon(Icon::Search, 12.0, Color::WHITE),
                        text("Search Now").size(12),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .padding([5, 14])
                .style(theme::primary_btn_style(&state.theme))
                .on_press(Message::AdvancedSearch(AdvancedSearchMessage::SubmitSearch)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            query_preview_container,
        ]
        .spacing(10),
    )
    .padding(16)
    .style(theme::card_style(&state.theme))
    .width(Length::Fill);

    // 2. Keywords & Phrases Card (Chunked 2-Column Grid for Miller's Law UX)
    let col1 = column![
        column![
            text("All of these words:").size(12),
            text_input("e.g. project budget quarterly", &filters.all_words)
                .on_input(|s| Message::AdvancedSearch(AdvancedSearchMessage::UpdateAllWords(s)))
                .style(theme::search_input_style(&state.theme))
                .padding(8)
                .size(13),
        ]
        .spacing(4),
        column![
            text("Exact phrase:").size(12),
            text_input("e.g. annual financial report", &filters.exact_phrase)
                .on_input(|s| Message::AdvancedSearch(AdvancedSearchMessage::UpdateExactPhrase(s)))
                .style(theme::search_input_style(&state.theme))
                .padding(8)
                .size(13),
        ]
        .spacing(4),
    ]
    .spacing(10)
    .width(Length::FillPortion(1));

    let col2 = column![
        column![
            text("Any of these words:").size(12),
            text_input("e.g. invoices receipts accounts", &filters.any_words)
                .on_input(|s| Message::AdvancedSearch(AdvancedSearchMessage::UpdateAnyWords(s)))
                .style(theme::search_input_style(&state.theme))
                .padding(8)
                .size(13),
        ]
        .spacing(4),
        column![
            text("None of these words:").size(12),
            text_input("e.g. draft temp backup", &filters.none_words)
                .on_input(|s| Message::AdvancedSearch(AdvancedSearchMessage::UpdateNoneWords(s)))
                .style(theme::search_input_style(&state.theme))
                .padding(8)
                .size(13),
        ]
        .spacing(4),
    ]
    .spacing(10)
    .width(Length::FillPortion(1));

    let keywords_grid = row![col1, col2].spacing(16);

    let keywords_card =
        container(column![text("Keywords & Phrases").size(14), keywords_grid,].spacing(12))
            .padding(16)
            .style(theme::card_style(&state.theme))
            .width(Length::Fill);

    // 3. Granular Extension Matrix Card
    let matrix_header = row![
        row![
            icons::icon(Icon::Sliders, 16.0, colors::LIGHT_ACCENT),
            text("File Types & Granular Extension Matrix").size(15),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
        Space::new().width(Length::Fill),
        text("Select specific file formats or categories (e.g. ogg, flac, rs, py)").size(11),
    ]
    .align_y(Alignment::Center);

    // Active Extensions Ribbon (visible when any extensions are active)
    let active_exts_count = filters.active_extensions.len();
    let ribbon_section: Option<Element<'_, Message>> = if active_exts_count > 0 {
        let mut chips_row =
            row![text(format!("Active Formats ({}):", active_exts_count)).size(11),]
                .spacing(6)
                .align_y(Alignment::Center);

        for ext in &filters.active_extensions {
            let chip = container(
                row![
                    text(format!(".{}", ext)).size(11),
                    button(icons::icon(Icon::Cross, 10.0, colors::LIGHT_DANGER))
                        .padding(2)
                        .style(theme::danger_btn_style(&state.theme))
                        .on_press(Message::AdvancedSearch(
                            AdvancedSearchMessage::ToggleExtension(ext.clone())
                        )),
                ]
                .spacing(4)
                .align_y(Alignment::Center),
            )
            .padding([2, 6])
            .style(theme::subtle_card_style(&state.theme));

            chips_row = chips_row.push(chip);
        }

        chips_row = chips_row.push(Space::new().width(Length::Fill));
        chips_row = chips_row.push(
            button(
                row![
                    icons::icon(Icon::Trash, 11.0, colors::LIGHT_DANGER),
                    text("Clear All Formats").size(11),
                ]
                .spacing(4)
                .align_y(Alignment::Center),
            )
            .padding([3, 8])
            .style(theme::danger_btn_style(&state.theme))
            .on_press(Message::AdvancedSearch(
                AdvancedSearchMessage::ClearAllExtensions,
            )),
        );

        Some(
            container(chips_row)
                .padding([6, 10])
                .style(theme::hero_card_style(
                    &state.theme,
                    if state.theme.is_dark() {
                        colors::DARK_HOVER
                    } else {
                        colors::TINT_BLUE
                    },
                ))
                .width(Length::Fill)
                .into(),
        )
    } else {
        None
    };

    // Category Tabs Row
    let mut category_tabs_row = row![].spacing(6).align_y(Alignment::Center);
    for cat in ExtensionCategory::ALL {
        let is_active = filters.active_ext_category == cat;
        let btn = button(text(cat.title()).size(11))
            .padding([5, 12])
            .style(theme::chip_btn_style(&state.theme, is_active))
            .on_press(Message::AdvancedSearch(
                AdvancedSearchMessage::SelectExtCategory(cat),
            ));
        category_tabs_row = category_tabs_row.push(btn);
    }

    // Format Chips for Selected Category
    let current_cat = filters.active_ext_category;
    let format_actions_row = row![
        text(format!("{} Formats:", current_cat.title())).size(12),
        Space::new().width(Length::Fill),
        button(text("Select All in Category").size(11))
            .padding([3, 8])
            .style(theme::secondary_btn_style(&state.theme))
            .on_press(Message::AdvancedSearch(
                AdvancedSearchMessage::SelectAllInCategory(current_cat),
            )),
        button(text("Clear Category").size(11))
            .padding([3, 8])
            .style(theme::secondary_btn_style(&state.theme))
            .on_press(Message::AdvancedSearch(
                AdvancedSearchMessage::ClearCategory(current_cat),
            )),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let mut format_chips_row = row![].spacing(6).align_y(Alignment::Center);
    for &ext in current_cat.extensions() {
        let is_selected = filters.is_extension_active(ext);
        let label = if is_selected {
            format!("✓ .{}", ext)
        } else {
            format!(".{}", ext)
        };

        let btn = button(text(label).size(11))
            .padding([4, 10])
            .style(theme::chip_btn_style(&state.theme, is_selected))
            .on_press(Message::AdvancedSearch(
                AdvancedSearchMessage::ToggleExtension(ext.to_string()),
            ));
        format_chips_row = format_chips_row.push(btn);
    }

    // Custom Extension Tag Adder
    let custom_ext_row = row![
        text_input(
            "Add custom format (e.g. parquet, sqlite3, toml, env, sh)",
            &filters.custom_ext_input
        )
        .on_input(|s| Message::AdvancedSearch(AdvancedSearchMessage::UpdateCustomExtInput(s)))
        .style(theme::search_input_style(&state.theme))
        .padding(8)
        .size(12)
        .width(Length::Fill),
        button(
            row![
                icons::icon(Icon::Plus, 11.0, Color::WHITE),
                text("Add Format").size(11),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        )
        .padding([6, 12])
        .style(theme::primary_btn_style(&state.theme))
        .on_press(Message::AdvancedSearch(
            AdvancedSearchMessage::AddCustomExtFromInput,
        )),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let mut matrix_col = column![matrix_header,].spacing(10);

    if let Some(ribbon) = ribbon_section {
        matrix_col = matrix_col.push(ribbon);
    }

    matrix_col = matrix_col
        .push(category_tabs_row)
        .push(format_actions_row)
        .push(format_chips_row)
        .push(custom_ext_row);

    let ext_card = container(matrix_col)
        .padding(16)
        .style(theme::card_style(&state.theme))
        .width(Length::Fill);

    // 4. File Size Bounds Card
    let mut min_unit_buttons = row![].spacing(6);
    for unit in SizeUnit::ALL {
        let is_active = filters.size_min_unit == unit;
        let btn = button(text(unit.as_str()).size(11))
            .padding([4, 10])
            .style(theme::chip_btn_style(&state.theme, is_active))
            .on_press(Message::AdvancedSearch(
                AdvancedSearchMessage::SelectSizeMinUnit(unit),
            ));
        min_unit_buttons = min_unit_buttons.push(btn);
    }

    let min_size_row = row![
        text("Minimum Size:").size(12).width(Length::Fixed(140.0)),
        text_input("e.g. 10", &filters.size_min)
            .on_input(|s| Message::AdvancedSearch(AdvancedSearchMessage::UpdateSizeMin(s)))
            .style(theme::search_input_style(&state.theme))
            .padding(8)
            .size(13)
            .width(Length::Fixed(120.0)),
        min_unit_buttons,
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let mut max_unit_buttons = row![].spacing(6);
    for unit in SizeUnit::ALL {
        let is_active = filters.size_max_unit == unit;
        let btn = button(text(unit.as_str()).size(11))
            .padding([4, 10])
            .style(theme::chip_btn_style(&state.theme, is_active))
            .on_press(Message::AdvancedSearch(
                AdvancedSearchMessage::SelectSizeMaxUnit(unit),
            ));
        max_unit_buttons = max_unit_buttons.push(btn);
    }

    let max_size_row = row![
        text("Maximum Size:").size(12).width(Length::Fixed(140.0)),
        text_input("e.g. 500", &filters.size_max)
            .on_input(|s| Message::AdvancedSearch(AdvancedSearchMessage::UpdateSizeMax(s)))
            .style(theme::search_input_style(&state.theme))
            .padding(8)
            .size(13)
            .width(Length::Fixed(120.0)),
        max_unit_buttons,
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let size_card = container(
        column![
            text("File Size Bounds").size(14),
            min_size_row,
            max_size_row,
        ]
        .spacing(12),
    )
    .padding(16)
    .style(theme::card_style(&state.theme))
    .width(Length::Fill);

    // 5. Modification Date & Scope Constraints Card
    let mut date_buttons = row![].spacing(6);
    for preset in DatePreset::ALL {
        let is_active = filters.date_preset == preset;
        let btn = button(text(preset.title()).size(11))
            .padding([5, 12])
            .style(theme::chip_btn_style(&state.theme, is_active))
            .on_press(Message::AdvancedSearch(
                AdvancedSearchMessage::SelectDatePreset(preset),
            ));
        date_buttons = date_buttons.push(btn);
    }

    let paths_col = row![
        column![
            text("Search within folder:").size(12),
            text_input("e.g. /home/user/projects or /media", &filters.search_path)
                .on_input(|s| Message::AdvancedSearch(AdvancedSearchMessage::UpdateSearchPath(s)))
                .style(theme::search_input_style(&state.theme))
                .padding(8)
                .size(12),
        ]
        .spacing(4)
        .width(Length::FillPortion(1)),
        column![
            text("Exclude folder:").size(12),
            text_input("e.g. node_modules or target", &filters.exclude_path)
                .on_input(|s| Message::AdvancedSearch(AdvancedSearchMessage::UpdateExcludePath(s)))
                .style(theme::search_input_style(&state.theme))
                .padding(8)
                .size(12),
        ]
        .spacing(4)
        .width(Length::FillPortion(1)),
    ]
    .spacing(16);

    let flags_row = row![
        checkbox(filters.match_case)
            .label("Match Case (case:)")
            .text_size(12)
            .on_toggle(|b| Message::AdvancedSearch(AdvancedSearchMessage::ToggleMatchCase(b))),
        checkbox(filters.match_whole_word)
            .label("Match Whole Word")
            .text_size(12)
            .on_toggle(|b| Message::AdvancedSearch(AdvancedSearchMessage::ToggleMatchWholeWord(b))),
        checkbox(filters.regex_mode)
            .label("Regular Expression (regex:)")
            .text_size(12)
            .on_toggle(|b| Message::AdvancedSearch(AdvancedSearchMessage::ToggleRegexMode(b))),
    ]
    .spacing(20);

    let scope_card = container(
        column![
            text("Modification Date & Search Scope").size(14),
            date_buttons,
            paths_col,
            flags_row,
        ]
        .spacing(12),
    )
    .padding(16)
    .style(theme::card_style(&state.theme))
    .width(Length::Fill);

    let main_content = column![query_card, keywords_card, ext_card, size_card, scope_card]
        .spacing(14)
        .padding(16);

    scrollable(main_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
