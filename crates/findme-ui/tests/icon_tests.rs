//! Integration tests for Find-Me modern vector SVG icon engine
//!
//! Validates SVG syntax, non-empty paths, dynamic color tinting, and screen/result mapping.

use findme_core::SearchResultItem;
use findme_ui::icons::{icon, Icon};
use findme_ui::message::{Message, ScreenId};
use findme_ui::views::table::item_vector_icon;
use iced::Color;

#[test]
fn test_all_vector_icons_have_valid_svg_markup() {
    let all_icons = [
        Icon::Search,
        Icon::Sliders,
        Icon::Folder,
        Icon::FolderOpen,
        Icon::Clock,
        Icon::Settings,
        Icon::Info,
        Icon::Moon,
        Icon::Sun,
        Icon::File,
        Icon::FileText,
        Icon::FileCode,
        Icon::FileImage,
        Icon::FileAudio,
        Icon::FileVideo,
        Icon::FileArchive,
        Icon::Trash,
        Icon::Refresh,
        Icon::Plus,
        Icon::Save,
        Icon::StarFilled,
        Icon::StarEmpty,
        Icon::ExternalLink,
        Icon::Clipboard,
        Icon::Copy,
        Icon::Cross,
        Icon::ChevronDown,
        Icon::ChevronRight,
        Icon::ChevronVertical,
        Icon::Collapse,
        Icon::Expand,
        Icon::Sparkles,
        Icon::Terminal,
        Icon::Database,
    ];

    assert_eq!(all_icons.len(), 34);

    for ic in all_icons {
        let svg = ic.svg_str();
        assert!(!svg.is_empty(), "Icon {ic:?} SVG string must not be empty");
        assert!(
            svg.starts_with("<svg"),
            "Icon {ic:?} SVG must start with <svg tag, got: {svg}"
        );
        assert!(
            svg.ends_with("</svg>"),
            "Icon {ic:?} SVG must end with </svg>, got: {svg}"
        );
        assert!(
            svg.contains("viewBox=\"0 0 24 24\""),
            "Icon {ic:?} SVG must have viewBox 0 0 24 24"
        );
        assert!(
            svg.contains("stroke=\"currentColor\"") || svg.contains("fill=\"currentColor\""),
            "Icon {ic:?} SVG must use currentColor for dynamic tinting"
        );
    }
}

#[test]
fn test_icon_widget_instantiation() {
    let widget: iced::Element<'_, Message> = icon(Icon::Search, 16.0, Color::WHITE);
    let _ = widget;

    let colored_widget: iced::Element<'_, Message> =
        icon(Icon::Settings, 24.0, Color::from_rgb(0.2, 0.4, 0.8));
    let _ = colored_widget;
}

#[test]
fn test_screen_id_vector_icons() {
    assert_eq!(ScreenId::Search.vector_icon(), Icon::Search);
    assert_eq!(ScreenId::AdvancedSearch.vector_icon(), Icon::Sliders);
    assert_eq!(ScreenId::IndexManagement.vector_icon(), Icon::Folder);
    assert_eq!(ScreenId::SearchHistory.vector_icon(), Icon::Clock);
    assert_eq!(ScreenId::Settings.vector_icon(), Icon::Settings);
    assert_eq!(ScreenId::About.vector_icon(), Icon::Info);
}

#[test]
fn test_item_vector_icon_categorization() {
    // 1. Directory
    let dir_item = SearchResultItem {
        id: 1,
        name: "Documents".to_string(),
        path: "/home/user/Documents".to_string(),
        is_dir: true,
        size_bytes: 4096,
        mtime_secs: 1000,
    };
    let (ic, label) = item_vector_icon(&dir_item);
    assert_eq!(ic, Icon::Folder);
    assert_eq!(label, "Folder");

    // 2. Code (Rust)
    let rs_item = SearchResultItem {
        id: 2,
        name: "main.rs".to_string(),
        path: "/src/main.rs".to_string(),
        is_dir: false,
        size_bytes: 1024,
        mtime_secs: 1000,
    };
    let (ic, label) = item_vector_icon(&rs_item);
    assert_eq!(ic, Icon::FileCode);
    assert_eq!(label, "Rust");

    // 3. Document (Markdown)
    let md_item = SearchResultItem {
        id: 3,
        name: "README.md".to_string(),
        path: "/README.md".to_string(),
        is_dir: false,
        size_bytes: 2048,
        mtime_secs: 1000,
    };
    let (ic, label) = item_vector_icon(&md_item);
    assert_eq!(ic, Icon::FileText);
    assert_eq!(label, "Markdown");

    // 4. Archive (zip)
    let zip_item = SearchResultItem {
        id: 4,
        name: "backup.zip".to_string(),
        path: "/backup.zip".to_string(),
        is_dir: false,
        size_bytes: 100000,
        mtime_secs: 1000,
    };
    let (ic, label) = item_vector_icon(&zip_item);
    assert_eq!(ic, Icon::FileArchive);
    assert_eq!(label, "Archive");

    // 5. Image (png)
    let img_item = SearchResultItem {
        id: 5,
        name: "photo.png".to_string(),
        path: "/photo.png".to_string(),
        is_dir: false,
        size_bytes: 500000,
        mtime_secs: 1000,
    };
    let (ic, label) = item_vector_icon(&img_item);
    assert_eq!(ic, Icon::FileImage);
    assert_eq!(label, "PNG Image");

    // 6. Generic file
    let unknown_item = SearchResultItem {
        id: 6,
        name: "data.bin".to_string(),
        path: "/data.bin".to_string(),
        is_dir: false,
        size_bytes: 8000,
        mtime_secs: 1000,
    };
    let (ic, label) = item_vector_icon(&unknown_item);
    assert_eq!(ic, Icon::File);
    assert_eq!(label, "File");
}
