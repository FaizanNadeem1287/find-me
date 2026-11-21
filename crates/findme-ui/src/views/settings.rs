//! Settings and user preferences screen

use crate::message::{Message, SettingsMessage};
use crate::state::AppState;
use iced::widget::{button, checkbox, column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};

/// Renders the Settings screen
pub fn view(state: &AppState) -> Element<'_, Message> {
    let settings = &state.settings;

    let header = column![
        text("⚙️ Application Preferences & Settings").size(18),
        text("Configure desktop autostart, default search options, privacy boundaries, and shortcuts")
            .size(12),
    ]
    .spacing(4);

    // 1. Appearance & Theme Card
    let theme_text = if state.theme.is_dark() {
        "Dark Theme (Active)"
    } else {
        "Light Theme (Active)"
    };

    let appearance_card = container(
        column![
            text("Appearance & Visual Theme").size(14),
            row![
                text(theme_text).size(12),
                Space::new().width(Length::Fill),
                button(text("Toggle Theme (Ctrl+T)").size(11))
                    .padding([4, 10])
                    .on_press(Message::ToggleTheme),
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(8),
    )
    .padding(12)
    .width(Length::Fill);

    // 2. General Desktop Integration Card
    let general_card = container(
        column![
            text("General & Desktop Integration").size(14),
            checkbox(settings.autostart)
                .label("Launch Find-Me automatically on system startup (XDG autostart)")
                .text_size(12)
                .on_toggle(|b| Message::Settings(SettingsMessage::ToggleAutostart(b))),
            checkbox(settings.close_to_tray)
                .label("Keep background watcher alive when window is closed (tray integration)")
                .text_size(12)
                .on_toggle(|b| Message::Settings(SettingsMessage::ToggleCloseToTray(b))),
        ]
        .spacing(10),
    )
    .padding(12)
    .width(Length::Fill);

    // 3. Search Defaults Card
    let mut limit_buttons = row![text("Max Results Limit:").size(12)]
        .spacing(8)
        .align_y(Alignment::Center);
    for limit in [1000, 5000, 10000, 50000] {
        let is_active = settings.max_results == limit;
        let label = if is_active {
            format!("● {}", crate::state::format_number(limit as u64))
        } else {
            crate::state::format_number(limit as u64)
        };
        let btn = button(text(label).size(11))
            .padding([3, 8])
            .on_press(Message::Settings(SettingsMessage::SetMaxResults(limit)));
        limit_buttons = limit_buttons.push(btn);
    }

    let search_defaults_card = container(
        column![
            text("Search Behavior & Defaults").size(14),
            checkbox(settings.default_regex)
                .label("Default to Regular Expression mode (regex:) for all queries")
                .text_size(12)
                .on_toggle(|b| Message::Settings(SettingsMessage::ToggleDefaultRegex(b))),
            checkbox(settings.default_case_sensitive)
                .label("Default to Case-Sensitive matching (case:) for all queries")
                .text_size(12)
                .on_toggle(|b| Message::Settings(SettingsMessage::ToggleDefaultCase(b))),
            limit_buttons,
        ]
        .spacing(10),
    )
    .padding(12)
    .width(Length::Fill);

    // 4. Privacy & Filesystem Boundaries Card
    let privacy_card = container(
        column![
            text("Privacy & Filesystem Boundaries").size(14),
            checkbox(settings.exclude_hidden_files)
                .label("Exclude hidden files and dot-directories (.*) from search results")
                .text_size(12)
                .on_toggle(|b| Message::Settings(SettingsMessage::ToggleExcludeHidden(b))),
            checkbox(settings.exclude_system_paths)
                .label("Exclude Linux kernel virtual filesystems (/proc, /sys, /dev)")
                .text_size(12)
                .on_toggle(|b| Message::Settings(SettingsMessage::ToggleExcludeSystem(b))),
        ]
        .spacing(10),
    )
    .padding(12)
    .width(Length::Fill);

    // 5. Hotkeys Cheatsheet Card
    let hotkeys_card = container(
        column![
            text("Desktop Shortcuts Cheat Sheet").size(14),
            row![
                text("• Ctrl + F  or  / : Focus search input bar")
                    .size(12)
                    .width(Length::FillPortion(1)),
                text("• Esc : Clear search input or dismiss")
                    .size(12)
                    .width(Length::FillPortion(1)),
            ],
            row![
                text("• Up / Down : Step result selection")
                    .size(12)
                    .width(Length::FillPortion(1)),
                text("• PageUp / PageDown : Jump 15 rows")
                    .size(12)
                    .width(Length::FillPortion(1)),
            ],
            row![
                text("• Home / End : Jump to first / last item")
                    .size(12)
                    .width(Length::FillPortion(1)),
                text("• Enter : Open selected file")
                    .size(12)
                    .width(Length::FillPortion(1)),
            ],
            row![
                text("• Alt + Enter : Open containing directory")
                    .size(12)
                    .width(Length::FillPortion(1)),
                text("• Ctrl + Shift + C : Copy absolute path")
                    .size(12)
                    .width(Length::FillPortion(1)),
            ],
            row![
                text("• Ctrl + T : Toggle visual theme")
                    .size(12)
                    .width(Length::FillPortion(1)),
                text("• Ctrl + 1 .. 6 : Switch screens")
                    .size(12)
                    .width(Length::FillPortion(1)),
            ],
        ]
        .spacing(6),
    )
    .padding(12)
    .width(Length::Fill);

    // 6. Action Buttons Bar
    let action_bar = row![
        button(text("💾 Save Preferences").size(12))
            .padding([6, 14])
            .on_press(Message::Settings(SettingsMessage::SaveSettings)),
        button(text("↺ Reset to Defaults").size(12))
            .padding([6, 14])
            .on_press(Message::Settings(SettingsMessage::ResetToDefaults)),
    ]
    .spacing(10);

    let content = column![
        header,
        appearance_card,
        general_card,
        search_defaults_card,
        privacy_card,
        hotkeys_card,
        action_bar,
    ]
    .spacing(12)
    .padding(16);

    scrollable(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
