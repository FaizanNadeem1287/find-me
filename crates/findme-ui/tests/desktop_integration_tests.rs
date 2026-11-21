//! Comprehensive integration and QA tests for Linux desktop integration
//!
//! Tests DBus FileManager1 integration, clipboard management, POSIX metadata extraction,
//! context menu Elm loop state transitions, and properties modal dialog rendering.

use findme_core::SearchResultItem;
use findme_ui::desktop::{
    copy_file_uri, copy_to_clipboard, get_file_properties, show_in_file_manager, FileProperties,
};
use findme_ui::message::Message;
use findme_ui::views;
use findme_ui::FindMeApp;
use iced::keyboard::key::Named;
use iced::keyboard::{Event, Key, Modifiers};
use std::path::Path;

#[test]
fn test_posix_mode_formatting() {
    // 1. Directory with 0755
    let dir_mode = FileProperties::format_mode(0o755, true);
    assert_eq!(dir_mode, "drwxr-xr-x (0755)");

    // 2. Regular file with 0644
    let file_mode = FileProperties::format_mode(0o644, false);
    assert_eq!(file_mode, "-rw-r--r-- (0644)");

    // 3. Executable file with 0777
    let exec_mode = FileProperties::format_mode(0o777, false);
    assert_eq!(exec_mode, "-rwxrwxrwx (0777)");

    // 4. Strict read-only with 0400
    let ro_mode = FileProperties::format_mode(0o400, false);
    assert_eq!(ro_mode, "-r-------- (0400)");

    // 5. Zero permissions with 0000
    let no_perms = FileProperties::format_mode(0o000, false);
    assert_eq!(no_perms, "---------- (0000)");
}

#[test]
fn test_timestamp_utc_formatting() {
    // Unix epoch 0 should be 1970-01-01 00:00:00 UTC
    let epoch = FileProperties::format_timestamp(0);
    assert_eq!(epoch, "1970-01-01 00:00:00 UTC");

    // Known timestamp: 86400 (1 day later)
    let day_one = FileProperties::format_timestamp(86400);
    assert_eq!(day_one, "1970-01-02 00:00:00 UTC");

    // Known timestamp: 1774224000 (2026-03-23 00:00:00 UTC)
    let stamp_2026 = FileProperties::format_timestamp(1774224000);
    assert!(stamp_2026.starts_with("2026-03-23"));
    assert!(stamp_2026.ends_with("UTC"));
}

#[test]
fn test_get_file_properties_real_fs() {
    // Test on repository Cargo.toml
    let cargo_toml = Path::new("Cargo.toml");
    assert!(cargo_toml.exists(), "Cargo.toml must exist in repo root");

    let props = get_file_properties(cargo_toml).expect("Should extract Cargo.toml properties");
    assert_eq!(props.name, "Cargo.toml");
    assert!(!props.is_dir);
    assert!(props.size_bytes > 0);
    assert!(props.inode > 0);
    assert!(props.device_id > 0);
    assert!(props.mode_string.contains("(0"));
    assert!(props.flags_description.contains("Regular File"));

    // Test on directory (src)
    let src_dir = Path::new("src");
    let dir_props = get_file_properties(src_dir).expect("Should extract src/ dir properties");
    assert!(dir_props.is_dir);
    assert!(dir_props.flags_description.contains("Directory"));

    // Test non-existent path
    let bad_path = Path::new("this_file_does_not_exist_at_all.xyz");
    let err = get_file_properties(bad_path);
    assert!(err.is_err(), "Non-existent path must return Err");
}

#[test]
fn test_clipboard_and_file_uri() {
    // Test copy_to_clipboard
    let text = "find-me-test-clipboard-payload";
    // arboard might return Err in purely headless environments without X11/Wayland server running,
    // so we verify that the function handles it gracefully without panicking.
    let _ = copy_to_clipboard(text);

    // Test copy_file_uri
    let cargo_toml = Path::new("Cargo.toml");
    let uri_res = copy_file_uri(cargo_toml);
    if let Ok(uri) = uri_res {
        assert!(uri.starts_with("file://"));
        assert!(uri.contains("Cargo.toml"));
    }
}

#[test]
fn test_show_in_file_manager_safety() {
    let cargo_toml = Path::new("Cargo.toml");
    // Should attempt DBus ShowItems then fallback to open::that_detached without panicking
    let _ = show_in_file_manager(cargo_toml);
}

#[test]
fn test_app_context_menu_lifecycle() {
    let mut app = FindMeApp::new();
    assert!(!app.state.context_menu.is_open);
    assert!(app.state.context_menu.target_item.is_none());

    let test_item = SearchResultItem {
        id: 42,
        name: "main.rs".to_string(),
        path: "/home/user/project/src/main.rs".to_string(),
        size_bytes: 1024,
        mtime_secs: 1700000000,
        is_dir: false,
    };

    // 1. Open context menu
    let _ = app.update(Message::OpenContextMenu {
        item: test_item.clone(),
        x: 100.0,
        y: 200.0,
    });
    assert!(app.state.context_menu.is_open);
    assert_eq!(
        app.state.context_menu.target_item.as_ref().unwrap().name,
        "main.rs"
    );
    assert_eq!(app.state.context_menu.position, (100.0, 200.0));

    // 2. Close context menu
    let _ = app.update(Message::CloseContextMenu);
    assert!(!app.state.context_menu.is_open);

    // 3. Re-open and execute CopyFileName
    let _ = app.update(Message::OpenContextMenu {
        item: test_item.clone(),
        x: 0.0,
        y: 0.0,
    });
    assert!(app.state.context_menu.is_open);
    let _ = app.update(Message::CopyFileName("main.rs".to_string()));
    assert!(!app.state.context_menu.is_open);
    assert!(app.state.notification.is_some());

    // 4. Re-open and execute CopyPath
    let _ = app.update(Message::OpenContextMenu {
        item: test_item.clone(),
        x: 0.0,
        y: 0.0,
    });
    let _ = app.update(Message::CopyPath("/tmp/test".to_string()));
    assert!(!app.state.context_menu.is_open);
    assert!(app.state.notification.is_some());

    // 5. Re-open and execute CopyFileUri
    let _ = app.update(Message::OpenContextMenu {
        item: test_item.clone(),
        x: 0.0,
        y: 0.0,
    });
    let _ = app.update(Message::CopyFileUri("Cargo.toml".to_string()));
    assert!(!app.state.context_menu.is_open);
    assert!(app.state.notification.is_some());
}

#[test]
fn test_app_properties_modal_lifecycle() {
    let mut app = FindMeApp::new();
    assert!(app.state.properties_modal.is_none());

    // 1. Show properties for existing Cargo.toml
    let _ = app.update(Message::ShowProperties("Cargo.toml".to_string()));
    assert!(app.state.properties_modal.is_some());
    let props = app.state.properties_modal.as_ref().unwrap();
    assert_eq!(props.name, "Cargo.toml");
    assert!(!props.is_dir);

    // 2. Close properties modal
    let _ = app.update(Message::CloseProperties);
    assert!(app.state.properties_modal.is_none());

    // 3. Show properties for invalid path sets error notification
    let _ = app.update(Message::ShowProperties("non_existent_file.xyz".to_string()));
    assert!(app.state.properties_modal.is_none());
    assert!(app.state.notification.is_some());
}

#[test]
fn test_keyboard_escape_dismisses_properties_and_context_menu() {
    let mut app = FindMeApp::new();

    // Setup: open properties modal
    let _ = app.update(Message::ShowProperties("Cargo.toml".to_string()));
    assert!(app.state.properties_modal.is_some());

    // Press Escape -> closes properties modal
    let esc_event = Event::KeyPressed {
        key: Key::Named(Named::Escape),
        modified_key: Key::Named(Named::Escape),
        physical_key: iced::keyboard::key::Physical::Code(iced::keyboard::key::Code::Escape),
        location: iced::keyboard::Location::Standard,
        modifiers: Modifiers::empty(),
        repeat: false,
        text: None,
    };
    let _ = app.update(Message::KeyboardEvent(esc_event));
    assert!(app.state.properties_modal.is_none());

    // Setup: open context menu
    let test_item = SearchResultItem {
        id: 1,
        name: "test.txt".to_string(),
        path: "/test.txt".to_string(),
        size_bytes: 100,
        mtime_secs: 1000,
        is_dir: false,
    };
    let _ = app.update(Message::OpenContextMenu {
        item: test_item,
        x: 0.0,
        y: 0.0,
    });
    assert!(app.state.context_menu.is_open);

    // Press Escape -> closes context menu
    let esc_event2 = Event::KeyPressed {
        key: Key::Named(Named::Escape),
        modified_key: Key::Named(Named::Escape),
        physical_key: iced::keyboard::key::Physical::Code(iced::keyboard::key::Code::Escape),
        location: iced::keyboard::Location::Standard,
        modifiers: Modifiers::empty(),
        repeat: false,
        text: None,
    };
    let _ = app.update(Message::KeyboardEvent(esc_event2));
    assert!(!app.state.context_menu.is_open);
}

#[test]
fn test_keyboard_shortcuts_for_selection() {
    let mut app = FindMeApp::new();
    let test_item = SearchResultItem {
        id: 1,
        name: "Cargo.toml".to_string(),
        path: "Cargo.toml".to_string(),
        size_bytes: 500,
        mtime_secs: 1000,
        is_dir: false,
    };
    app.state.results = vec![test_item];
    app.state.selected_row = Some(0);

    // 1. Shift+F10 opens Context Menu
    let mut shift_mod = Modifiers::empty();
    shift_mod.insert(Modifiers::SHIFT);
    let shift_f10 = Event::KeyPressed {
        key: Key::Named(Named::F10),
        modified_key: Key::Named(Named::F10),
        physical_key: iced::keyboard::key::Physical::Code(iced::keyboard::key::Code::F10),
        location: iced::keyboard::Location::Standard,
        modifiers: shift_mod,
        repeat: false,
        text: None,
    };
    let _ = app.update(Message::KeyboardEvent(shift_f10));
    assert!(app.state.context_menu.is_open);
    app.state.context_menu.is_open = false;

    // 2. Alt+Enter opens Properties
    let mut alt_mod = Modifiers::empty();
    alt_mod.insert(Modifiers::ALT);
    let alt_enter = Event::KeyPressed {
        key: Key::Named(Named::Enter),
        modified_key: Key::Named(Named::Enter),
        physical_key: iced::keyboard::key::Physical::Code(iced::keyboard::key::Code::Enter),
        location: iced::keyboard::Location::Standard,
        modifiers: alt_mod,
        repeat: false,
        text: None,
    };
    let _ = app.update(Message::KeyboardEvent(alt_enter));
    assert!(app.state.properties_modal.is_some());
}

#[test]
fn test_desktop_views_rendering() {
    let test_item = SearchResultItem {
        id: 10,
        name: "test.rs".to_string(),
        path: "/home/user/src/test.rs".to_string(),
        size_bytes: 2048,
        mtime_secs: 1700000000,
        is_dir: false,
    };

    // 1. Render Context Menu Component
    {
        let _menu_element = views::context_menu::view(&test_item);
    }

    // 2. Render Properties Modal Component
    let props = FileProperties::from_item(&test_item, None);
    {
        let _props_element = views::properties::view(&props);
    }

    // 3. Render FindMeApp view with active Context Menu
    let mut app = FindMeApp::new();
    app.state.context_menu.is_open = true;
    app.state.context_menu.target_item = Some(test_item.clone());
    {
        let _app_view_menu = app.view();
    }

    // 4. Render FindMeApp view with active Properties Modal
    let mut app2 = FindMeApp::new();
    app2.state.properties_modal = Some(props);
    {
        let _app_view_modal = app2.view();
    }
}
