//! Find-Me Semantic: Deep content extraction, offline vector embeddings, and hybrid retrieval.

pub mod embeddings;
pub mod error;
pub mod extractors;
pub mod models;
pub mod search;
pub mod storage;
pub mod worker;

pub use embeddings::{TextEmbedder, VectorEmbedding, EMBEDDING_DIM};
pub use error::SemanticError;
pub use extractors::extract_content;
pub use models::{DocumentChunk, DocumentMetadata, DocumentType, ExtractedDocument};
pub use search::{
    generate_contextual_snippet, parse_location_metadata, parse_search_query,
    reciprocal_rank_fusion, Bm25Index, Bm25Params, HybridSearchEngine, MatchKind,
    ParsedSearchQuery, SearchMode, SemanticHit,
};
pub use storage::{ChunkStore, HnswVectorIndex, StoredChunk};
pub use worker::{
    BackgroundIndexer, FileFingerprint, FileFingerprintCache, IndexerProgress, ThrottleMode,
};

use findme_core::arena::FileArena;
use findme_core::record::FileRecord;

/// Extracts content from a `FileRecord` using the provided arena to resolve its filesystem path.
pub fn extract_record_content(
    arena: &FileArena,
    record: &FileRecord,
) -> Result<ExtractedDocument, SemanticError> {
    let path = arena.reconstruct_path(record).ok_or_else(|| {
        SemanticError::Extraction("Failed to reconstruct record path".to_string())
    })?;

    extract_content(&path)
}
