//! Search engines: Lexical BM25, semantic dense vector search, and hybrid Reciprocal Rank Fusion

pub mod bm25;
pub mod hybrid;

pub use bm25::{tokenize, Bm25Index, Bm25Params};
pub use hybrid::{
    generate_contextual_snippet, parse_location_metadata, parse_search_query,
    reciprocal_rank_fusion, HybridSearchEngine, MatchKind, ParsedSearchQuery, SearchMode,
    SemanticHit,
};
