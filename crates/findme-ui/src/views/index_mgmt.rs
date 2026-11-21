//! Index management and custom exclusion rules screen

use crate::index_mgmt::IndexingState;
use crate::message::{IndexManagementMessage, Message};
use crate::state::AppState;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};

/// Renders the Index Management screen
pub fn view(state: &AppState) -> Element<'_, Message> {
    let mgmt = &state.index_mgmt;
    let stats = &mgmt.stats;

    // 1. Header & Actions Bar
    let header_title = column![
        text("📁 Index Management & Custom Folder Exclusions").size(18),
        text("Configure indexed directory roots, monitor memory footprint, and manage skip rules")
            .size(12),
    ]
    .spacing(4);

    let action_bar = row![
        button(text("⚡ Rebuild Index").size(12))
            .padding([5, 12])
            .on_press(Message::IndexManagement(
                IndexManagementMessage::TriggerRebuildIndex
            )),
        button(text("🔄 Refresh Index").size(12))
            .padding([5, 12])
            .on_press(Message::IndexManagement(
                IndexManagementMessage::TriggerRefreshIndex
            )),
        button(text("🗑️ Clear Index").size(12))
            .padding([5, 12])
            .on_press(Message::IndexManagement(IndexManagementMessage::ClearIndex)),
    ]
    .spacing(8);

    let header_card = container(
        row![header_title, Space::new().width(Length::Fill), action_bar,]
            .spacing(12)
            .align_y(Alignment::Center),
    )
    .padding(12)
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
                        text(format!("⚡ Rebuilding Index: {:.1}%", percent)).size(14),
                        Space::new().width(Length::Fill),
                        text(format!(
                            "{} files indexed so far",
                            crate::state::format_number(*files_found)
                        ))
                        .size(12),
                    ]
                    .align_y(Alignment::Center),
                    text(format!("Scanning: {}", current_path)).size(11),
                ]
                .spacing(6),
            )
            .padding(12)
            .width(Length::Fill)
            .into(),
        ),
        _ => None,
    };

    // 3. Stats Overview Cards
    let files_card = container(
        column![
            text("Total Indexed Files").size(11),
            text(stats.formatted_files()).size(18),
        ]
        .spacing(2),
    )
    .padding(10)
    .width(Length::FillPortion(1));

    let folders_card = container(
        column![
            text("Total Folders").size(11),
            text(stats.formatted_folders()).size(18),
        ]
        .spacing(2),
    )
    .padding(10)
    .width(Length::FillPortion(1));

    let memory_card = container(
        column![
            text("Memory / DB Size").size(11),
            text(stats.formatted_memory()).size(18),
        ]
        .spacing(2),
    )
    .padding(10)
    .width(Length::FillPortion(1));

    let scan_card = container(
        column![
            text("Last Full Scan").size(11),
            text(stats.formatted_last_scan()).size(18),
        ]
        .spacing(2),
    )
    .padding(10)
    .width(Length::FillPortion(1));

    let watcher_card = container(
        column![
            text("Real-time Watcher").size(11),
            text(stats.watcher_status_label()).size(14),
        ]
        .spacing(2),
    )
    .padding(10)
    .width(Length::FillPortion(1));

    let stats_row = row![
        files_card,
        folders_card,
        memory_card,
        scan_card,
        watcher_card,
    ]
    .spacing(10);

    // 4. Indexed Locations Section
    let roots_header = row![
        text(format!("Indexed Root Directories ({})", mgmt.roots.len())).size(14),
        Space::new().width(Length::Fill),
        text("Monitored for live changes via Linux inotify").size(11),
    ]
    .align_y(Alignment::Center);

    let mut roots_col = column![roots_header].spacing(8);

    for root in &mgmt.roots {
        let path_str = root.display_path();
        let counts_str = format!(
            "{} files  •  {} folders",
            crate::state::format_number(root.file_count),
            crate::state::format_number(root.folder_count)
        );

        let root_row = container(
            row![
                text("📁").size(16),
                column![text(path_str.clone()).size(13), text(counts_str).size(11),].spacing(2),
                Space::new().width(Length::Fill),
                text(format!("[{}]", root.status.as_str())).size(11),
                button(text("🗑️ Remove").size(11)).padding([3, 8]).on_press(
                    Message::IndexManagement(
                        IndexManagementMessage::RemoveRoot(root.path.clone(),)
                    )
                ),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .padding(8)
        .width(Length::Fill);

        roots_col = roots_col.push(root_row);
    }

    let add_root_row = row![
        text_input(
            "Enter path to index, e.g. /home/user/projects or /media/data",
            &mgmt.new_root_input
        )
        .on_input(|s| Message::IndexManagement(IndexManagementMessage::UpdateNewRootInput(s)))
        .padding(6)
        .size(13)
        .width(Length::Fill),
        button(text("➕ Add Folder").size(12))
            .padding([6, 12])
            .on_press(Message::IndexManagement(
                IndexManagementMessage::AddRootFromInput
            )),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    roots_col = roots_col.push(add_root_row);

    let roots_card = container(roots_col).padding(12).width(Length::Fill);

    // 5. Excluded Locations & Skip Patterns Section
    let exclusions_header = row![
        text(format!(
            "Excluded Locations & Patterns ({})",
            mgmt.exclusions.len()
        ))
        .size(14),
        Space::new().width(Length::Fill),
        text("Skipped during crawler traversal and file indexing").size(11),
    ]
    .align_y(Alignment::Center);

    let quick_add_row = row![
        text("Quick Add:").size(11),
        button(text("+ .git").size(11))
            .padding([2, 6])
            .on_press(Message::IndexManagement(
                IndexManagementMessage::AddExclusion(".git".to_string())
            )),
        button(text("+ node_modules").size(11))
            .padding([2, 6])
            .on_press(Message::IndexManagement(
                IndexManagementMessage::AddExclusion("node_modules".to_string())
            )),
        button(text("+ target").size(11))
            .padding([2, 6])
            .on_press(Message::IndexManagement(
                IndexManagementMessage::AddExclusion("target".to_string())
            )),
        button(text("+ ~/.cache").size(11))
            .padding([2, 6])
            .on_press(Message::IndexManagement(
                IndexManagementMessage::AddExclusion("~/.cache".to_string())
            )),
        Space::new().width(Length::Fill),
        button(text("↺ Restore Defaults").size(11))
            .padding([2, 8])
            .on_press(Message::IndexManagement(
                IndexManagementMessage::AddDefaultExclusions
            )),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let mut exclusions_col = column![exclusions_header, quick_add_row].spacing(8);

    for exclusion in &mgmt.exclusions {
        let pattern_str = exclusion.pattern.clone();
        let is_builtin = exclusion.is_builtin;

        let right_widget: Element<'_, Message> = if is_builtin {
            text("🔒 System Built-in (Kernel Virtual FS)")
                .size(11)
                .into()
        } else {
            button(text("✕ Remove").size(11))
                .padding([2, 8])
                .on_press(Message::IndexManagement(
                    IndexManagementMessage::RemoveExclusion(pattern_str.clone()),
                ))
                .into()
        };

        let item_row = container(
            row![
                text("🚫").size(14),
                text(pattern_str).size(13),
                Space::new().width(Length::Fill),
                right_widget,
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .padding([6, 8])
        .width(Length::Fill);

        exclusions_col = exclusions_col.push(item_row);
    }

    let add_exclusion_row = row![
        text_input(
            "Enter path or pattern to exclude, e.g. build/ or *.tmp or /opt/data",
            &mgmt.new_exclusion_input
        )
        .on_input(|s| Message::IndexManagement(IndexManagementMessage::UpdateNewExclusionInput(s)))
        .padding(6)
        .size(13)
        .width(Length::Fill),
        button(text("➕ Add Exclusion").size(12))
            .padding([6, 12])
            .on_press(Message::IndexManagement(
                IndexManagementMessage::AddExclusionFromInput
            )),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    exclusions_col = exclusions_col.push(add_exclusion_row);

    let exclusions_card = container(exclusions_col).padding(12).width(Length::Fill);

    let mut main_content = column![header_card,].spacing(12).padding(16);

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
