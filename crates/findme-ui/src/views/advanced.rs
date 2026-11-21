//! Advanced Search visual filter builder screen

use crate::message::Message;
use crate::state::AppState;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Length};

/// Renders the Advanced Search query builder
pub fn view(_state: &AppState) -> Element<'_, Message> {
    let header = column![
        text("⚡ Advanced Search Builder").size(18),
        text("Construct precise multi-criteria file search queries visually").size(12),
    ]
    .spacing(4);

    // Extensions Card
    let ext_card = container(
        column![
            text("File Types & Extensions").size(14),
            row![
                button(text("Source Code (ext:rs;c;cpp;py;go;js)").size(12))
                    .on_press(Message::RunQuery("ext:rs;c;cpp;py;go;js".into())),
                button(text("Documents (ext:pdf;docx;txt;md)").size(12))
                    .on_press(Message::RunQuery("ext:pdf;docx;txt;md".into())),
                button(text("Media (ext:png;jpg;mp4;mp3)").size(12))
                    .on_press(Message::RunQuery("ext:png;jpg;mp4;mp3".into())),
                button(text("Archives (ext:zip;tar;gz;xz)").size(12))
                    .on_press(Message::RunQuery("ext:zip;tar;gz;xz".into())),
            ]
            .spacing(8),
        ]
        .spacing(8),
    )
    .padding(12)
    .width(Length::Fill);

    // Size Bounds Card
    let size_card = container(
        column![
            text("File Size Filters").size(14),
            row![
                button(text("Small (< 100 KB)").size(12))
                    .on_press(Message::RunQuery("size:<100kb".into())),
                button(text("Medium (100 KB .. 10 MB)").size(12))
                    .on_press(Message::RunQuery("size:100kb..10mb".into())),
                button(text("Large (> 100 MB)").size(12))
                    .on_press(Message::RunQuery("size:>100mb".into())),
                button(text("Gigantic (> 1 GB)").size(12))
                    .on_press(Message::RunQuery("size:>1gb".into())),
            ]
            .spacing(8),
        ]
        .spacing(8),
    )
    .padding(12)
    .width(Length::Fill);

    // Date Modified Card
    let date_card = container(
        column![
            text("Modification Date").size(14),
            row![
                button(text("Today (dm:today)").size(12))
                    .on_press(Message::RunQuery("dm:today".into())),
                button(text("Yesterday (dm:yesterday)").size(12))
                    .on_press(Message::RunQuery("dm:yesterday".into())),
                button(text("Past Week (dm:lastweek)").size(12))
                    .on_press(Message::RunQuery("dm:lastweek".into())),
                button(text("Past Month (dm:lastmonth)").size(12))
                    .on_press(Message::RunQuery("dm:lastmonth".into())),
            ]
            .spacing(8),
        ]
        .spacing(8),
    )
    .padding(12)
    .width(Length::Fill);

    // Directory Restriction Card
    let path_card = container(
        column![
            text("Target Location & Exclusions").size(14),
            row![
                button(text("Search Home Folder (path:~/)").size(12))
                    .on_press(Message::RunQuery("path:~/".into())),
                button(text("Exclude Build Targets (!target)").size(12))
                    .on_press(Message::RunQuery("!target !node_modules".into())),
            ]
            .spacing(8),
        ]
        .spacing(8),
    )
    .padding(12)
    .width(Length::Fill);

    let form_content = column![header, ext_card, size_card, date_card, path_card].spacing(14);

    scrollable(form_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
