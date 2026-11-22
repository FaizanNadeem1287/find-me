use findme_semantic::{
    generate_contextual_snippet, parse_location_metadata, parse_search_query,
    reciprocal_rank_fusion, Bm25Index, Bm25Params, ChunkStore, DocumentChunk, DocumentMetadata,
    ExtractedDocument, HnswVectorIndex, HybridSearchEngine, MatchKind, SearchMode, StoredChunk,
};
use std::path::PathBuf;

#[test]
fn test_query_prefix_parsing() {
    let q1 = parse_search_query("semantic: faizan nadeem resume", SearchMode::Hybrid);
    assert_eq!(q1.mode, SearchMode::Semantic);
    assert_eq!(q1.cleaned_query, "faizan nadeem resume");

    let q2 = parse_search_query("sem:deep learning", SearchMode::Everything);
    assert_eq!(q2.mode, SearchMode::Semantic);
    assert_eq!(q2.cleaned_query, "deep learning");

    let q3 = parse_search_query("content:fn main()", SearchMode::Semantic);
    assert_eq!(q3.mode, SearchMode::Content);
    assert_eq!(q3.cleaned_query, "fn main()");

    let q4 = parse_search_query("text:database_secret_key", SearchMode::Hybrid);
    assert_eq!(q4.mode, SearchMode::Content);
    assert_eq!(q4.cleaned_query, "database_secret_key");

    let q5 = parse_search_query("hybrid:rust vector search", SearchMode::Everything);
    assert_eq!(q5.mode, SearchMode::Hybrid);
    assert_eq!(q5.cleaned_query, "rust vector search");

    let q6 = parse_search_query("plain query without prefix", SearchMode::Hybrid);
    assert_eq!(q6.mode, SearchMode::Hybrid);
    assert_eq!(q6.cleaned_query, "plain query without prefix");
}

#[test]
fn test_bm25_lexical_ranking() {
    let chunk1 = StoredChunk {
        chunk_id: 1,
        path: PathBuf::from("/docs/rust.txt"),
        chunk_index: 0,
        location: "Lines 1-10".to_string(),
        snippet_text: "Rust is a high performance systems programming language with memory safety"
            .to_string(),
        vector_id: 1,
        mtime: 100,
        size_bytes: 500,
    };

    let chunk2 = StoredChunk {
        chunk_id: 2,
        path: PathBuf::from("/docs/python.txt"),
        chunk_index: 0,
        location: "Lines 1-10".to_string(),
        snippet_text:
            "Python is an interpreted scripting language commonly used in machine learning"
                .to_string(),
        vector_id: 2,
        mtime: 200,
        size_bytes: 400,
    };

    let chunks = vec![chunk1, chunk2];
    let bm25 = Bm25Index::build(&chunks, Bm25Params::default());

    let results_rust = bm25.search("rust systems", 10);
    assert!(!results_rust.is_empty());
    assert_eq!(results_rust[0].0, 0); // chunk index 0
    assert!(results_rust[0].1 > 0.0);

    let results_py = bm25.search("python scripting", 10);
    assert!(!results_py.is_empty());
    assert_eq!(results_py[0].0, 1); // chunk index 1
    assert!(results_py[0].1 > 0.0);

    let results_none = bm25.search("unmentioned_xyz_word", 10);
    assert!(results_none.is_empty());
}

#[test]
fn test_location_and_snippet_generation() {
    // 1. Page number extraction
    let (p1, t1) = parse_location_metadata("Page 5");
    assert_eq!(p1, Some(5));
    assert_eq!(t1, None);

    // 2. Lines extraction
    let (p2, t2) = parse_location_metadata("Lines 42-120");
    assert_eq!(p2, Some(42));
    assert_eq!(t2, None);

    // 3. Audio timestamp extraction
    let (p3, t3) = parse_location_metadata("[02:30 - 03:00]");
    assert_eq!(p3, None);
    assert_eq!(t3, Some(150.0)); // 2*60 + 30 = 150

    // 4. Contextual snippet extraction
    let text = "Welcome to the Find-Me search engine project. Muhammad Faizan Nadeem designed the high performance indexing architecture in pure Rust for Linux desktops.";
    let snippet = generate_contextual_snippet(text, "faizan nadeem", 80);
    assert!(snippet.contains("Faizan"));
    assert!(snippet.contains("Nadeem"));
}

#[test]
fn test_reciprocal_rank_fusion_logic() {
    // Document 100 is rank 1 in lexical and rank 1 in semantic
    let lex_ranks = vec![(100, 5.0), (101, 3.0), (102, 1.0)];
    let sem_ranks = vec![(100, 0.95), (102, 0.85), (103, 0.70)];

    let fused = reciprocal_rank_fusion(&lex_ranks, &sem_ranks, 60.0, 0.5, 0.5);
    assert!(!fused.is_empty());

    // Document 100 must be undisputed rank 1
    assert_eq!(fused[0].0, 100);
    // Score should be 0.5/(60+1) + 0.5/(60+1) = 1.0/61 = ~0.01639
    assert!((fused[0].1 - (1.0 / 61.0)).abs() < 1e-4);

    // Document 102 was rank 3 in lex and rank 2 in sem: 0.5/(60+3) + 0.5/(60+2)
    // Document 101 was rank 2 in lex: 0.5/(60+2)
    // Therefore 102 must score higher than 101
    let pos_102 = fused.iter().position(|r| r.0 == 102).unwrap();
    let pos_101 = fused.iter().position(|r| r.0 == 101).unwrap();
    assert!(pos_102 < pos_101);
}

#[test]
fn test_end_to_end_hybrid_search_engine() {
    let hnsw = HnswVectorIndex::new(16, 64, 32);
    let chunk_store = ChunkStore::new();

    // Document 1: Faizan CV
    let doc_cv = ExtractedDocument::new(
        PathBuf::from("/home/user/Faizan_Resume.pdf"),
        findme_semantic::DocumentType::Pdf,
        vec![
            DocumentChunk::new(
                0,
                "Page 1",
                "Muhammad Faizan Nadeem | Lead AI Engineer | Rust Systems Programming, Machine Learning, RAG and Vector Search",
            ),
            DocumentChunk::new(
                1,
                "Page 2",
                "Experience: Implemented HNSW vector indexing and distributed database systems in Linux unprivileged space",
            ),
        ],
        DocumentMetadata::default(),
    );

    // Document 2: Financial report
    let doc_finance = ExtractedDocument::new(
        PathBuf::from("/home/user/Q4_Financials.xlsx"),
        findme_semantic::DocumentType::Spreadsheet,
        vec![DocumentChunk::new(
            0,
            "Sheet 1",
            "Quarterly balance sheet: Operating revenue $4.2M, total net profit $1.1M, expenses breakdown",
        )],
        DocumentMetadata::default(),
    );

    // Document 3: Audio meeting note
    let doc_audio = ExtractedDocument::new(
        PathBuf::from("/home/user/Voice_Note_01.ogg"),
        findme_semantic::DocumentType::Audio,
        vec![DocumentChunk::new(
            0,
            "[01:30 - 02:00]",
            "Discussion on audio speech-to-text transcription and whisper integration for desktop voice notes",
        )],
        DocumentMetadata::default(),
    );

    let mut engine = HybridSearchEngine::new(hnsw, chunk_store);
    engine.index_document(&doc_cv).unwrap();
    engine.index_document(&doc_finance).unwrap();
    engine.index_document(&doc_audio).unwrap();

    assert_eq!(engine.chunk_store().len(), 4);

    // 1. Semantic search for AI CV
    let sem_hits = engine.search("semantic: Faizan AI resume CV", 5);
    assert!(!sem_hits.is_empty());
    assert_eq!(
        sem_hits[0].path,
        std::path::Path::new("/home/user/Faizan_Resume.pdf")
    );
    assert_eq!(sem_hits[0].page_or_line, Some(1));
    assert!(sem_hits[0].best_snippet.contains("Faizan"));

    // 2. Exact content search for financial terms
    let content_hits = engine.search("content: quarterly revenue profit", 5);
    assert!(!content_hits.is_empty());
    assert_eq!(
        content_hits[0].path,
        std::path::Path::new("/home/user/Q4_Financials.xlsx")
    );
    assert_eq!(content_hits[0].match_kind, MatchKind::ExactContent);

    // 3. Audio transcript search with timestamp jump
    let audio_hits = engine.search("audio speech voice notes", 5);
    assert!(!audio_hits.is_empty());
    let audio_hit = audio_hits
        .iter()
        .find(|h| h.path == std::path::Path::new("/home/user/Voice_Note_01.ogg"))
        .expect("Audio hit should be present");
    assert_eq!(audio_hit.match_kind, MatchKind::AudioTranscript);
    assert_eq!(audio_hit.audio_timestamp, Some(90.0)); // 1*60 + 30 = 90.0

    // 4. File aggregation (no duplicate files returned)
    let aggregated = engine.search_aggregate_by_file("Rust", SearchMode::Hybrid, 10);
    let paths: Vec<&PathBuf> = aggregated.iter().map(|h| &h.path).collect();
    let mut deduped = paths.clone();
    deduped.dedup();
    assert_eq!(paths.len(), deduped.len());
}
