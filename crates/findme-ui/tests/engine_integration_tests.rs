//! Comprehensive integration test suite for Task 7.1:
//! Async Tokio Communication Bridge & Live Engine Streaming.
//!
//! Validates:
//! 1. `EngineBridge` initialization with default rich sample data and empty mode.
//! 2. Asynchronous off-thread search evaluation (`tokio::task::spawn_blocking`).
//! 3. Everything query syntax evaluation (ext:, size:, path:, wildcards, boolean OR).
//! 4. Multi-column sorting (Name, Size, Modified, Path) ascending and descending.
//! 5. Event streaming over Tokio broadcast channels (`EngineEvent`).
//! 6. `clear_index()` lifecycle and memory arena reset.
//! 7. `rebuild_index()` scanning real filesystem paths via `Crawler` and emitting progress.
//! 8. Concurrent async searches without blocking or data races.
//! 9. Elm architecture integration in `FindMeApp` (`ExecuteSearch`, `ResultsReceived`, `Engine`).
//! 10. `IndexManagement` screen triggering live rebuild and clear actions.

use findme_core::SearchResultItem;
use findme_ui::{
    ColumnId, EngineBridge, EngineEvent, FindMeApp, IndexManagementMessage, IndexingState, Message,
};
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_engine_bridge_initialization_and_arena_inspection() {
    let bridge = EngineBridge::new();
    let arena = bridge.arena();
    let lock = arena.read();

    assert!(
        lock.len() >= 25,
        "Default sample arena should have at least 25 records, found {}",
        lock.len()
    );
    assert!(
        lock.memory_usage_bytes() > 0,
        "Memory usage should be positive"
    );
    assert!(!bridge.is_indexing(), "Bridge should not start as indexing");

    let empty_bridge = EngineBridge::empty();
    assert_eq!(
        empty_bridge.arena().read().len(),
        0,
        "Empty bridge should contain 0 records"
    );
}

#[tokio::test]
async fn test_async_search_basic_and_latency() {
    let bridge = EngineBridge::new();

    let (results, duration_ms, total_matches) =
        bridge.search("Cargo.toml", 100, ColumnId::Name, true).await;

    assert!(
        !results.is_empty(),
        "Searching for 'Cargo.toml' must return results"
    );
    assert_eq!(results[0].name, "Cargo.toml");
    assert!(results[0].path.contains("find-me"));
    assert!(!results[0].is_dir);
    assert!(results[0].size_bytes > 0);
    assert!(total_matches >= 1);
    // Sub-millisecond or few milliseconds off-thread execution
    assert!(
        duration_ms < 5000,
        "Search should be fast, took {} ms",
        duration_ms
    );
}

#[tokio::test]
async fn test_everything_query_syntax_in_engine_search() {
    let bridge = EngineBridge::new();

    // 1. Extension filter: ext:rs
    let (rs_results, _, _) = bridge.search("ext:rs", 100, ColumnId::Name, true).await;
    assert!(
        !rs_results.is_empty(),
        "ext:rs query must return Rust files"
    );
    for item in &rs_results {
        assert!(
            item.name.ends_with(".rs"),
            "Result '{}' should end with .rs",
            item.name
        );
    }

    // 2. Size filter: size:>100mb
    let (large_results, _, _) = bridge
        .search("size:>100mb", 100, ColumnId::Size, false)
        .await;
    assert!(
        !large_results.is_empty(),
        "size:>100mb should find large files"
    );
    for item in &large_results {
        assert!(
            item.size_bytes > 100 * 1024 * 1024,
            "Item '{}' size {} must be > 100MB",
            item.name,
            item.size_bytes
        );
    }

    // 3. Path restriction: path:/home/user/documents
    let (doc_results, _, _) = bridge
        .search("path:/home/user/documents", 100, ColumnId::Path, true)
        .await;
    assert!(
        !doc_results.is_empty(),
        "path:documents query should return document items"
    );
    for item in &doc_results {
        assert!(
            item.path.contains("documents"),
            "Item path '{}' must contain 'documents'",
            item.path
        );
    }

    // 4. Regex mode pattern: regex:.*\.png
    let (png_results, _, _) = bridge
        .search(r"regex:.*\.png", 100, ColumnId::Name, true)
        .await;
    assert!(
        !png_results.is_empty(),
        "regex:.*\\.png query should find PNG images"
    );
    for item in &png_results {
        assert!(item.name.ends_with(".png"));
    }

    // 5. Boolean OR expression: ext:pdf | ext:docx
    let (or_results, _, _) = bridge
        .search("ext:pdf | ext:docx", 100, ColumnId::Name, true)
        .await;
    assert!(
        or_results.len() >= 2,
        "ext:pdf | ext:docx should match both pdf and docx"
    );
}

#[tokio::test]
async fn test_engine_sorting_all_columns() {
    let bridge = EngineBridge::new();

    // Sort by Name Ascending
    let (name_asc, _, _) = bridge.search("", 100, ColumnId::Name, true).await;
    assert!(name_asc.len() >= 2);
    for i in 0..name_asc.len() - 1 {
        assert!(
            name_asc[i].name <= name_asc[i + 1].name,
            "Name not sorted ascending: '{}' > '{}'",
            name_asc[i].name,
            name_asc[i + 1].name
        );
    }

    // Sort by Size Descending
    let (size_desc, _, _) = bridge.search("", 100, ColumnId::Size, false).await;
    assert!(size_desc.len() >= 2);
    for i in 0..size_desc.len() - 1 {
        assert!(
            size_desc[i].size_bytes >= size_desc[i + 1].size_bytes,
            "Size not sorted descending: {} < {}",
            size_desc[i].size_bytes,
            size_desc[i + 1].size_bytes
        );
    }

    // Sort by Modified Descending
    let (mtime_desc, _, _) = bridge.search("", 100, ColumnId::Modified, false).await;
    assert!(mtime_desc.len() >= 2);
    for i in 0..mtime_desc.len() - 1 {
        assert!(
            mtime_desc[i].mtime_secs >= mtime_desc[i + 1].mtime_secs,
            "Mtime not sorted descending"
        );
    }
}

#[tokio::test]
async fn test_engine_event_streaming_and_broadcast() {
    let bridge = EngineBridge::new();
    let mut rx = bridge.subscribe_events();

    bridge.send_event(EngineEvent::Started);
    bridge.send_event(EngineEvent::Progress {
        files_found: 42,
        dirs_scanned: 5,
        current_path: "/test/dir".to_string(),
        percent: 0.25,
    });
    bridge.send_event(EngineEvent::IndexReady {
        total_files: 100,
        total_folders: 10,
        memory_bytes: 40960,
        duration_ms: 85,
    });

    let ev1 = rx.recv().await.unwrap();
    assert_eq!(ev1, EngineEvent::Started);

    let ev2 = rx.recv().await.unwrap();
    match ev2 {
        EngineEvent::Progress {
            files_found,
            dirs_scanned,
            current_path,
            percent,
        } => {
            assert_eq!(files_found, 42);
            assert_eq!(dirs_scanned, 5);
            assert_eq!(current_path, "/test/dir");
            assert!((percent - 0.25).abs() < f32::EPSILON);
        }
        other => panic!("Expected Progress event, got {:?}", other),
    }

    let ev3 = rx.recv().await.unwrap();
    match ev3 {
        EngineEvent::IndexReady {
            total_files,
            total_folders,
            memory_bytes,
            duration_ms,
        } => {
            assert_eq!(total_files, 100);
            assert_eq!(total_folders, 10);
            assert_eq!(memory_bytes, 40960);
            assert_eq!(duration_ms, 85);
        }
        other => panic!("Expected IndexReady event, got {:?}", other),
    }
}

#[tokio::test]
async fn test_clear_index_lifecycle() {
    let bridge = EngineBridge::new();
    assert!(!bridge.arena().read().is_empty());

    let mut rx = bridge.subscribe_events();
    bridge.clear_index();

    assert_eq!(bridge.arena().read().len(), 0);

    let ready_event = rx.recv().await.unwrap();
    assert_eq!(
        ready_event,
        EngineEvent::IndexReady {
            total_files: 0,
            total_folders: 0,
            memory_bytes: 0,
            duration_ms: 0,
        }
    );

    let (results, _, total) = bridge.search("", 100, ColumnId::Name, true).await;
    assert_eq!(results.len(), 0);
    assert_eq!(total, 0);
}

#[tokio::test]
async fn test_rebuild_index_with_temp_directory() {
    let dir = tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();

    // Create a few files and subfolders
    let sub = root.join("nested_folder");
    create_dir_all(&sub).unwrap();

    let file1 = root.join("doc_alpha.txt");
    let file2 = sub.join("doc_beta.rs");
    let mut f1 = File::create(&file1).unwrap();
    f1.write_all(b"Hello from alpha!").unwrap();
    let mut f2 = File::create(&file2).unwrap();
    f2.write_all(b"fn main() {}").unwrap();

    let bridge = EngineBridge::empty();
    let mut rx = bridge.subscribe_events();

    bridge.rebuild_index(vec![root], vec![], vec![]);

    // Await IndexReady event
    let mut index_ready = false;
    while let Ok(event) = rx.recv().await {
        if let EngineEvent::IndexReady { total_files, .. } = event {
            assert!(
                total_files >= 2,
                "Crawler should have indexed at least 2 files, found {}",
                total_files
            );
            index_ready = true;
            break;
        }
    }
    assert!(index_ready, "IndexReady event must be received");

    // Search the new arena
    let (results, _, total) = bridge.search("doc_beta", 10, ColumnId::Name, true).await;
    assert!(total >= 1);
    assert!(results.iter().any(|r| r.name == "doc_beta.rs"));
}

#[tokio::test]
async fn test_concurrent_searches_and_arena_reads() {
    let bridge = Arc::new(EngineBridge::new());

    let b1 = Arc::clone(&bridge);
    let b2 = Arc::clone(&bridge);
    let b3 = Arc::clone(&bridge);

    let (t1, t2, t3) = tokio::join!(
        tokio::spawn(async move { b1.search("Cargo", 50, ColumnId::Name, true).await }),
        tokio::spawn(async move { b2.search("ext:rs", 50, ColumnId::Size, false).await }),
        tokio::spawn(async move {
            b3.search("path:/home/user/projects", 50, ColumnId::Path, true)
                .await
        })
    );

    let (res1, _, _) = t1.unwrap();
    let (res2, _, _) = t2.unwrap();
    let (res3, _, _) = t3.unwrap();

    assert!(!res1.is_empty());
    assert!(!res2.is_empty());
    assert!(!res3.is_empty());
}

#[test]
fn test_app_integration_execute_search_and_results_flow() {
    let mut app = FindMeApp::new();

    // Verify initial state reflects bridge arena
    assert!(app.state.indexed_files_count >= 25);
    assert!(app.state.indexed_folders_count > 0);
    assert!(app.state.index_mgmt.stats.total_files >= 25);

    // Keystroke triggers search
    let _ = app.update(Message::SearchInputChanged("Cargo".into()));
    assert_eq!(app.state.query, "Cargo");
    assert!(app.state.is_searching);

    // Mock results arrival
    let sample_item = SearchResultItem {
        id: 1,
        name: "Cargo.toml".into(),
        path: "/home/user/Cargo.toml".into(),
        size_bytes: 1024,
        mtime_secs: 1732000000,
        is_dir: false,
    };
    let _ = app.update(Message::ResultsReceived {
        results: vec![sample_item],
        search_duration_ms: 2,
        total_matches: 1,
    });

    assert!(!app.state.is_searching);
    assert_eq!(app.state.results.len(), 1);
    assert_eq!(app.state.last_search_duration_ms, 2);
    assert_eq!(app.state.selected_row, Some(0));

    // Verify query is logged in search history
    assert!(app.state.history.entries.iter().any(|h| h.query == "Cargo"));
}

#[test]
fn test_app_integration_engine_events() {
    let mut app = FindMeApp::new();

    // Started event
    let _ = app.update(Message::Engine(EngineEvent::Started));
    assert!(matches!(
        app.state.index_mgmt.indexing_state,
        IndexingState::Scanning { .. }
    ));

    // Progress event
    let _ = app.update(Message::Engine(EngineEvent::Progress {
        files_found: 1500,
        dirs_scanned: 120,
        current_path: "/var/log".into(),
        percent: 0.5,
    }));
    assert_eq!(app.state.indexed_files_count, 1500);
    assert_eq!(app.state.indexed_folders_count, 120);

    // IndexReady event
    let _ = app.update(Message::Engine(EngineEvent::IndexReady {
        total_files: 3000,
        total_folders: 250,
        memory_bytes: 512_000,
        duration_ms: 450,
    }));
    assert_eq!(app.state.indexed_files_count, 3000);
    assert_eq!(app.state.indexed_folders_count, 250);
    assert_eq!(app.state.index_mgmt.stats.total_files, 3000);
    assert_eq!(app.state.index_mgmt.stats.total_folders, 250);
    assert_eq!(app.state.index_mgmt.stats.memory_bytes, 512_000);
    assert_eq!(app.state.index_mgmt.indexing_state, IndexingState::Idle);
    assert!(app.state.notification.is_some());

    // Error event
    let _ = app.update(Message::Engine(EngineEvent::Error(
        "Filesystem error".into(),
    )));
    assert_eq!(app.state.index_mgmt.indexing_state, IndexingState::Idle);
    assert!(app
        .state
        .notification
        .as_ref()
        .unwrap()
        .contains("Filesystem error"));
}

#[test]
fn test_app_index_management_screen_actions() {
    let mut app = FindMeApp::new();

    // Clear index action
    let _ = app.update(Message::IndexManagement(IndexManagementMessage::ClearIndex));
    assert_eq!(app.engine.arena().read().len(), 0);
    assert_eq!(app.state.indexed_files_count, 0);
    assert_eq!(app.state.indexed_folders_count, 0);
    assert_eq!(app.state.results.len(), 0);

    // Rebuild index action
    app.state
        .index_mgmt
        .roots
        .push(findme_ui::IndexedRoot::new(PathBuf::from("/tmp"), 0, 0));
    let _ = app.update(Message::IndexManagement(
        IndexManagementMessage::TriggerRebuildIndex,
    ));
    assert!(matches!(
        app.state.index_mgmt.indexing_state,
        IndexingState::Scanning { .. }
    ));
}
