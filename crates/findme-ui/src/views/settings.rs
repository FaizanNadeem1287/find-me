//! Settings and user preferences screen

use crate::message::{Message, SettingsMessage};
use crate::state::AppState;
use crate::theme::{self, colors};
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
                container(text(theme_text).size(12)).padding([3, 8]).style(
                    theme::pill_badge_style(
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
                    )
                ),
                Space::new().width(Length::Fill),
                button(text("Toggle Theme (Ctrl+T)").size(11))
                    .padding([5, 12])
                    .style(theme::secondary_btn_style(&state.theme))
                    .on_press(Message::ToggleTheme),
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(10),
    )
    .padding(16)
    .style(theme::card_style(&state.theme))
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
        .spacing(12),
    )
    .padding(16)
    .style(theme::card_style(&state.theme))
    .width(Length::Fill);

    // 3. Search Defaults Card
    let mut limit_buttons = row![text("Max Results Limit:").size(12)]
        .spacing(8)
        .align_y(Alignment::Center);
    for limit in [1000, 5000, 10000, 50000] {
        let is_active = settings.max_results == limit;
        let label = crate::state::format_number(limit as u64);
        let btn = button(text(label).size(11))
            .padding([4, 10])
            .style(theme::chip_btn_style(&state.theme, is_active))
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
        .spacing(12),
    )
    .padding(16)
    .style(theme::card_style(&state.theme))
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
        .spacing(12),
    )
    .padding(16)
    .style(theme::card_style(&state.theme))
    .width(Length::Fill);

    // 5. Hotkeys Cheatsheet Card
    let shortcut_row = |key: &'static str, desc: &'static str| -> Element<'_, Message> {
        row![
            container(text(key).size(11))
                .padding([2, 6])
                .style(theme::kbd_badge_style(&state.theme)),
            text(desc).size(12),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .width(Length::FillPortion(1))
        .into()
    };

    let hotkeys_card = container(
        column![
            text("Desktop Shortcuts Cheat Sheet").size(14),
            row![
                shortcut_row("Ctrl + F / /", "Focus search input bar"),
                shortcut_row("Esc", "Clear search input or dismiss"),
            ],
            row![
                shortcut_row("Up / Down", "Step result selection"),
                shortcut_row("PageUp / PageDn", "Jump 15 rows"),
            ],
            row![
                shortcut_row("Home / End", "Jump to first / last item"),
                shortcut_row("Enter", "Open selected file"),
            ],
            row![
                shortcut_row("Alt + Enter", "File properties modal"),
                shortcut_row("Ctrl+Shift+C", "Copy absolute path"),
            ],
            row![
                shortcut_row("Ctrl+Shift+N", "Copy file name"),
                shortcut_row("Ctrl+Shift+U", "Copy file URI"),
            ],
            row![
                shortcut_row("Ctrl + T", "Toggle visual theme"),
                shortcut_row("Ctrl + B", "Toggle sidebar"),
            ],
            row![
                shortcut_row("Ctrl + 1 .. 6", "Switch screens"),
                shortcut_row("F5 / Ctrl + R", "Refresh search query"),
            ],
            row![
                shortcut_row("Alt + 1 .. 5", "Sort columns"),
                shortcut_row("Shift + F10", "Open context menu"),
            ],
        ]
        .spacing(10),
    )
    .padding(16)
    .style(theme::card_style(&state.theme))
    .width(Length::Fill);

    // 6. Action Buttons Bar
    let action_bar = row![
        button(text("💾 Save Preferences").size(12))
            .padding([7, 16])
            .style(theme::primary_btn_style(&state.theme))
            .on_press(Message::Settings(SettingsMessage::SaveSettings)),
        button(text("↺ Reset to Defaults").size(12))
            .padding([7, 16])
            .style(theme::secondary_btn_style(&state.theme))
            .on_press(Message::Settings(SettingsMessage::ResetToDefaults)),
    ]
    .spacing(12);

    let content = column![
        header,
        appearance_card,
        general_card,
        search_defaults_card,
        privacy_card,
        hotkeys_card,
        action_bar,
    ]
    .spacing(14)
    .padding(16);

    scrollable(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
