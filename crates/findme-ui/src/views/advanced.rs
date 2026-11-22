//! Advanced Search visual filter builder screen

use crate::advanced::{DatePreset, PresetFilter, SizeUnit};
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

    // 1. Header & Live Query Preview Card
    let header_title = column![
        row![
            icons::icon(Icon::Sliders, 18.0, colors::ACCENT_AMBER),
            text("Advanced Search & Visual Query Builder").size(18),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        text("Construct precise multi-criteria file search queries without memorizing syntax")
            .size(12),
    ]
    .spacing(4);

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
                text("Generated Query Syntax:").size(13),
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

    // 2. Keywords & Text Matching Card
    let keywords_card = container(
        column![
            text("Keywords & Phrases").size(14),
            row![
                text("All of these words:")
                    .size(12)
                    .width(Length::Fixed(160.0)),
                text_input("e.g. project budget quarterly", &filters.all_words)
                    .on_input(|s| Message::AdvancedSearch(AdvancedSearchMessage::UpdateAllWords(s)))
                    .style(theme::search_input_style(&state.theme))
                    .padding(8)
                    .size(13)
                    .width(Length::Fill),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![
                text("Exact phrase:").size(12).width(Length::Fixed(160.0)),
                text_input("e.g. annual financial report", &filters.exact_phrase)
                    .on_input(|s| Message::AdvancedSearch(
                        AdvancedSearchMessage::UpdateExactPhrase(s)
                    ))
                    .style(theme::search_input_style(&state.theme))
                    .padding(8)
                    .size(13)
                    .width(Length::Fill),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![
                text("Any of these words:")
                    .size(12)
                    .width(Length::Fixed(160.0)),
                text_input("e.g. invoices receipts accounts", &filters.any_words)
                    .on_input(|s| Message::AdvancedSearch(AdvancedSearchMessage::UpdateAnyWords(s)))
                    .style(theme::search_input_style(&state.theme))
                    .padding(8)
                    .size(13)
                    .width(Length::Fill),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![
                text("None of these words:")
                    .size(12)
                    .width(Length::Fixed(160.0)),
                text_input("e.g. draft temp backup", &filters.none_words)
                    .on_input(
                        |s| Message::AdvancedSearch(AdvancedSearchMessage::UpdateNoneWords(s))
                    )
                    .style(theme::search_input_style(&state.theme))
                    .padding(8)
                    .size(13)
                    .width(Length::Fill),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(12),
    )
    .padding(16)
    .style(theme::card_style(&state.theme))
    .width(Length::Fill);

    // 3. File Types & Extensions Card
    let ext_checkboxes_row1 = row![
        checkbox(filters.ext_documents)
            .label("Documents (pdf, doc, txt, md)")
            .text_size(12)
            .on_toggle(|b| Message::AdvancedSearch(AdvancedSearchMessage::ToggleExtDocuments(b))),
        checkbox(filters.ext_code)
            .label("Source Code (rs, c, cpp, py, js, go)")
            .text_size(12)
            .on_toggle(|b| Message::AdvancedSearch(AdvancedSearchMessage::ToggleExtCode(b))),
        checkbox(filters.ext_images)
            .label("Images (png, jpg, svg, gif)")
            .text_size(12)
            .on_toggle(|b| Message::AdvancedSearch(AdvancedSearchMessage::ToggleExtImages(b))),
    ]
    .spacing(16);

    let ext_checkboxes_row2 = row![
        checkbox(filters.ext_audio)
            .label("Audio (mp3, wav, flac, ogg)")
            .text_size(12)
            .on_toggle(|b| Message::AdvancedSearch(AdvancedSearchMessage::ToggleExtAudio(b))),
        checkbox(filters.ext_video)
            .label("Video (mp4, mkv, avi, webm)")
            .text_size(12)
            .on_toggle(|b| Message::AdvancedSearch(AdvancedSearchMessage::ToggleExtVideo(b))),
        checkbox(filters.ext_archives)
            .label("Archives (zip, tar, gz, xz, 7z)")
            .text_size(12)
            .on_toggle(|b| Message::AdvancedSearch(AdvancedSearchMessage::ToggleExtArchives(b))),
    ]
    .spacing(16);

    let custom_ext_row = row![
        text("Custom extensions:")
            .size(12)
            .width(Length::Fixed(160.0)),
        text_input("e.g. toml, json, yaml, sql", &filters.custom_ext)
            .on_input(|s| Message::AdvancedSearch(AdvancedSearchMessage::UpdateCustomExt(s)))
            .style(theme::search_input_style(&state.theme))
            .padding(8)
            .size(13)
            .width(Length::Fill),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let ext_card = container(
        column![
            text("File Types & Extensions").size(14),
            ext_checkboxes_row1,
            ext_checkboxes_row2,
            custom_ext_row,
        ]
        .spacing(12),
    )
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
        text("Minimum Size:").size(12).width(Length::Fixed(160.0)),
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
        text("Maximum Size:").size(12).width(Length::Fixed(160.0)),
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

    // 5. Modification Date Card
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

    let date_card =
        container(column![text("Modification Date").size(14), date_buttons,].spacing(12))
            .padding(16)
            .style(theme::card_style(&state.theme))
            .width(Length::Fill);

    // 6. Target Location & Exclusions Card
    let search_path_row = row![
        text("Search inside folder:")
            .size(12)
            .width(Length::Fixed(160.0)),
        text_input(
            "e.g. /home/user/projects or ~/Documents",
            &filters.search_path
        )
        .on_input(|s| Message::AdvancedSearch(AdvancedSearchMessage::UpdateSearchPath(s)))
        .style(theme::search_input_style(&state.theme))
        .padding(8)
        .size(13)
        .width(Length::Fill),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let exclude_path_row = row![
        text("Exclude folder:").size(12).width(Length::Fixed(160.0)),
        text_input(
            "e.g. target or node_modules or .cache",
            &filters.exclude_path
        )
        .on_input(|s| Message::AdvancedSearch(AdvancedSearchMessage::UpdateExcludePath(s)))
        .style(theme::search_input_style(&state.theme))
        .padding(8)
        .size(13)
        .width(Length::Fill),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let path_card = container(
        column![
            text("Target Location & Exclusions").size(14),
            search_path_row,
            exclude_path_row,
        ]
        .spacing(12),
    )
    .padding(16)
    .style(theme::card_style(&state.theme))
    .width(Length::Fill);

    // 7. Search Options Card
    let options_row = row![
        checkbox(filters.match_case)
            .label("Match Case (case:)")
            .text_size(12)
            .on_toggle(|b| Message::AdvancedSearch(AdvancedSearchMessage::ToggleMatchCase(b))),
        checkbox(filters.match_whole_word)
            .label("Match Whole Word")
            .text_size(12)
            .on_toggle(|b| Message::AdvancedSearch(AdvancedSearchMessage::ToggleMatchWholeWord(b))),
        checkbox(filters.regex_mode)
            .label("Regular Expression Mode (regex:)")
            .text_size(12)
            .on_toggle(|b| Message::AdvancedSearch(AdvancedSearchMessage::ToggleRegexMode(b))),
    ]
    .spacing(20);

    let options_card =
        container(column![text("Search Options & Modes").size(14), options_row,].spacing(12))
            .padding(16)
            .style(theme::card_style(&state.theme))
            .width(Length::Fill);

    // 8. 1-Click Quick Template Presets
    let template_btn = |label: &'static str, preset: PresetFilter| -> Element<'_, Message> {
        button(
            row![
                icons::icon(Icon::Sparkles, 11.0, colors::ACCENT_AMBER),
                text(label).size(11),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .padding([5, 12])
        .style(theme::secondary_btn_style(&state.theme))
        .on_press(Message::AdvancedSearch(AdvancedSearchMessage::ApplyPreset(
            preset,
        )))
        .into()
    };

    let templates_row = row![
        template_btn("Large Code (> 1 MB)", PresetFilter::LargeCode),
        template_btn("Recent Docs (Past Week)", PresetFilter::RecentDocuments),
        template_btn("Gigantic Files (> 1 GB)", PresetFilter::GiganticFiles),
        template_btn("Archives Only", PresetFilter::ArchivesOnly),
    ]
    .spacing(8);

    let templates_card =
        container(column![text("1-Click Preset Templates").size(14), templates_row,].spacing(12))
            .padding(16)
            .style(theme::card_style(&state.theme))
            .width(Length::Fill);

    let form_content = column![
        header_title,
        query_card,
        keywords_card,
        ext_card,
        size_card,
        date_card,
        path_card,
        options_card,
        templates_card,
    ]
    .spacing(14)
    .padding(16);

    scrollable(form_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
