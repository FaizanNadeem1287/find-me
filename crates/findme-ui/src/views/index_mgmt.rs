//! Index management and visual .gitignore-style exclusion rules screen

use crate::icons::{self, Icon};
use crate::index_mgmt::{ExclusionPreset, IndexingState};
use crate::message::{IndexManagementMessage, Message};
use crate::state::AppState;
use crate::theme::{self, colors};
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Color, Element, Length};

/// Renders the Index Management & Visual .gitignore Exclusion screen
pub fn view(state: &AppState) -> Element<'_, Message> {
    let mgmt = &state.index_mgmt;
    let stats = &mgmt.stats;

    // 1. Header & Actions Bar
    let header_title = column![
        row![
            icons::icon(Icon::Folder, 18.0, colors::LIGHT_ACCENT),
            text("Folder Management & Exclusion Rules").size(18),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        text("Configure monitored directory roots, manage visual .gitignore-style exclusion rules, and monitor memory.")
            .size(12),
    ]
    .spacing(4);

    let action_bar = row![
        button(
            row![
                icons::icon(Icon::Refresh, 12.0, Color::WHITE),
                text("Rebuild Index").size(12),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .padding([6, 14])
        .style(theme::primary_btn_style(&state.theme))
        .on_press(Message::IndexManagement(
            IndexManagementMessage::TriggerRebuildIndex,
        )),
        button(
            row![
                icons::icon(Icon::Refresh, 12.0, state.theme.text_color()),
                text("Refresh Index").size(12),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .padding([6, 14])
        .style(theme::secondary_btn_style(&state.theme))
        .on_press(Message::IndexManagement(
            IndexManagementMessage::TriggerRefreshIndex,
        )),
        button(
            row![
                icons::icon(Icon::Trash, 12.0, colors::LIGHT_DANGER),
                text("Clear Index").size(12),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .padding([6, 14])
        .style(theme::danger_btn_style(&state.theme))
        .on_press(Message::IndexManagement(IndexManagementMessage::ClearIndex)),
    ]
    .spacing(8);

    let header_card = container(
        row![header_title, Space::new().width(Length::Fill), action_bar,]
            .spacing(12)
            .align_y(Alignment::Center),
    )
    .padding(16)
    .style(theme::card_style(&state.theme))
    .width(Length::Fill);

    // 2. Active Rebuilding Progress Banner (visible if scanning)
    let scanning_banner: Option<Element<'_, Message>> = match &mgmt.indexing_state {
        IndexingState::Scanning {
            current_path,
            files_found,
            percent,
        } => Some(
            container(
                column![
                    row![
                        icons::icon(Icon::Refresh, 16.0, colors::LIGHT_ACCENT),
                        text(format!("Rebuilding Index: {:.1}%", percent)).size(14),
                        Space::new().width(Length::Fill),
                        text(format!(
                            "{} files indexed so far",
                            crate::state::format_number(*files_found)
                        ))
                        .size(12),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                    text(format!("Scanning: {}", current_path)).size(11),
                ]
                .spacing(6),
            )
            .padding(14)
            .style(theme::hero_card_style(
                &state.theme,
                if state.theme.is_dark() {
                    colors::DARK_SURFACE
                } else {
                    colors::TINT_BLUE
                },
            ))
            .width(Length::Fill)
            .into(),
        ),
        _ => None,
    };

    // 3. Stats Overview Cards
    let files_card = container(
        column![
            text("TOTAL FILES").size(11).color(colors::ACCENT_PURPLE),
            text(stats.formatted_files()).size(20),
        ]
        .spacing(4),
    )
    .padding(14)
    .style(theme::card_style(&state.theme))
    .width(Length::FillPortion(1));

    let folders_card = container(
        column![
            text("TOTAL FOLDERS").size(11).color(colors::ACCENT_BLUE),
            text(stats.formatted_folders()).size(20),
        ]
        .spacing(4),
    )
    .padding(14)
    .style(theme::card_style(&state.theme))
    .width(Length::FillPortion(1));

    let memory_card = container(
        column![
            text("MEMORY / DB SIZE")
                .size(11)
                .color(colors::ACCENT_AMBER),
            text(stats.formatted_memory()).size(20),
        ]
        .spacing(4),
    )
    .padding(14)
    .style(theme::card_style(&state.theme))
    .width(Length::FillPortion(1));

    let scan_card = container(
        column![
            text("LAST FULL SCAN")
                .size(11)
                .color(colors::ACCENT_EMERALD),
            text(stats.formatted_last_scan()).size(16),
        ]
        .spacing(4),
    )
    .padding(14)
    .style(theme::card_style(&state.theme))
    .width(Length::FillPortion(1));

    let watcher_accent = if stats.watcher_active {
        colors::LIGHT_SUCCESS
    } else {
        colors::LIGHT_MUTED
    };
    let watcher_card = container(
        column![
            text("REAL-TIME WATCHER").size(11).color(watcher_accent),
            text(stats.watcher_status_label()).size(13),
        ]
        .spacing(4),
    )
    .padding(14)
    .style(theme::card_style(&state.theme))
    .width(Length::FillPortion(1));

    let stats_row = row![
        files_card,
        folders_card,
        memory_card,
        scan_card,
        watcher_card,
    ]
    .spacing(12);

    // 4. Indexed Locations Section (Clean & Easy to Use)
    let roots_header = row![
        text(format!("Monitored Directory Roots ({})", mgmt.roots.len())).size(15),
        Space::new().width(Length::Fill),
        text("Monitored for live changes via Linux inotify").size(11),
    ]
    .align_y(Alignment::Center);

    let quick_add_roots_row = row![
        text("Quick Add:").size(11),
        button(text("+ Home (~)").size(11))
            .padding([3, 8])
            .style(theme::secondary_btn_style(&state.theme))
            .on_press(Message::IndexManagement(
                IndexManagementMessage::AddQuickRoot("~".to_string())
            )),
        button(text("+ Documents").size(11))
            .padding([3, 8])
            .style(theme::secondary_btn_style(&state.theme))
            .on_press(Message::IndexManagement(
                IndexManagementMessage::AddQuickRoot("~/Documents".to_string())
            )),
        button(text("+ Downloads").size(11))
            .padding([3, 8])
            .style(theme::secondary_btn_style(&state.theme))
            .on_press(Message::IndexManagement(
                IndexManagementMessage::AddQuickRoot("~/Downloads".to_string())
            )),
        button(text("+ Projects").size(11))
            .padding([3, 8])
            .style(theme::secondary_btn_style(&state.theme))
            .on_press(Message::IndexManagement(
                IndexManagementMessage::AddQuickRoot("~/Projects".to_string())
            )),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let mut roots_col = column![roots_header, quick_add_roots_row].spacing(10);

    for root in &mgmt.roots {
        let path_str = root.display_path();
        let counts_str = format!(
            "{} files  •  {} folders",
            crate::state::format_number(root.file_count),
            crate::state::format_number(root.folder_count)
        );

        let root_row = container(
            row![
                icons::icon(Icon::Folder, 16.0, colors::LIGHT_ACCENT),
                column![text(path_str.clone()).size(13), text(counts_str).size(11),].spacing(2),
                Space::new().width(Length::Fill),
                container(text(format!("[{}]", root.status.as_str())).size(11))
                    .padding([2, 6])
                    .style(theme::pill_badge_style(
                        if state.theme.is_dark() {
                            colors::DARK_SELECTED
                        } else {
                            colors::LIGHT_SELECTED
                        },
                        if state.theme.is_dark() {
                            colors::DARK_ACCENT
                        } else {
                            colors::LIGHT_ACCENT
                        },
                    )),
                button(
                    row![
                        icons::icon(Icon::Trash, 11.0, colors::LIGHT_DANGER),
                        text("Remove").size(11),
                    ]
                    .spacing(4)
                    .align_y(Alignment::Center),
                )
                .padding([4, 10])
                .style(theme::danger_btn_style(&state.theme))
                .on_press(Message::IndexManagement(
                    IndexManagementMessage::RemoveRoot(root.path.clone(),)
                )),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        )
        .padding(10)
        .style(theme::subtle_card_style(&state.theme))
        .width(Length::Fill);

        roots_col = roots_col.push(root_row);
    }

    let add_root_row = row![
        text_input(
            "Enter path to index, e.g. /home/user/projects or /media/data",
            &mgmt.new_root_input
        )
        .on_input(|s| Message::IndexManagement(IndexManagementMessage::UpdateNewRootInput(s)))
        .style(theme::search_input_style(&state.theme))
        .padding(8)
        .size(13)
        .width(Length::Fill),
        button(
            row![
                icons::icon(Icon::Plus, 12.0, Color::WHITE),
                text("Add Folder").size(12),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .padding([6, 14])
        .style(theme::primary_btn_style(&state.theme))
        .on_press(Message::IndexManagement(
            IndexManagementMessage::AddRootFromInput
        )),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    roots_col = roots_col.push(add_root_row);

    let roots_card = container(roots_col)
        .padding(16)
        .style(theme::card_style(&state.theme))
        .width(Length::Fill);

    // 5. Visual .gitignore-Style Exclusion Engine Card
    let exclusions_header = row![
        row![
            icons::icon(Icon::Sliders, 16.0, colors::LIGHT_ACCENT),
            text("Exclusion Rules (.gitignore Style)").size(15),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
        Space::new().width(Length::Fill),
        text("Ignored during directory crawls, inotify events, and search queries").size(11),
    ]
    .align_y(Alignment::Center);

    // 5A. 1-Click Common Presets Grid
    let presets_title = text("1-Click Common Presets").size(12);

    let preset_row_1 = row![
        render_preset_chip(state, mgmt, ExclusionPreset::NodeModules),
        render_preset_chip(state, mgmt, ExclusionPreset::Git),
        render_preset_chip(state, mgmt, ExclusionPreset::PythonCache),
        render_preset_chip(state, mgmt, ExclusionPreset::RustTarget),
        render_preset_chip(state, mgmt, ExclusionPreset::BuildArtifacts),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let preset_row_2 = row![
        render_preset_chip(state, mgmt, ExclusionPreset::SystemCache),
        render_preset_chip(state, mgmt, ExclusionPreset::VirtualEnvs),
        render_preset_chip(state, mgmt, ExclusionPreset::TempLogs),
        render_preset_chip(state, mgmt, ExclusionPreset::HiddenFiles),
        Space::new().width(Length::Fill),
        button(
            row![
                icons::icon(Icon::Refresh, 11.0, state.theme.text_color()),
                text("Restore Defaults").size(11),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        )
        .padding([3, 10])
        .style(theme::secondary_btn_style(&state.theme))
        .on_press(Message::IndexManagement(
            IndexManagementMessage::AddDefaultExclusions
        )),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let presets_section = column![presets_title, preset_row_1, preset_row_2].spacing(8);

    // 5B. Excluded File Extensions (File Types)
    let ext_header = row![
        text(format!(
            "Excluded File Types & Extensions ({})",
            mgmt.excluded_extensions.len()
        ))
        .size(13),
        Space::new().width(Length::Fill),
        text("Files ending with these extensions will not be indexed (e.g. *.pyc, *.tmp)").size(11),
    ]
    .align_y(Alignment::Center);

    let mut ext_chips_row = row![].spacing(6).align_y(Alignment::Center);
    for ext in &mgmt.excluded_extensions {
        let ext_chip = container(
            row![
                text(format!("*.{}", ext)).size(11),
                button(icons::icon(Icon::Cross, 10.0, colors::LIGHT_DANGER))
                    .padding(2)
                    .style(theme::danger_btn_style(&state.theme))
                    .on_press(Message::IndexManagement(
                        IndexManagementMessage::RemoveExcludedExtension(ext.clone())
                    )),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .padding([3, 8])
        .style(theme::subtle_card_style(&state.theme));

        ext_chips_row = ext_chips_row.push(ext_chip);
    }

    let add_ext_row = row![
        text_input(
            "Enter extension to exclude (e.g. pyc, tmp, log, bak, swp, out)",
            &mgmt.new_extension_input
        )
        .on_input(|s| Message::IndexManagement(IndexManagementMessage::UpdateNewExtensionInput(s)))
        .style(theme::search_input_style(&state.theme))
        .padding(8)
        .size(12)
        .width(Length::Fill),
        button(
            row![
                icons::icon(Icon::Plus, 11.0, Color::WHITE),
                text("Add Extension").size(11),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        )
        .padding([6, 12])
        .style(theme::primary_btn_style(&state.theme))
        .on_press(Message::IndexManagement(
            IndexManagementMessage::AddExtensionFromInput
        )),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let extensions_section = column![ext_header, ext_chips_row, add_ext_row].spacing(8);

    // 5C. Excluded Folder Names
    let folders_header = row![
        text(format!(
            "Excluded Folder Names ({})",
            mgmt.excluded_folders.len()
        ))
        .size(13),
        Space::new().width(Length::Fill),
        text("Any folder with this name anywhere in tree is skipped").size(11),
    ]
    .align_y(Alignment::Center);

    let mut folder_chips_row = row![].spacing(6).align_y(Alignment::Center);
    for folder in &mgmt.excluded_folders {
        let folder_chip = container(
            row![
                text(format!("{}/", folder)).size(11),
                button(icons::icon(Icon::Cross, 10.0, colors::LIGHT_DANGER))
                    .padding(2)
                    .style(theme::danger_btn_style(&state.theme))
                    .on_press(Message::IndexManagement(
                        IndexManagementMessage::RemoveExcludedFolder(folder.clone())
                    )),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .padding([3, 8])
        .style(theme::subtle_card_style(&state.theme));

        folder_chips_row = folder_chips_row.push(folder_chip);
    }

    let add_folder_row = row![
        text_input(
            "Enter folder name to exclude (e.g. node_modules, target, dist, build, vendor, .cache)",
            &mgmt.new_folder_input
        )
        .on_input(|s| Message::IndexManagement(IndexManagementMessage::UpdateNewFolderInput(s)))
        .style(theme::search_input_style(&state.theme))
        .padding(8)
        .size(12)
        .width(Length::Fill),
        button(
            row![
                icons::icon(Icon::Plus, 11.0, Color::WHITE),
                text("Add Folder Rule").size(11),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        )
        .padding([6, 12])
        .style(theme::primary_btn_style(&state.theme))
        .on_press(Message::IndexManagement(
            IndexManagementMessage::AddFolderFromInput
        )),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let folders_section = column![folders_header, folder_chips_row, add_folder_row].spacing(8);

    // 5D. Specific Excluded Paths (if any)
    let paths_section = if !mgmt.excluded_paths.is_empty() {
        let paths_header = row![
            text(format!(
                "Specific Excluded Paths ({})",
                mgmt.excluded_paths.len()
            ))
            .size(13),
            Space::new().width(Length::Fill),
            text("Exact absolute or tilde-relative filesystem paths excluded").size(11),
        ]
        .align_y(Alignment::Center);

        let mut path_chips_row = row![].spacing(6).align_y(Alignment::Center);
        for path in &mgmt.excluded_paths {
            let path_chip = container(
                row![
                    text(path.clone()).size(11),
                    button(icons::icon(Icon::Cross, 10.0, colors::LIGHT_DANGER))
                        .padding(2)
                        .style(theme::danger_btn_style(&state.theme))
                        .on_press(Message::IndexManagement(
                            IndexManagementMessage::RemoveExcludedPath(path.clone())
                        )),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .padding([3, 8])
            .style(theme::subtle_card_style(&state.theme));

            path_chips_row = path_chips_row.push(path_chip);
        }

        Some(column![paths_header, path_chips_row].spacing(6))
    } else {
        None
    };

    // 5E. Smart Pattern Rule Adder (.gitignore syntax)
    let smart_header = row![
        text("Smart Rule Adder (.gitignore Syntax)").size(13),
        Space::new().width(Length::Fill),
        text("Type wildcard (*.min.js), folder (vendor/), or path to auto-categorize").size(11),
    ]
    .align_y(Alignment::Center);

    let add_smart_row = row![
        text_input(
            "Enter .gitignore pattern, e.g. *.pyc, node_modules/, ~/.cache, or /var/log",
            &mgmt.new_exclusion_input
        )
        .on_input(|s| Message::IndexManagement(IndexManagementMessage::UpdateNewExclusionInput(s)))
        .style(theme::search_input_style(&state.theme))
        .padding(8)
        .size(12)
        .width(Length::Fill),
        button(
            row![
                icons::icon(Icon::Plus, 11.0, Color::WHITE),
                text("Add Smart Rule").size(11),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        )
        .padding([6, 12])
        .style(theme::primary_btn_style(&state.theme))
        .on_press(Message::IndexManagement(
            IndexManagementMessage::AddExclusionFromInput
        )),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let smart_section = column![smart_header, add_smart_row].spacing(8);

    // 5F. Live .gitignore Syntax Preview
    let gitignore_actions_row = row![
        button(
            row![
                icons::icon(Icon::FileCode, 12.0, state.theme.text_color()),
                text(if mgmt.view_gitignore_raw {
                    "Hide .gitignore Syntax"
                } else {
                    "View .gitignore Syntax"
                })
                .size(11),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        )
        .padding([4, 10])
        .style(theme::secondary_btn_style(&state.theme))
        .on_press(Message::IndexManagement(
            IndexManagementMessage::ToggleGitignoreRawView
        )),
        button(
            row![
                icons::icon(Icon::Copy, 12.0, state.theme.text_color()),
                text("Copy Rules").size(11),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        )
        .padding([4, 10])
        .style(theme::secondary_btn_style(&state.theme))
        .on_press(Message::IndexManagement(
            IndexManagementMessage::CopyGitignoreRules
        )),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let gitignore_preview: Option<Element<'_, Message>> = if mgmt.view_gitignore_raw {
        let code_box = container(
            text(mgmt.to_gitignore_string())
                .size(11)
                .color(colors::DARK_SUBTEXT),
        )
        .padding(12)
        .style(theme::hero_card_style(
            &state.theme,
            if state.theme.is_dark() {
                colors::DARK_HOVER
            } else {
                colors::LIGHT_HOVER
            },
        ))
        .width(Length::Fill);

        Some(column![code_box].spacing(4).into())
    } else {
        None
    };

    let mut exclusions_col = column![
        exclusions_header,
        presets_section,
        extensions_section,
        folders_section,
    ]
    .spacing(14);

    if let Some(p_section) = paths_section {
        exclusions_col = exclusions_col.push(p_section);
    }

    exclusions_col = exclusions_col
        .push(smart_section)
        .push(gitignore_actions_row);

    if let Some(preview) = gitignore_preview {
        exclusions_col = exclusions_col.push(preview);
    }

    let exclusions_card = container(exclusions_col)
        .padding(16)
        .style(theme::card_style(&state.theme))
        .width(Length::Fill);

    let mut main_content = column![header_card,].spacing(14).padding(16);

    if let Some(banner) = scanning_banner {
        main_content = main_content.push(banner);
    }

    main_content = main_content
        .push(stats_row)
        .push(roots_card)
        .push(exclusions_card);

    scrollable(main_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Helper to render an individual 1-click preset toggle button
fn render_preset_chip<'a>(
    state: &AppState,
    mgmt: &crate::index_mgmt::IndexManagementState,
    preset: ExclusionPreset,
) -> Element<'a, Message> {
    let active = mgmt.is_preset_active(preset);
    let label = if active {
        format!("✓ {}", preset.label())
    } else {
        format!("+ {}", preset.label())
    };

    button(text(label).size(11))
        .padding([4, 10])
        .style(theme::chip_btn_style(&state.theme, active))
        .on_press(Message::IndexManagement(
            IndexManagementMessage::TogglePreset(preset),
        ))
        .into()
}
