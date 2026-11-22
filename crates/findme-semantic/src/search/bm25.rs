//! Lexical BM25 (Best Matching 25) ranking for chunk text

use crate::storage::StoredChunk;
use std::collections::HashMap;

/// Configurable BM25 scoring parameters
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bm25Params {
    /// Term frequency saturation parameter (standard default: 1.2)
    pub k1: f32,
    /// Document length normalization parameter (standard default: 0.75)
    pub b: f32,
}

impl Default for Bm25Params {
    fn default() -> Self {
        Self { k1: 1.2, b: 0.75 }
    }
}

/// Tokenizer for lexical search (lowercase alphanumeric tokens)
pub fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|t| !t.is_empty())
        .map(|t| t.to_ascii_lowercase())
        .collect()
}

/// In-memory BM25 index built over stored chunks
#[derive(Debug, Clone)]
pub struct Bm25Index {
    params: Bm25Params,
    doc_count: usize,
    avg_doc_len: f32,
    doc_lens: Vec<usize>,
    doc_freqs: HashMap<String, usize>,
    inverted_index: HashMap<String, Vec<(usize, usize)>>,
}

impl Bm25Index {
    /// Builds a BM25 index from stored chunks
    pub fn build(chunks: &[StoredChunk], params: Bm25Params) -> Self {
        let doc_count = chunks.len();
        if doc_count == 0 {
            return Self {
                params,
                doc_count: 0,
                avg_doc_len: 0.0,
                doc_lens: Vec::new(),
                doc_freqs: HashMap::new(),
                inverted_index: HashMap::new(),
            };
        }

        let mut doc_lens = Vec::with_capacity(doc_count);
        let mut doc_freqs = HashMap::new();
        let mut inverted_index = HashMap::new();
        let mut total_len = 0usize;

        for (doc_idx, chunk) in chunks.iter().enumerate() {
            let tokens = tokenize(&chunk.snippet_text);
            let len = tokens.len();
            doc_lens.push(len);
            total_len += len;

            let mut tf_map: HashMap<String, usize> = HashMap::new();
            for token in tokens {
                *tf_map.entry(token).or_insert(0) += 1;
            }

            for (token, count) in tf_map {
                *doc_freqs.entry(token.clone()).or_insert(0) += 1;
                inverted_index
                    .entry(token)
                    .or_insert_with(Vec::new)
                    .push((doc_idx, count));
            }
        }

        let avg_doc_len = if doc_count > 0 {
            total_len as f32 / doc_count as f32
        } else {
            0.0
        };

        Self {
            params,
            doc_count,
            avg_doc_len,
            doc_lens,
            doc_freqs,
            inverted_index,
        }
    }

    /// Queries the BM25 index and returns ranked (chunk_index, bm25_score)
    pub fn search(&self, query: &str, top_k: usize) -> Vec<(usize, f32)> {
        let query_tokens = tokenize(query);
        if query_tokens.is_empty() || self.doc_count == 0 {
            return Vec::new();
        }

        let mut scores: HashMap<usize, f32> = HashMap::new();
        let k1 = self.params.k1;
        let b = self.params.b;
        let avgdl = self.avg_doc_len.max(1.0);
        let n = self.doc_count as f32;

        for token in &query_tokens {
            if let Some(postings) = self.inverted_index.get(token) {
                let df = *self.doc_freqs.get(token).unwrap_or(&1) as f32;
                // Standard Robertson-Spärck Jones IDF with smoothing
                let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln().max(0.01);

                for &(doc_idx, tf) in postings {
                    let doc_len = self.doc_lens.get(doc_idx).copied().unwrap_or(1) as f32;
                    let tf_f = tf as f32;
                    let numerator = tf_f * (k1 + 1.0);
                    let denominator = tf_f + k1 * (1.0 - b + b * (doc_len / avgdl));
                    let term_score = idf * (numerator / denominator);

                    *scores.entry(doc_idx).or_insert(0.0) += term_score;
                }
            }
        }

        let mut ranked: Vec<(usize, f32)> = scores.into_iter().collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        if ranked.len() > top_k {
            ranked.truncate(top_k);
        }
        ranked
    }
}
