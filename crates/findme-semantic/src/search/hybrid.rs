//! Hybrid Search Engine blending lexical BM25 and dense vector similarity using Reciprocal Rank Fusion (RRF)

use crate::embeddings::TextEmbedder;
use crate::error::SemanticError;
use crate::models::{DocumentChunk, ExtractedDocument};
use crate::search::bm25::{tokenize, Bm25Index, Bm25Params};
use crate::storage::{ChunkStore, HnswVectorIndex, StoredChunk};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Search operation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum SearchMode {
    /// Filename and metadata search (handled by core engine)
    Everything,
    /// Exact and BM25 token content search across chunks
    Content,
    /// Dense vector cosine semantic search
    Semantic,
    /// Blended BM25 and vector Reciprocal Rank Fusion
    #[default]
    Hybrid,
}

/// The nature of how a result was matched
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MatchKind {
    ExactContent,
    SemanticMatch,
    HybridFusion,
    AudioTranscript,
    VisionMatch,
}

/// A parsed search query with extracted search mode and cleaned terms
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSearchQuery {
    pub mode: SearchMode,
    pub raw_query: String,
    pub cleaned_query: String,
}

/// Parses query prefixes such as `sem:`, `semantic:`, `content:`, `hybrid:`
pub fn parse_search_query(input: &str, default_mode: SearchMode) -> ParsedSearchQuery {
    let trimmed = input.trim();

    if let Some(rest) = trimmed.strip_prefix("semantic:") {
        ParsedSearchQuery {
            mode: SearchMode::Semantic,
            raw_query: input.to_string(),
            cleaned_query: rest.trim().to_string(),
        }
    } else if let Some(rest) = trimmed.strip_prefix("sem:") {
        ParsedSearchQuery {
            mode: SearchMode::Semantic,
            raw_query: input.to_string(),
            cleaned_query: rest.trim().to_string(),
        }
    } else if let Some(rest) = trimmed.strip_prefix("content:") {
        ParsedSearchQuery {
            mode: SearchMode::Content,
            raw_query: input.to_string(),
            cleaned_query: rest.trim().to_string(),
        }
    } else if let Some(rest) = trimmed.strip_prefix("text:") {
        ParsedSearchQuery {
            mode: SearchMode::Content,
            raw_query: input.to_string(),
            cleaned_query: rest.trim().to_string(),
        }
    } else if let Some(rest) = trimmed.strip_prefix("hybrid:") {
        ParsedSearchQuery {
            mode: SearchMode::Hybrid,
            raw_query: input.to_string(),
            cleaned_query: rest.trim().to_string(),
        }
    } else if let Some(rest) = trimmed.strip_prefix("everything:") {
        ParsedSearchQuery {
            mode: SearchMode::Everything,
            raw_query: input.to_string(),
            cleaned_query: rest.trim().to_string(),
        }
    } else {
        ParsedSearchQuery {
            mode: default_mode,
            raw_query: input.to_string(),
            cleaned_query: trimmed.to_string(),
        }
    }
}

/// A synthesized search result item with contextual snippet
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticHit {
    pub path: PathBuf,
    pub score: f32,
    pub match_kind: MatchKind,
    pub location: String,
    pub best_snippet: String,
    pub vector_id: u64,
    pub mtime: u64,
    pub size_bytes: u64,
    pub page_or_line: Option<usize>,
    pub audio_timestamp: Option<f64>,
}

/// Parses page number or audio timestamp from location strings
pub fn parse_location_metadata(location: &str) -> (Option<usize>, Option<f64>) {
    let mut page_or_line = None;
    let mut audio_timestamp = None;

    let loc_lower = location.to_ascii_lowercase();

    // Check for page number e.g. "Page 3"
    if let Some(idx) = loc_lower.find("page ") {
        let after = &location[idx + 5..];
        let num_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(p) = num_str.parse::<usize>() {
            page_or_line = Some(p);
        }
    } else if let Some(idx) = loc_lower.find("lines ") {
        // e.g. "Lines 1-50"
        let after = &location[idx + 6..];
        let num_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(l) = num_str.parse::<usize>() {
            page_or_line = Some(l);
        }
    }

    // Check for audio timestamp e.g. "[01:30 - 02:00]" or "01:30"
    if let Some(start_bracket) = location.find('[') {
        if let Some(end_bracket) = location[start_bracket..].find(']') {
            let slice = &location[start_bracket + 1..start_bracket + end_bracket];
            let time_part = slice.split('-').next().unwrap_or(slice).trim();
            if let Some(colon) = time_part.find(':') {
                let mins: f64 = time_part[..colon].parse().unwrap_or(0.0);
                let secs: f64 = time_part[colon + 1..].parse().unwrap_or(0.0);
                audio_timestamp = Some(mins * 60.0 + secs);
            }
        }
    }

    (page_or_line, audio_timestamp)
}

/// Extracts a clean, contextual ~160 character snippet centered on query match terms
pub fn generate_contextual_snippet(text: &str, query: &str, max_len: usize) -> String {
    let clean_text = text.replace('\n', " ").replace('\r', "");
    let clean_text = clean_text.split_whitespace().collect::<Vec<_>>().join(" ");

    if clean_text.is_empty() {
        return String::new();
    }

    let query_terms = tokenize(query);
    let mut best_pos = None;

    // Search for first occurrence of any query token
    let text_lower = clean_text.to_ascii_lowercase();
    for term in &query_terms {
        if let Some(pos) = text_lower.find(term) {
            match best_pos {
                None => best_pos = Some(pos),
                Some(current) if pos < current => best_pos = Some(pos),
                _ => {}
            }
        }
    }

    let half = max_len / 2;
    let (start_idx, end_idx) = match best_pos {
        Some(pos) => {
            let start = pos.saturating_sub(half);
            let end = (pos + half).min(clean_text.len());
            (start, end)
        }
        None => (0, max_len.min(clean_text.len())),
    };

    // Align to char boundaries
    let mut start_bound = start_idx;
    while start_bound > 0 && !clean_text.is_char_boundary(start_bound) {
        start_bound -= 1;
    }

    let mut end_bound = end_idx;
    while end_bound < clean_text.len() && !clean_text.is_char_boundary(end_bound) {
        end_bound += 1;
    }

    let snippet_slice = &clean_text[start_bound..end_bound];
    let mut result = String::with_capacity(snippet_slice.len() + 8);

    if start_bound > 0 {
        result.push_str("... ");
    }
    result.push_str(snippet_slice.trim());
    if end_bound < clean_text.len() {
        result.push_str(" ...");
    }

    result
}

/// Merges lexical ranks and semantic ranks using Cormack Reciprocal Rank Fusion
pub fn reciprocal_rank_fusion(
    lexical_ranks: &[(u64, f32)],
    semantic_ranks: &[(u64, f32)],
    k: f32,
    lexical_weight: f32,
    semantic_weight: f32,
) -> Vec<(u64, f32)> {
    let mut fused_scores: HashMap<u64, f32> = HashMap::new();

    for (rank, &(id, _)) in lexical_ranks.iter().enumerate() {
        let rank_1based = (rank + 1) as f32;
        let rrf = lexical_weight / (k + rank_1based);
        *fused_scores.entry(id).or_insert(0.0) += rrf;
    }

    for (rank, &(id, _)) in semantic_ranks.iter().enumerate() {
        let rank_1based = (rank + 1) as f32;
        let rrf = semantic_weight / (k + rank_1based);
        *fused_scores.entry(id).or_insert(0.0) += rrf;
    }

    let mut results: Vec<(u64, f32)> = fused_scores.into_iter().collect();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    results
}

/// Complete hybrid search engine coordinating text embedder, HNSW index, chunk store, and BM25
pub struct HybridSearchEngine {
    embedder: TextEmbedder,
    vector_index: HnswVectorIndex,
    chunk_store: ChunkStore,
    bm25_index: Bm25Index,
    next_vector_id: u64,
    rrf_k: f32,
    lexical_weight: f32,
    semantic_weight: f32,
}

impl HybridSearchEngine {
    /// Creates a new `HybridSearchEngine` wrapping vector index and chunk store
    pub fn new(vector_index: HnswVectorIndex, chunk_store: ChunkStore) -> Self {
        let bm25_index = Bm25Index::build(chunk_store.all_chunks(), Bm25Params::default());
        let next_vector_id = (vector_index.len() as u64) + 1;
        Self {
            embedder: TextEmbedder::new(),
            vector_index,
            chunk_store,
            bm25_index,
            next_vector_id,
            rrf_k: 60.0,
            lexical_weight: 0.5,
            semantic_weight: 0.5,
        }
    }

    /// Customizes RRF hyper-parameters (k constant, lexical weight, semantic weight)
    pub fn set_rrf_weights(&mut self, k: f32, lexical_weight: f32, semantic_weight: f32) {
        self.rrf_k = k.max(1.0);
        self.lexical_weight = lexical_weight.max(0.0);
        self.semantic_weight = semantic_weight.max(0.0);
    }

    /// Access to chunk store
    pub fn chunk_store(&self) -> &ChunkStore {
        &self.chunk_store
    }

    /// Mutable access to chunk store
    pub fn chunk_store_mut(&mut self) -> &mut ChunkStore {
        &mut self.chunk_store
    }

    /// Access to HNSW vector index
    pub fn vector_index(&self) -> &HnswVectorIndex {
        &self.vector_index
    }

    /// Mutable access to HNSW vector index
    pub fn vector_index_mut(&mut self) -> &mut HnswVectorIndex {
        &mut self.vector_index
    }

    /// Rebuilds the in-memory BM25 index from current chunks
    pub fn rebuild_bm25(&mut self) {
        self.bm25_index = Bm25Index::build(self.chunk_store.all_chunks(), Bm25Params::default());
    }

    /// Indexes an extracted document into HNSW and ChunkStore, returning chunk count
    pub fn index_document(&mut self, doc: &ExtractedDocument) -> Result<usize, SemanticError> {
        // First remove existing chunks for this path to prevent duplicates on re-index
        let _ = self.remove_document(&doc.path);

        let mut indexed_count = 0;
        let metadata = std::fs::metadata(&doc.path).ok();
        let size_bytes = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        let mtime = metadata
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        for chunk in &doc.chunks {
            let embedding = self.embedder.embed_text(&chunk.text);
            let vector_id = self.next_vector_id;
            self.next_vector_id += 1;
            self.vector_index.insert(vector_id, embedding);

            let stored_chunk = StoredChunk {
                chunk_id: vector_id,
                path: doc.path.clone(),
                chunk_index: chunk.chunk_index,
                location: chunk.location.clone(),
                snippet_text: chunk.text.clone(),
                vector_id,
                mtime,
                size_bytes,
            };

            self.chunk_store.add_chunk(stored_chunk);
            indexed_count += 1;
        }

        self.rebuild_bm25();
        Ok(indexed_count)
    }

    /// Indexes a single chunk with specific metadata
    pub fn index_chunk(
        &mut self,
        path: PathBuf,
        chunk: &DocumentChunk,
        mtime: u64,
        size_bytes: u64,
    ) -> u64 {
        let embedding = self.embedder.embed_text(&chunk.text);
        let vector_id = self.next_vector_id;
        self.next_vector_id += 1;
        self.vector_index.insert(vector_id, embedding);

        let stored_chunk = StoredChunk {
            chunk_id: vector_id,
            path,
            chunk_index: chunk.chunk_index,
            location: chunk.location.clone(),
            snippet_text: chunk.text.clone(),
            vector_id,
            mtime,
            size_bytes,
        };

        self.chunk_store.add_chunk(stored_chunk);
        vector_id
    }

    /// Removes all indexed chunks for a file path
    pub fn remove_document(&mut self, path: &Path) -> usize {
        let removed = self.chunk_store.remove_path(path);
        if removed > 0 {
            self.rebuild_bm25();
        }
        removed
    }

    /// Queries engine with automatic operator prefix parsing (e.g. `semantic:`, `content:`)
    pub fn search(&self, query_str: &str, top_k: usize) -> Vec<SemanticHit> {
        let parsed = parse_search_query(query_str, SearchMode::Hybrid);
        self.search_with_mode(&parsed.cleaned_query, parsed.mode, top_k)
    }

    /// Queries engine with an explicit `SearchMode`
    pub fn search_with_mode(
        &self,
        query: &str,
        mode: SearchMode,
        top_k: usize,
    ) -> Vec<SemanticHit> {
        if query.trim().is_empty() || self.chunk_store.is_empty() {
            return Vec::new();
        }

        match mode {
            SearchMode::Everything => {
                // Everything mode is delegated to core filename search
                Vec::new()
            }
            SearchMode::Content => self.search_content(query, top_k),
            SearchMode::Semantic => self.search_semantic(query, top_k),
            SearchMode::Hybrid => self.search_hybrid(query, top_k),
        }
    }

    /// Performs exact and BM25 token content search
    fn search_content(&self, query: &str, top_k: usize) -> Vec<SemanticHit> {
        let bm25_hits = self.bm25_index.search(query, top_k * 2);
        let mut results = Vec::with_capacity(bm25_hits.len());

        let all_chunks = self.chunk_store.all_chunks();

        for (chunk_idx, score) in bm25_hits {
            if let Some(chunk) = all_chunks.get(chunk_idx) {
                let (page_or_line, audio_timestamp) = parse_location_metadata(&chunk.location);
                let best_snippet = generate_contextual_snippet(&chunk.snippet_text, query, 160);

                let match_kind = if audio_timestamp.is_some() {
                    MatchKind::AudioTranscript
                } else {
                    MatchKind::ExactContent
                };

                results.push(SemanticHit {
                    path: chunk.path.clone(),
                    score,
                    match_kind,
                    location: chunk.location.clone(),
                    best_snippet,
                    vector_id: chunk.vector_id,
                    mtime: chunk.mtime,
                    size_bytes: chunk.size_bytes,
                    page_or_line,
                    audio_timestamp,
                });
            }
        }

        if results.len() > top_k {
            results.truncate(top_k);
        }
        results
    }

    /// Performs dense vector semantic search using HNSW
    fn search_semantic(&self, query: &str, top_k: usize) -> Vec<SemanticHit> {
        let query_vec = self.embedder.embed_text(query);
        let hnsw_results = self.vector_index.search(&query_vec, top_k * 4);

        let mut results = Vec::with_capacity(hnsw_results.len());

        for (vector_id, sim) in hnsw_results {
            // HnswVectorIndex::search already returns cosine similarity sorted descending
            let similarity = sim.max(0.0);

            if let Some(chunk) = self.chunk_store.get_by_vector_id(vector_id) {
                let (page_or_line, audio_timestamp) = parse_location_metadata(&chunk.location);
                let best_snippet = generate_contextual_snippet(&chunk.snippet_text, query, 160);

                let match_kind = if audio_timestamp.is_some() {
                    MatchKind::AudioTranscript
                } else {
                    MatchKind::SemanticMatch
                };

                results.push(SemanticHit {
                    path: chunk.path.clone(),
                    score: similarity,
                    match_kind,
                    location: chunk.location.clone(),
                    best_snippet,
                    vector_id,
                    mtime: chunk.mtime,
                    size_bytes: chunk.size_bytes,
                    page_or_line,
                    audio_timestamp,
                });
            }
        }

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if results.len() > top_k {
            results.truncate(top_k);
        }
        results
    }

    /// Blends BM25 lexical ranking and dense vector semantics using Reciprocal Rank Fusion (RRF)
    fn search_hybrid(&self, query: &str, top_k: usize) -> Vec<SemanticHit> {
        let fetch_limit = (top_k * 3).max(50);

        // 1. Lexical BM25 rank
        let all_chunks = self.chunk_store.all_chunks();
        let bm25_hits = self.bm25_index.search(query, fetch_limit);
        let lexical_ranks: Vec<(u64, f32)> = bm25_hits
            .into_iter()
            .filter_map(|(chunk_idx, score)| {
                all_chunks.get(chunk_idx).map(|c| (c.vector_id, score))
            })
            .collect();

        // 2. Semantic vector rank
        let query_vec = self.embedder.embed_text(query);
        let hnsw_hits = self.vector_index.search(&query_vec, fetch_limit);
        let semantic_ranks: Vec<(u64, f32)> = hnsw_hits
            .into_iter()
            .map(|(vid, sim)| (vid, sim.max(0.0)))
            .collect();

        // 3. Cormack Reciprocal Rank Fusion
        let fused = reciprocal_rank_fusion(
            &lexical_ranks,
            &semantic_ranks,
            self.rrf_k,
            self.lexical_weight,
            self.semantic_weight,
        );

        let mut results = Vec::with_capacity(fused.len());

        for (vector_id, fused_score) in fused {
            if let Some(chunk) = self.chunk_store.get_by_vector_id(vector_id) {
                let (page_or_line, audio_timestamp) = parse_location_metadata(&chunk.location);
                let best_snippet = generate_contextual_snippet(&chunk.snippet_text, query, 160);

                let match_kind = if audio_timestamp.is_some() {
                    MatchKind::AudioTranscript
                } else {
                    MatchKind::HybridFusion
                };

                results.push(SemanticHit {
                    path: chunk.path.clone(),
                    score: fused_score,
                    match_kind,
                    location: chunk.location.clone(),
                    best_snippet,
                    vector_id,
                    mtime: chunk.mtime,
                    size_bytes: chunk.size_bytes,
                    page_or_line,
                    audio_timestamp,
                });
            }
        }

        if results.len() > top_k {
            results.truncate(top_k);
        }
        results
    }

    /// Aggregates search hits at the document file level (returning top chunk per file)
    pub fn search_aggregate_by_file(
        &self,
        query_str: &str,
        mode: SearchMode,
        top_k: usize,
    ) -> Vec<SemanticHit> {
        let chunk_hits = self.search_with_mode(query_str, mode, top_k * 4);
        let mut seen_paths = std::collections::HashSet::new();
        let mut file_hits = Vec::new();

        for hit in chunk_hits {
            if seen_paths.insert(hit.path.clone()) {
                file_hits.push(hit);
                if file_hits.len() >= top_k {
                    break;
                }
            }
        }

        file_hits
    }
}
