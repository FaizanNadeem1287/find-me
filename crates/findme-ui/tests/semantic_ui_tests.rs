use findme_semantic::{MatchKind, SearchMode, SemanticHit, ThrottleMode};
use findme_ui::app::FindMeApp;
use findme_ui::engine::EngineBridge;
use findme_ui::message::Message;
use findme_ui::views;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn test_search_mode_pills_rendering_and_interaction() {
    let mut app = FindMeApp::new();
    assert_eq!(app.state.search_mode, SearchMode::Everything);

    // Switch to Semantic mode
    let _ = app.update(Message::SetSearchMode(SearchMode::Semantic));
    assert_eq!(app.state.search_mode, SearchMode::Semantic);

    // Switch to Content mode
    let _ = app.update(Message::SetSearchMode(SearchMode::Content));
    assert_eq!(app.state.search_mode, SearchMode::Content);

    // Switch to Hybrid mode
    let _ = app.update(Message::SetSearchMode(SearchMode::Hybrid));
    assert_eq!(app.state.search_mode, SearchMode::Hybrid);

    // Verify search view renders cleanly across all search modes
    for mode in [
        SearchMode::Everything,
        SearchMode::Content,
        SearchMode::Semantic,
        SearchMode::Hybrid,
    ] {
        app.state.search_mode = mode;
        let element = views::search::view(&app.state);
        let _ = element.as_widget();
    }
}

#[test]
fn test_semantic_results_received_and_table_rendering() {
    let mut app = FindMeApp::new();

    let sample_hits = vec![
        SemanticHit {
            path: PathBuf::from("/home/user/Faizan_Resume.pdf"),
            score: 0.96,
            match_kind: MatchKind::SemanticMatch,
            location: "Page 1".to_string(),
            best_snippet: "... Muhammad Faizan Nadeem | Lead AI Engineer | Rust RAG ..."
                .to_string(),
            vector_id: 1,
            mtime: 1700000000,
            size_bytes: 142000,
            page_or_line: Some(1),
            audio_timestamp: None,
        },
        SemanticHit {
            path: PathBuf::from("/home/user/Sprint_Sync.ogg"),
            score: 0.88,
            match_kind: MatchKind::AudioTranscript,
            location: "[01:30 - 02:00]".to_string(),
            best_snippet: "... Discussion on offline speech-to-text audio indexing ...".to_string(),
            vector_id: 2,
            mtime: 1700005000,
            size_bytes: 840000,
            page_or_line: None,
            audio_timestamp: Some(90.0),
        },
        SemanticHit {
            path: PathBuf::from("/home/user/database_config.rs"),
            score: 3.4,
            match_kind: MatchKind::ExactContent,
            location: "Lines 1-25".to_string(),
            best_snippet: "... pub struct DatabaseSecretKey { token: String } ...".to_string(),
            vector_id: 3,
            mtime: 1700010000,
            size_bytes: 1200,
            page_or_line: Some(1),
            audio_timestamp: None,
        },
    ];

    let _ = app.update(Message::SemanticResultsReceived {
        results: sample_hits,
        search_duration_ms: 12,
    });

    assert_eq!(app.state.results.len(), 3);
    assert_eq!(app.state.semantic_results.len(), 3);
    assert_eq!(app.state.last_search_duration_ms, 12);
    assert_eq!(app.state.selected_row, Some(0));

    // Verify results table view renders with match badges and snippets without panic
    let table_elem = views::table::view(&app.state);
    let _ = table_elem.as_widget();
}

#[test]
fn test_snippet_expansion_and_audio_jump() {
    let mut app = FindMeApp::new();

    // Toggle snippet expansion
    assert!(!app.state.expanded_snippets.contains(&0));
    let _ = app.update(Message::ToggleSnippetExpansion(0));
    assert!(app.state.expanded_snippets.contains(&0));
    let _ = app.update(Message::ToggleSnippetExpansion(0));
    assert!(!app.state.expanded_snippets.contains(&0));

    // Audio timestamp jump
    let _ = app.update(Message::JumpAudioTimestamp {
        path: "/home/user/Voice_Note.mp3".to_string(),
        timestamp_secs: 154.0, // 2 mins 34 secs
    });

    assert!(app.state.notification.is_some());
    let note = app.state.notification.unwrap();
    assert!(note.contains("02:34"));
    assert!(note.contains("Voice_Note.mp3"));
}

#[test]
fn test_index_management_semantic_card_and_throttling() {
    let engine = Arc::new(EngineBridge::new());
    let mut app = FindMeApp::with_engine(engine);

    // Toggle throttle mode
    let _ = app.update(Message::SetThrottleMode(ThrottleMode::BatterySaver));
    assert_eq!(app.state.throttle_mode, ThrottleMode::BatterySaver);

    let _ = app.update(Message::SetThrottleMode(ThrottleMode::MaxPerformance));
    assert_eq!(app.state.throttle_mode, ThrottleMode::MaxPerformance);

    // Toggle pause/resume
    assert!(!app.state.semantic_progress.is_paused);
    let _ = app.update(Message::ToggleSemanticPause);
    assert!(app.state.semantic_progress.is_paused);
    let _ = app.update(Message::ToggleSemanticPause);
    assert!(!app.state.semantic_progress.is_paused);

    // Verify index management view renders cleanly
    let mgmt_elem = views::index_mgmt::view(&app.state);
    let _ = mgmt_elem.as_widget();
}

#[test]
fn test_live_semantic_indexing_progress_and_completion_events() {
    let mut app = FindMeApp::new();

    // 1. Send live indexing progress event
    let progress = findme_semantic::IndexerProgress {
        total_discovered: 40,
        total_processed: 25,
        indexed_chunks: 68,
        current_file: Some(PathBuf::from("/home/user/Downloads/report.pdf")),
        is_indexing: true,
        is_paused: false,
        throttle_mode: ThrottleMode::Balanced,
    };

    let _ = app.update(Message::Engine(
        findme_ui::engine::EngineEvent::SemanticProgress(progress),
    ));

    assert!(app.state.semantic_progress.is_indexing);
    assert_eq!(app.state.semantic_progress.total_processed, 25);
    assert_eq!(app.state.semantic_progress.total_discovered, 40);
    assert_eq!(app.state.semantic_progress.indexed_chunks, 68);

    // Verify index management view renders live progress smoothly
    {
        let mgmt_view = views::index_mgmt::view(&app.state);
        let _ = mgmt_view.as_widget();
    }

    // 2. Send indexing complete event
    let ready_event = findme_ui::engine::EngineEvent::SemanticIndexReady {
        indexed_chunks: 110,
        total_files: 40,
    };

    let _ = app.update(Message::Engine(ready_event));

    assert!(!app.state.semantic_progress.is_indexing);
    assert_eq!(app.state.semantic_chunks_count, 110);
    assert!(app.state.notification.is_some());
    let note = app.state.notification.as_ref().unwrap();
    assert!(note.contains("Semantic vector indexing complete"));
    assert!(note.contains("40"));
    assert!(note.contains("110"));

    // Verify index management view renders completion state
    {
        let mgmt_view_done = views::index_mgmt::view(&app.state);
        let _ = mgmt_view_done.as_widget();
    }
}

#[test]
fn test_semantic_type_column_sorts_by_score() {
    let mut app = FindMeApp::new();

    let hits = vec![
        SemanticHit {
            path: PathBuf::from("/home/user/document_b.html"),
            score: 0.20,
            match_kind: MatchKind::SemanticMatch,
            location: "Lines 1-10".to_string(),
            best_snippet: "... b ...".to_string(),
            vector_id: 1,
            mtime: 1700000000,
            size_bytes: 1000,
            page_or_line: Some(1),
            audio_timestamp: None,
        },
        SemanticHit {
            path: PathBuf::from("/home/user/document_a.pdf"),
            score: 0.90,
            match_kind: MatchKind::SemanticMatch,
            location: "Page 1".to_string(),
            best_snippet: "... a ...".to_string(),
            vector_id: 2,
            mtime: 1700000000,
            size_bytes: 5000,
            page_or_line: Some(1),
            audio_timestamp: None,
        },
    ];

    let _ = app.update(Message::SemanticResultsReceived {
        results: hits,
        search_duration_ms: 5,
    });

    // When sorting by ColumnId::Type in semantic mode, highest score comes first
    let _ = app.update(Message::SortByColumn(findme_ui::message::ColumnId::Type));
    assert_eq!(app.state.results[0].name, "document_a.pdf");
    assert_eq!(app.state.results[1].name, "document_b.html");
}
