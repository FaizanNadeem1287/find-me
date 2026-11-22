use findme_semantic::embeddings::{TextEmbedder, VectorEmbedding, EMBEDDING_DIM};
use findme_semantic::storage::{ChunkStore, HnswVectorIndex, StoredChunk};
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

#[test]
fn test_vector_math_operations() {
    let v1 = VectorEmbedding::new(vec![1.0, 0.0, 0.0]);
    let v2 = VectorEmbedding::new(vec![0.0, 1.0, 0.0]);
    let v3 = VectorEmbedding::new(vec![1.0, 0.0, 0.0]);
    let v4 = VectorEmbedding::new(vec![-1.0, 0.0, 0.0]);

    // Unit norm after initialization
    assert!((v1.norm() - 1.0).abs() < 1e-5);
    assert!((v2.norm() - 1.0).abs() < 1e-5);

    // Orthogonal
    assert!(v1.cosine_similarity(&v2).abs() < 1e-5);

    // Identical
    assert!((v1.cosine_similarity(&v3) - 1.0).abs() < 1e-5);

    // Opposite
    assert!((v1.cosine_similarity(&v4) - (-1.0)).abs() < 1e-5);

    // Euclidean distance
    assert!((v1.euclidean_distance(&v3)).abs() < 1e-5);
    assert!((v1.euclidean_distance(&v4) - 2.0).abs() < 1e-5);
}

#[test]
fn test_text_embedder_semantic_discrimination() {
    let embedder = TextEmbedder::new();

    let doc_cv1 =
        embedder.embed_text("Muhammad Faizan Nadeem Senior AI Engineer CV Resume Machine Learning");
    let doc_cv2 = embedder
        .embed_text("Curriculum Vitae Faizan Artificial Intelligence Deep Learning Engineer");
    let doc_food = embedder.embed_text(
        "Delicious Italian pasta recipe with garlic, olive oil, and fresh tomato sauce",
    );

    assert_eq!(doc_cv1.len(), EMBEDDING_DIM);
    assert_eq!(doc_cv2.len(), EMBEDDING_DIM);
    assert_eq!(doc_food.len(), EMBEDDING_DIM);

    let sim_cvs = doc_cv1.cosine_similarity(&doc_cv2);
    let sim_unrelated = doc_cv1.cosine_similarity(&doc_food);

    // Related AI CV documents should have high similarity
    assert!(
        sim_cvs > 0.60,
        "Expected high similarity between related CV texts, got {sim_cvs}"
    );

    // Unrelated food recipe should have low similarity
    assert!(
        sim_unrelated < 0.30,
        "Expected low similarity between CV and pasta recipe, got {sim_unrelated}"
    );

    assert!(sim_cvs > sim_unrelated);
}

#[test]
fn test_hnsw_insertion_and_search() {
    let mut index = HnswVectorIndex::new(8, 32, 16);

    let embedder = TextEmbedder::new();
    let texts = [
        (1, "Rust systems programming with memory safety"),
        (2, "Linux kernel notify filesystem monitoring inotify"),
        (3, "Machine learning deep neural networks and PyTorch"),
        (4, "Italian pizza dough recipe with yeast and flour"),
        (5, "Desktop GUI architecture using Iced framework Elm"),
    ];

    for (id, text) in texts {
        let vec = embedder.embed_text(text);
        index.insert(id, vec);
    }

    assert_eq!(index.len(), 5);

    // Query for Machine Learning
    let query_ml = embedder.embed_text("deep learning AI neural network");
    let results = index.search(&query_ml, 3);

    assert!(!results.is_empty());
    // ID 3 (Machine learning) should be top result
    assert_eq!(results[0].0, 3);
    assert!(results[0].1 > 0.50);

    // Query for Linux inotify
    let query_linux = embedder.embed_text("linux notify filesystem event");
    let res_linux = index.search(&query_linux, 2);
    assert!(!res_linux.is_empty());
    assert_eq!(res_linux[0].0, 2);
}

#[test]
fn test_hnsw_disk_persistence_and_crc() {
    let file = NamedTempFile::new().expect("tempfile");
    let mut index = HnswVectorIndex::new(8, 32, 16);

    let embedder = TextEmbedder::new();
    for i in 1..=20 {
        let vec = embedder.embed_text(&format!("Document number {i} covering topic {i}"));
        index.insert(i, vec);
    }

    assert_eq!(index.len(), 20);

    // Save to disk
    index.save_to_file(file.path()).expect("save_to_file");

    // Load from disk
    let loaded = HnswVectorIndex::load_from_file(file.path()).expect("load_from_file");
    assert_eq!(loaded.len(), 20);

    let query = embedder.embed_text("Document number 5");
    let orig_res = index.search(&query, 3);
    let loaded_res = loaded.search(&query, 3);
    assert_eq!(orig_res, loaded_res);

    // Corrupt a byte in the file and assert CRC validation failure
    {
        let mut f = File::options()
            .read(true)
            .write(true)
            .open(file.path())
            .expect("open");
        f.seek(SeekFrom::Start(16)).expect("seek");
        f.write_all(&[0xFF]).expect("write");
    }

    let corrupt_err = HnswVectorIndex::load_from_file(file.path());
    assert!(corrupt_err.is_err(), "Expected CRC mismatch error");
}

#[test]
fn test_chunk_store_lifecycle_and_persistence() {
    let file = NamedTempFile::new().expect("tempfile");
    let mut store = ChunkStore::new();

    let chunk1 = StoredChunk {
        chunk_id: 1,
        path: PathBuf::from("/home/user/resume.pdf"),
        chunk_index: 0,
        location: "Page 1".to_string(),
        snippet_text: "Muhammad Faizan Nadeem | Senior AI Engineer".to_string(),
        vector_id: 101,
        mtime: 1700000000,
        size_bytes: 45000,
    };

    let chunk2 = StoredChunk {
        chunk_id: 2,
        path: PathBuf::from("/home/user/resume.pdf"),
        chunk_index: 1,
        location: "Page 2".to_string(),
        snippet_text: "Skills: Rust, Python, PyTorch, Docker, Linux".to_string(),
        vector_id: 102,
        mtime: 1700000000,
        size_bytes: 45000,
    };

    let chunk3 = StoredChunk {
        chunk_id: 3,
        path: PathBuf::from("/home/user/notes.txt"),
        chunk_index: 0,
        location: "Lines 1-10".to_string(),
        snippet_text: "Groceries: Milk, Bread, Coffee".to_string(),
        vector_id: 103,
        mtime: 1700000500,
        size_bytes: 500,
    };

    store.add_chunk(chunk1.clone());
    store.add_chunk(chunk2.clone());
    store.add_chunk(chunk3.clone());

    assert_eq!(store.len(), 3);

    // Vector ID lookup
    assert_eq!(store.get_by_vector_id(101), Some(&chunk1));
    assert_eq!(store.get_by_vector_id(103), Some(&chunk3));
    assert_eq!(store.get_by_vector_id(999), None);

    // Path lookup
    let resume_chunks = store.get_chunks_for_path(Path::new("/home/user/resume.pdf"));
    assert_eq!(resume_chunks.len(), 2);

    // Save and load
    store.save_to_file(file.path()).expect("save_to_file");
    let loaded = ChunkStore::load_from_file(file.path()).expect("load_from_file");
    assert_eq!(loaded.len(), 3);
    assert_eq!(loaded.get_by_vector_id(102), Some(&chunk2));

    // Remove path
    let mut store_mut = loaded;
    let removed = store_mut.remove_path(Path::new("/home/user/resume.pdf"));
    assert_eq!(removed, 2);
    assert_eq!(store_mut.len(), 1);
    assert_eq!(store_mut.get_by_vector_id(101), None);
    assert_eq!(store_mut.get_by_vector_id(103), Some(&chunk3));
}

#[test]
fn test_end_to_end_semantic_indexing_and_querying() {
    let embedder = TextEmbedder::new();
    let mut hnsw = HnswVectorIndex::new(8, 32, 16);
    let mut chunk_store = ChunkStore::new();

    let documents = [
        (
            "/home/user/Downloads/Proposal- AI Powered Realtor APP.pdf",
            "Page 1",
            "Proposal for AI Powered Real Estate valuation and property search application",
        ),
        (
            "/home/user/Downloads/ARTEX-main/Dockerfile",
            "Lines 1-10",
            "FROM rust:1.80-slim as builder RUN cargo build --release",
        ),
        (
            "/home/user/Documents/CV_Faizan_Nadeem.pdf",
            "Page 1",
            "Muhammad Faizan Nadeem - Senior Artificial Intelligence and Systems Engineer specializing in Rust and Machine Learning",
        ),
        (
            "/home/user/Downloads/WhatsApp Audio 2026-11-18.ogg",
            "[00:00 - 00:30]",
            "Audio recording discussing real estate project timeline and milestones",
        ),
    ];

    for (i, (path_str, loc, snippet)) in documents.iter().enumerate() {
        let vec_id = (i + 1) as u64;
        let vector = embedder.embed_text(snippet);
        hnsw.insert(vec_id, vector);

        chunk_store.add_chunk(StoredChunk {
            chunk_id: vec_id,
            path: PathBuf::from(path_str),
            chunk_index: 0,
            location: loc.to_string(),
            snippet_text: snippet.to_string(),
            vector_id: vec_id,
            mtime: 1700000000,
            size_bytes: 1024,
        });
    }

    assert_eq!(hnsw.len(), 4);
    assert_eq!(chunk_store.len(), 4);

    // Natural language query: "my CV where my name is Faizan and it is about AI"
    let query_vector = embedder.embed_text("my CV where my name is Faizan and it is about AI");
    let matches = hnsw.search(&query_vector, 2);

    assert!(!matches.is_empty());
    let best_match_id = matches[0].0;
    let best_chunk = chunk_store
        .get_by_vector_id(best_match_id)
        .expect("chunk found");

    // The best match should be the CV document!
    assert!(best_chunk.path.to_string_lossy().contains("CV_Faizan"));
    assert!(best_chunk.snippet_text.contains("Faizan Nadeem"));
    assert!(best_chunk.snippet_text.contains("Artificial Intelligence"));
}
