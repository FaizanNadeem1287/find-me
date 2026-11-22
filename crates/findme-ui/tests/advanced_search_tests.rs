//! Integration test suite for the Advanced Search visual form builder and query compiler

use findme_core::QueryParser;
use findme_ui::advanced::{AdvancedFilterState, DatePreset, PresetFilter, SizeUnit};
use findme_ui::app::FindMeApp;
use findme_ui::message::{AdvancedSearchMessage, Message, ScreenId};

#[test]
fn test_default_state_and_empty_query() {
    let state = AdvancedFilterState::default();
    assert_eq!(state.synthesize_query(), "");
    assert_eq!(state.date_preset, DatePreset::Any);
    assert_eq!(state.size_min_unit, SizeUnit::MB);
    assert_eq!(state.size_max_unit, SizeUnit::MB);
    assert!(!state.match_case);
    assert!(!state.regex_mode);
}

#[test]
fn test_keywords_query_synthesis() {
    // All words
    let state_all = AdvancedFilterState {
        all_words: "alpha beta gamma".to_string(),
        ..Default::default()
    };
    assert_eq!(state_all.synthesize_query(), "alpha beta gamma");

    // Exact phrase
    let state_exact = AdvancedFilterState {
        exact_phrase: "financial report 2026".to_string(),
        ..Default::default()
    };
    assert_eq!(state_exact.synthesize_query(), "\"financial report 2026\"");

    // Exact phrase already quoted
    let state_quoted = AdvancedFilterState {
        exact_phrase: "\"quoted phrase\"".to_string(),
        ..Default::default()
    };
    assert_eq!(state_quoted.synthesize_query(), "\"quoted phrase\"");

    // Any words (OR grouping)
    let state_any_single = AdvancedFilterState {
        any_words: "cat".to_string(),
        ..Default::default()
    };
    assert_eq!(state_any_single.synthesize_query(), "cat");

    let state_any_multi = AdvancedFilterState {
        any_words: "cat dog fish".to_string(),
        ..Default::default()
    };
    assert_eq!(state_any_multi.synthesize_query(), "(cat | dog | fish)");

    // None words (negation)
    let state_none = AdvancedFilterState {
        none_words: "draft temp backup".to_string(),
        ..Default::default()
    };
    assert_eq!(state_none.synthesize_query(), "!draft !temp !backup");

    // None words already negated
    let state_negated = AdvancedFilterState {
        none_words: "!draft !temp".to_string(),
        ..Default::default()
    };
    assert_eq!(state_negated.synthesize_query(), "!draft !temp");

    // Combined keywords
    let state_combined = AdvancedFilterState {
        all_words: "project".to_string(),
        exact_phrase: "final draft".to_string(),
        any_words: "pdf docx".to_string(),
        none_words: "backup".to_string(),
        ..Default::default()
    };
    let synthesized = state_combined.synthesize_query();
    assert!(synthesized.contains("project"));
    assert!(synthesized.contains("\"final draft\""));
    assert!(synthesized.contains("(pdf | docx)"));
    assert!(synthesized.contains("!backup"));
}

#[test]
fn test_extensions_synthesis() {
    let mut state = AdvancedFilterState {
        ext_code: true,
        ..Default::default()
    };
    assert_eq!(state.synthesize_query(), "ext:rs;c;cpp;py;js;ts;go;java");

    // Multiple categories + custom extensions
    state = AdvancedFilterState {
        ext_documents: true,
        custom_ext: "toml, json; md".to_string(), // md is already in documents, should deduplicate
        ..Default::default()
    };
    let query = state.synthesize_query();
    assert!(query.starts_with("ext:"));
    assert!(query.contains("doc"));
    assert!(query.contains("pdf"));
    assert!(query.contains("toml"));
    assert!(query.contains("json"));
    // Ensure md appears only once
    let count_md = query.matches("md").count();
    assert_eq!(count_md, 1, "Duplicate extensions should be filtered out");
}

#[test]
fn test_size_bounds_synthesis() {
    let state_mb_gb = AdvancedFilterState {
        size_min: "10".to_string(),
        size_min_unit: SizeUnit::MB,
        size_max: "2".to_string(),
        size_max_unit: SizeUnit::GB,
        ..Default::default()
    };
    assert_eq!(state_mb_gb.synthesize_query(), "size:>=10mb size:<=2gb");

    // Units B and KB
    let state_b_kb = AdvancedFilterState {
        size_min: "512".to_string(),
        size_min_unit: SizeUnit::B,
        size_max: "64".to_string(),
        size_max_unit: SizeUnit::KB,
        ..Default::default()
    };
    assert_eq!(state_b_kb.synthesize_query(), "size:>=512b size:<=64kb");

    // Invalid or zero values should be safely ignored
    let state_invalid = AdvancedFilterState {
        size_min: "invalid".to_string(),
        size_max: "0".to_string(),
        ..Default::default()
    };
    assert_eq!(state_invalid.synthesize_query(), "");
}

#[test]
fn test_date_presets_synthesis() {
    let presets = [
        (DatePreset::Any, ""),
        (DatePreset::Today, "dm:today"),
        (DatePreset::Yesterday, "dm:yesterday"),
        (DatePreset::PastWeek, "dm:lastweek"),
        (DatePreset::PastMonth, "dm:lastmonth"),
        (DatePreset::PastYear, "dm:lastyear"),
    ];

    for (preset, expected) in presets {
        let state = AdvancedFilterState {
            date_preset: preset,
            ..Default::default()
        };
        assert_eq!(state.synthesize_query(), expected);
    }
}

#[test]
fn test_path_restrictions_synthesis() {
    let state_normal = AdvancedFilterState {
        search_path: "/home/user/projects".to_string(),
        exclude_path: "/var/log".to_string(),
        ..Default::default()
    };
    assert_eq!(
        state_normal.synthesize_query(),
        "path:/home/user/projects !path:/var/log"
    );

    // Paths with spaces must be quoted
    let state_spaced = AdvancedFilterState {
        search_path: "/home/user/My Documents".to_string(),
        exclude_path: "/home/user/Virtual Machines".to_string(),
        ..Default::default()
    };
    assert_eq!(
        state_spaced.synthesize_query(),
        "path:\"/home/user/My Documents\" !path:\"/home/user/Virtual Machines\""
    );
}

#[test]
fn test_flags_synthesis() {
    let state = AdvancedFilterState {
        regex_mode: true,
        match_case: true,
        all_words: "Report".to_string(),
        ..Default::default()
    };
    assert_eq!(state.synthesize_query(), "regex: case: Report");
}

#[test]
fn test_preset_templates() {
    let mut state = AdvancedFilterState::default();

    // Large Code
    state.apply_preset(PresetFilter::LargeCode);
    assert!(state.ext_code);
    assert_eq!(state.size_min, "1");
    assert_eq!(state.size_min_unit, SizeUnit::MB);
    assert_eq!(
        state.synthesize_query(),
        "ext:rs;c;cpp;py;js;ts;go;java size:>=1mb"
    );

    // Recent Documents
    state.apply_preset(PresetFilter::RecentDocuments);
    assert!(state.ext_documents);
    assert_eq!(state.date_preset, DatePreset::PastWeek);
    assert_eq!(
        state.synthesize_query(),
        "ext:doc;docx;pdf;txt;md;odt dm:lastweek"
    );

    // Gigantic Files
    state.apply_preset(PresetFilter::GiganticFiles);
    assert_eq!(state.size_min, "1");
    assert_eq!(state.size_min_unit, SizeUnit::GB);
    assert_eq!(state.synthesize_query(), "size:>=1gb");

    // Archives Only
    state.apply_preset(PresetFilter::ArchivesOnly);
    assert!(state.ext_archives);
    assert_eq!(state.synthesize_query(), "ext:zip;tar;gz;bz2;xz;7z");
}

#[test]
fn test_app_message_dispatching_and_reset() {
    let mut app = FindMeApp::new();

    // Send updates through App update loop
    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::UpdateAllWords("hello".to_string()),
    ));
    assert_eq!(app.state.advanced_filters.all_words, "hello");

    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::UpdateExactPhrase("foo bar".to_string()),
    ));
    assert_eq!(app.state.advanced_filters.exact_phrase, "foo bar");

    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::UpdateAnyWords("a b".to_string()),
    ));
    assert_eq!(app.state.advanced_filters.any_words, "a b");

    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::UpdateNoneWords("c d".to_string()),
    ));
    assert_eq!(app.state.advanced_filters.none_words, "c d");

    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::ToggleExtDocuments(true),
    ));
    assert!(app.state.advanced_filters.ext_documents);

    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::ToggleExtCode(true),
    ));
    assert!(app.state.advanced_filters.ext_code);

    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::ToggleExtImages(true),
    ));
    assert!(app.state.advanced_filters.ext_images);

    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::ToggleExtAudio(true),
    ));
    assert!(app.state.advanced_filters.ext_audio);

    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::ToggleExtVideo(true),
    ));
    assert!(app.state.advanced_filters.ext_video);

    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::ToggleExtArchives(true),
    ));
    assert!(app.state.advanced_filters.ext_archives);

    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::UpdateCustomExt("rs, toml".to_string()),
    ));
    assert_eq!(app.state.advanced_filters.custom_ext, "rs, toml");

    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::UpdateSizeMin("5".to_string()),
    ));
    assert_eq!(app.state.advanced_filters.size_min, "5");

    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::SelectSizeMinUnit(SizeUnit::KB),
    ));
    assert_eq!(app.state.advanced_filters.size_min_unit, SizeUnit::KB);

    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::UpdateSizeMax("100".to_string()),
    ));
    assert_eq!(app.state.advanced_filters.size_max, "100");

    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::SelectSizeMaxUnit(SizeUnit::GB),
    ));
    assert_eq!(app.state.advanced_filters.size_max_unit, SizeUnit::GB);

    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::SelectDatePreset(DatePreset::Today),
    ));
    assert_eq!(app.state.advanced_filters.date_preset, DatePreset::Today);

    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::UpdateSearchPath("/tmp".to_string()),
    ));
    assert_eq!(app.state.advanced_filters.search_path, "/tmp");

    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::UpdateExcludePath("/dev".to_string()),
    ));
    assert_eq!(app.state.advanced_filters.exclude_path, "/dev");

    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::ToggleMatchCase(true),
    ));
    assert!(app.state.advanced_filters.match_case);

    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::ToggleMatchWholeWord(true),
    ));
    assert!(app.state.advanced_filters.match_whole_word);

    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::ToggleRegexMode(true),
    ));
    assert!(app.state.advanced_filters.regex_mode);

    // Apply template preset
    let _ = app.update(Message::AdvancedSearch(AdvancedSearchMessage::ApplyPreset(
        PresetFilter::LargeCode,
    )));
    assert!(app.state.advanced_filters.ext_code);
    assert_eq!(app.state.advanced_filters.size_min, "1");

    // Reset filters
    let _ = app.update(Message::AdvancedSearch(AdvancedSearchMessage::ResetFilters));
    assert_eq!(app.state.advanced_filters, AdvancedFilterState::default());
}

#[test]
fn test_submit_search_transitions_to_search_screen() {
    let mut app = FindMeApp::new();
    let _ = app.update(Message::SwitchScreen(ScreenId::AdvancedSearch));
    assert_eq!(app.state.active_screen, ScreenId::AdvancedSearch);

    // Configure search filters
    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::UpdateAllWords("config".to_string()),
    ));
    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::ToggleExtCode(true),
    ));

    // Submit Search
    let _ = app.update(Message::AdvancedSearch(AdvancedSearchMessage::SubmitSearch));

    // Verify screen switched to Search and query updated
    assert_eq!(app.state.active_screen, ScreenId::Search);
    assert_eq!(app.state.query, "config ext:rs;c;cpp;py;js;ts;go;java");
}

#[test]
fn test_synthesized_queries_parse_in_query_engine() {
    // Verify that every combination of synthesized query parses cleanly into QueryAst!
    let test_cases = [
        "",
        "report",
        "\"exact phrase\"",
        "(cat | dog | fish)",
        "!draft !temp",
        "ext:rs;c;cpp;py;js;ts;go;java",
        "size:>=10mb size:<=2gb",
        "dm:today",
        "dm:yesterday",
        "dm:lastweek",
        "dm:lastmonth",
        "dm:lastyear",
        "path:/home/user !path:/tmp",
        "case: regex: config ext:rs",
        "project \"final draft\" (pdf | docx) !backup ext:pdf;doc size:>=1mb dm:lastweek path:/home/faizan",
    ];

    for query in test_cases {
        let ast = QueryParser::parse(query);
        if query.is_empty() {
            assert!(ast.is_empty(), "Empty query should produce empty AST");
        } else {
            assert!(
                !ast.is_empty(),
                "Query '{}' should produce non-empty AST",
                query
            );
        }
    }
}

#[test]
fn test_advanced_view_rendering() {
    let app = FindMeApp::new();
    // Render the Advanced Search view
    let _view_element = findme_ui::views::advanced::view(&app.state);
}

#[test]
fn test_granular_extension_matrix_and_user_scenario() {
    let mut app = FindMeApp::new();
    let _ = app.update(Message::SwitchScreen(ScreenId::AdvancedSearch));

    // 1. Select Audio category
    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::SelectExtCategory(findme_ui::advanced::ExtensionCategory::Audio),
    ));
    assert_eq!(
        app.state.advanced_filters.active_ext_category,
        findme_ui::advanced::ExtensionCategory::Audio
    );

    // 2. Select specifically OGG extension (User Request Scenario!)
    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::ToggleExtension("ogg".to_string()),
    ));
    assert!(app.state.advanced_filters.is_extension_active("ogg"));
    assert_eq!(app.state.advanced_filters.synthesize_query(), "ext:ogg");

    // 3. Select FLAC as well
    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::ToggleExtension("flac".to_string()),
    ));
    assert!(app.state.advanced_filters.is_extension_active("flac"));
    let q = app.state.advanced_filters.synthesize_query();
    assert!(q.contains("ogg"));
    assert!(q.contains("flac"));
    assert_eq!(q, "ext:flac;ogg");

    // 4. View rendering with active ribbon
    {
        let _view_element = findme_ui::views::advanced::view(&app.state);
    }

    // 5. Select all in Audio category
    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::SelectAllInCategory(findme_ui::advanced::ExtensionCategory::Audio),
    ));
    for ext in findme_ui::advanced::ExtensionCategory::Audio.extensions() {
        assert!(app.state.advanced_filters.is_extension_active(ext));
    }

    // 6. Clear Audio category
    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::ClearCategory(findme_ui::advanced::ExtensionCategory::Audio),
    ));
    for ext in findme_ui::advanced::ExtensionCategory::Audio.extensions() {
        assert!(!app.state.advanced_filters.is_extension_active(ext));
    }

    // 7. Add custom extension tag
    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::UpdateCustomExtInput("parquet".to_string()),
    ));
    let _ = app.update(Message::AdvancedSearch(
        AdvancedSearchMessage::AddCustomExtFromInput,
    ));
    assert!(app.state.advanced_filters.is_extension_active("parquet"));
    assert_eq!(app.state.advanced_filters.synthesize_query(), "ext:parquet");

    // 8. Submit search with custom format
    let _ = app.update(Message::AdvancedSearch(AdvancedSearchMessage::SubmitSearch));
    assert_eq!(app.state.active_screen, ScreenId::Search);
    assert_eq!(app.state.query, "ext:parquet");
}
