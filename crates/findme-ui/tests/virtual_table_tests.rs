//! Comprehensive integration test suite for Task 5.2:
//! Virtualized 60 FPS tabular results view, column sorting, and keyboard navigation.

use findme_core::SearchResultItem;
use findme_ui::{
    compute_virtual_range, item_type_str, ColumnId, FindMeApp, Message, OVERSCAN_ROWS, ROW_HEIGHT,
};
use iced::keyboard::{self, key, Key, Modifiers};
use std::time::Instant;

fn mock_item(
    id: u64,
    name: &str,
    path: &str,
    size_bytes: u64,
    mtime_secs: u64,
    is_dir: bool,
) -> SearchResultItem {
    SearchResultItem {
        id,
        name: name.to_string(),
        path: path.to_string(),
        size_bytes,
        mtime_secs,
        is_dir,
    }
}

#[test]
fn test_compute_virtual_range_edge_cases() {
    // 1. Zero rows
    assert_eq!(
        compute_virtual_range(0, 0.0, 600.0, ROW_HEIGHT, OVERSCAN_ROWS),
        (0, 0, 0.0, 0.0)
    );

    // 2. Negative or invalid row height
    assert_eq!(
        compute_virtual_range(100, 0.0, 600.0, 0.0, OVERSCAN_ROWS),
        (0, 0, 0.0, 0.0)
    );

    // 3. Small list: 10 items, 0 scroll, 600 viewport
    let (start, end, top, bottom) =
        compute_virtual_range(10, 0.0, 600.0, ROW_HEIGHT, OVERSCAN_ROWS);
    assert_eq!(start, 0);
    assert_eq!(end, 10);
    assert_eq!(top, 0.0);
    assert_eq!(bottom, 0.0);

    // 4. Medium list: 1,000 items, scrolled to 3,000px (row 100), 600px viewport (20 rows)
    let total = 1_000;
    let (start, end, top, bottom) = compute_virtual_range(total, 3_000.0, 600.0, 30.0, 5);
    assert_eq!(start, 95); // 100 - overscan 5
    assert_eq!(end, 125); // 95 + 20 visible + 10 overscan = 125
    assert_eq!(top, 95.0 * 30.0);
    assert_eq!(bottom, (1_000.0 - 125.0) * 30.0);

    // Exact height invariant
    let total_layout_height = top + ((end - start) as f32 * 30.0) + bottom;
    assert_eq!(total_layout_height, total as f32 * 30.0);

    // 5. 100,000 items scrolled to middle (1,500,000px = row 50,000)
    let big_total = 100_000;
    let (b_start, b_end, b_top, b_bottom) =
        compute_virtual_range(big_total, 1_500_000.0, 600.0, 30.0, 5);
    assert_eq!(b_start, 49_995);
    assert_eq!(b_end, 50_025);
    assert_eq!(b_end - b_start, 30); // strictly 30 rows instantiated
    let big_layout_height = b_top + ((b_end - b_start) as f32 * 30.0) + b_bottom;
    assert_eq!(big_layout_height, big_total as f32 * 30.0);
}

#[test]
fn test_item_type_str_categories() {
    let dir = mock_item(1, "src", "/home/user/src", 4096, 1700000000, true);
    assert_eq!(item_type_str(&dir), "Folder");

    let no_ext = mock_item(2, "LICENSE", "/home/user/LICENSE", 1024, 1700000000, false);
    assert_eq!(item_type_str(&no_ext), "File");

    let rust_file = mock_item(3, "main.rs", "/home/user/main.rs", 2048, 1700000000, false);
    assert_eq!(item_type_str(&rust_file), "Rust");

    let toml_file = mock_item(
        4,
        "Cargo.toml",
        "/home/user/Cargo.toml",
        500,
        1700000000,
        false,
    );
    assert_eq!(item_type_str(&toml_file), "TOML");

    let json_file = mock_item(
        5,
        "theme.json",
        "/home/user/theme.json",
        300,
        1700000000,
        false,
    );
    assert_eq!(item_type_str(&json_file), "JSON");

    let md_file = mock_item(
        6,
        "README.md",
        "/home/user/README.md",
        4500,
        1700000000,
        false,
    );
    assert_eq!(item_type_str(&md_file), "Markdown");

    let text_file = mock_item(
        7,
        "notes.txt",
        "/home/user/notes.txt",
        120,
        1700000000,
        false,
    );
    assert_eq!(item_type_str(&text_file), "Text");

    let pdf_file = mock_item(
        8,
        "manual.pdf",
        "/home/user/manual.pdf",
        900000,
        1700000000,
        false,
    );
    assert_eq!(item_type_str(&pdf_file), "PDF");

    let img_file = mock_item(
        9,
        "logo.png",
        "/home/user/logo.png",
        54000,
        1700000000,
        false,
    );
    assert_eq!(item_type_str(&img_file), "PNG Image");

    let archive_file = mock_item(
        10,
        "bundle.tar.gz",
        "/home/user/bundle.tar.gz",
        2000000,
        1700000000,
        false,
    );
    assert_eq!(item_type_str(&archive_file), "Archive");

    let deb_file = mock_item(
        11,
        "find-me.deb",
        "/home/user/find-me.deb",
        4000000,
        1700000000,
        false,
    );
    assert_eq!(item_type_str(&deb_file), "Debian Pkg");

    let sh_file = mock_item(
        12,
        "install.sh",
        "/home/user/install.sh",
        800,
        1700000000,
        false,
    );
    assert_eq!(item_type_str(&sh_file), "Shell Script");
}

#[test]
fn test_column_sorting_all_columns_asc_and_desc() {
    let mut app = FindMeApp::new();

    let items = vec![
        mock_item(
            1,
            "charlie.rs",
            "/home/user/src/charlie.rs",
            10_000,
            1700000003,
            false,
        ),
        mock_item(
            2,
            "alice.toml",
            "/etc/findme/alice.toml",
            500,
            1700000001,
            false,
        ),
        mock_item(3, "bob.md", "/var/log/bob.md", 50_000, 1700000002, false),
        mock_item(4, "build", "/home/user/build", 4096, 1700000004, true),
    ];

    let _ = app.update(Message::ResultsReceived {
        results: items,
        search_duration_ms: 1,
        total_matches: 4,
    });

    // 1. Sort by Name Ascending (default after receive is Name Ascending)
    assert_eq!(app.state.sort_column, ColumnId::Name);
    assert!(app.state.sort_ascending);
    assert_eq!(app.state.results[0].name, "alice.toml");
    assert_eq!(app.state.results[3].name, "charlie.rs");

    // 2. Sort by Name Descending
    let _ = app.update(Message::SortByColumn(ColumnId::Name));
    assert!(!app.state.sort_ascending);
    assert_eq!(app.state.results[0].name, "charlie.rs");
    assert_eq!(app.state.results[3].name, "alice.toml");

    // 3. Sort by Size Ascending
    let _ = app.update(Message::SortByColumn(ColumnId::Size));
    assert_eq!(app.state.sort_column, ColumnId::Size);
    assert!(app.state.sort_ascending);
    assert_eq!(app.state.results[0].size_bytes, 500);
    assert_eq!(app.state.results[3].size_bytes, 50_000);

    // 4. Sort by Size Descending
    let _ = app.update(Message::SortByColumn(ColumnId::Size));
    assert!(!app.state.sort_ascending);
    assert_eq!(app.state.results[0].size_bytes, 50_000);
    assert_eq!(app.state.results[3].size_bytes, 500);

    // 5. Sort by Modified Ascending
    let _ = app.update(Message::SortByColumn(ColumnId::Modified));
    assert_eq!(app.state.sort_column, ColumnId::Modified);
    assert!(app.state.sort_ascending);
    assert_eq!(app.state.results[0].mtime_secs, 1700000001);
    assert_eq!(app.state.results[3].mtime_secs, 1700000004);

    // 6. Sort by Modified Descending
    let _ = app.update(Message::SortByColumn(ColumnId::Modified));
    assert!(!app.state.sort_ascending);
    assert_eq!(app.state.results[0].mtime_secs, 1700000004);
    assert_eq!(app.state.results[3].mtime_secs, 1700000001);

    // 7. Sort by Path Ascending
    let _ = app.update(Message::SortByColumn(ColumnId::Path));
    assert_eq!(app.state.sort_column, ColumnId::Path);
    assert!(app.state.sort_ascending);
    assert_eq!(app.state.results[0].path, "/etc/findme/alice.toml");
    assert_eq!(app.state.results[3].path, "/var/log/bob.md");

    // 8. Sort by Type Ascending
    let _ = app.update(Message::SortByColumn(ColumnId::Type));
    assert_eq!(app.state.sort_column, ColumnId::Type);
    assert!(app.state.sort_ascending);
    assert_eq!(item_type_str(&app.state.results[0]), "Folder");
}

#[test]
fn test_keyboard_navigation_selection_stepping() {
    let mut app = FindMeApp::new();

    let items: Vec<SearchResultItem> = (0..50)
        .map(|i| {
            mock_item(
                i,
                &format!("file_{i}.txt"),
                &format!("/tmp/file_{i}.txt"),
                100 * i,
                1700000000 + i,
                false,
            )
        })
        .collect();

    let _ = app.update(Message::ResultsReceived {
        results: items,
        search_duration_ms: 1,
        total_matches: 50,
    });

    assert_eq!(app.state.selected_row, Some(0));

    let key_down = keyboard::Event::KeyPressed {
        key: Key::Named(key::Named::ArrowDown),
        modified_key: Key::Named(key::Named::ArrowDown),
        physical_key: key::Physical::Unidentified(key::NativeCode::Unidentified),
        location: keyboard::Location::Standard,
        modifiers: Modifiers::empty(),
        text: None,
        repeat: false,
    };

    let key_up = keyboard::Event::KeyPressed {
        key: Key::Named(key::Named::ArrowUp),
        modified_key: Key::Named(key::Named::ArrowUp),
        physical_key: key::Physical::Unidentified(key::NativeCode::Unidentified),
        location: keyboard::Location::Standard,
        modifiers: Modifiers::empty(),
        text: None,
        repeat: false,
    };

    // Step Down
    let _ = app.update(Message::KeyboardEvent(key_down.clone()));
    assert_eq!(app.state.selected_row, Some(1));

    let _ = app.update(Message::KeyboardEvent(key_down.clone()));
    assert_eq!(app.state.selected_row, Some(2));

    // Step Up
    let _ = app.update(Message::KeyboardEvent(key_up.clone()));
    assert_eq!(app.state.selected_row, Some(1));

    let _ = app.update(Message::KeyboardEvent(key_up.clone()));
    assert_eq!(app.state.selected_row, Some(0));

    // Underflow protection: up at 0 stays at 0
    let _ = app.update(Message::KeyboardEvent(key_up));
    assert_eq!(app.state.selected_row, Some(0));

    // PageDown jumps 15 rows
    let page_down = keyboard::Event::KeyPressed {
        key: Key::Named(key::Named::PageDown),
        modified_key: Key::Named(key::Named::PageDown),
        physical_key: key::Physical::Unidentified(key::NativeCode::Unidentified),
        location: keyboard::Location::Standard,
        modifiers: Modifiers::empty(),
        text: None,
        repeat: false,
    };
    let _ = app.update(Message::KeyboardEvent(page_down));
    assert_eq!(app.state.selected_row, Some(15));

    // PageUp jumps 15 rows back
    let page_up = keyboard::Event::KeyPressed {
        key: Key::Named(key::Named::PageUp),
        modified_key: Key::Named(key::Named::PageUp),
        physical_key: key::Physical::Unidentified(key::NativeCode::Unidentified),
        location: keyboard::Location::Standard,
        modifiers: Modifiers::empty(),
        text: None,
        repeat: false,
    };
    let _ = app.update(Message::KeyboardEvent(page_up));
    assert_eq!(app.state.selected_row, Some(0));

    // End jumps to last item
    let end_key = keyboard::Event::KeyPressed {
        key: Key::Named(key::Named::End),
        modified_key: Key::Named(key::Named::End),
        physical_key: key::Physical::Unidentified(key::NativeCode::Unidentified),
        location: keyboard::Location::Standard,
        modifiers: Modifiers::empty(),
        text: None,
        repeat: false,
    };
    let _ = app.update(Message::KeyboardEvent(end_key));
    assert_eq!(app.state.selected_row, Some(49));

    // Home jumps to 0
    let home_key = keyboard::Event::KeyPressed {
        key: Key::Named(key::Named::Home),
        modified_key: Key::Named(key::Named::Home),
        physical_key: key::Physical::Unidentified(key::NativeCode::Unidentified),
        location: keyboard::Location::Standard,
        modifiers: Modifiers::empty(),
        text: None,
        repeat: false,
    };
    let _ = app.update(Message::KeyboardEvent(home_key));
    assert_eq!(app.state.selected_row, Some(0));
}

#[test]
fn test_keyboard_enter_file_activation() {
    let mut app = FindMeApp::new();

    let items = vec![mock_item(
        1,
        "document.pdf",
        "/home/user/document.pdf",
        1024,
        1700000000,
        false,
    )];
    let _ = app.update(Message::ResultsReceived {
        results: items,
        search_duration_ms: 1,
        total_matches: 1,
    });
    assert_eq!(app.state.selected_row, Some(0));

    // Enter key activates OpenFile without panic
    let enter_key = keyboard::Event::KeyPressed {
        key: Key::Named(key::Named::Enter),
        modified_key: Key::Named(key::Named::Enter),
        physical_key: key::Physical::Unidentified(key::NativeCode::Unidentified),
        location: keyboard::Location::Standard,
        modifiers: Modifiers::empty(),
        text: None,
        repeat: false,
    };
    let _ = app.update(Message::KeyboardEvent(enter_key));

    // Alt + Enter activates OpenContainingFolder without panic
    let mut alt_mod = Modifiers::empty();
    alt_mod.set(Modifiers::ALT, true);
    let alt_enter_key = keyboard::Event::KeyPressed {
        key: Key::Named(key::Named::Enter),
        modified_key: Key::Named(key::Named::Enter),
        physical_key: key::Physical::Unidentified(key::NativeCode::Unidentified),
        location: keyboard::Location::Standard,
        modifiers: alt_mod,
        text: None,
        repeat: false,
    };
    let _ = app.update(Message::KeyboardEvent(alt_enter_key));
}

#[test]
fn test_table_scrolled_message_updates_state() {
    let mut app = FindMeApp::new();
    assert_eq!(app.state.scroll_y, 0.0);

    let _ = app.update(Message::TableScrolled {
        scroll_y: 450.0,
        viewport_height: 720.0,
    });

    assert_eq!(app.state.scroll_y, 450.0);
    assert_eq!(app.state.table_viewport_height, 720.0);
}

#[test]
fn test_100_000_items_stress_and_rendering() {
    let mut app = FindMeApp::new();

    // Generate 100,000 mock file records
    let gen_start = Instant::now();
    let big_items: Vec<SearchResultItem> = (0..100_000)
        .map(|i| {
            mock_item(
                i,
                &format!("file_{i:06}.rs"),
                &format!("/workspace/project/src/module_{}/file_{i:06}.rs", i % 100),
                (i * 137) % 1_000_000,
                1700000000 + (i % 86400),
                i % 10 == 0,
            )
        })
        .collect();
    let gen_time = gen_start.elapsed();
    println!("Generated 100,000 items in {gen_time:?}");

    app.state.results = big_items;
    assert_eq!(app.state.results.len(), 100_000);

    // Initial view rendering at top (scroll_y = 0.0)
    let render_start = Instant::now();
    {
        let _ = app.view();
    }
    let render_time = render_start.elapsed();
    println!("100,000 items view rendering took {render_time:?}");
    // Must render well within 60 FPS frame budget (<16.6ms, target <5ms)
    assert!(
        render_time.as_millis() < 10,
        "Rendering 100,000 items took {render_time:?}, exceeding 10ms threshold"
    );

    // Scroll to middle (1,500,000px)
    let _ = app.update(Message::TableScrolled {
        scroll_y: 1_500_000.0,
        viewport_height: 800.0,
    });
    assert_eq!(app.state.scroll_y, 1_500_000.0);

    // Re-render at scrolled position
    let scroll_render_start = Instant::now();
    {
        let _ = app.view();
    }
    let scroll_render_time = scroll_render_start.elapsed();
    println!("Scrolled view rendering took {scroll_render_time:?}");
    assert!(
        scroll_render_time.as_millis() < 10,
        "Scrolled view rendering took {scroll_render_time:?}, exceeding 10ms threshold"
    );
}
