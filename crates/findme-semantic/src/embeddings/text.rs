//! Offline local text embedding engine mapping text chunks to dense vectors

use super::{VectorEmbedding, EMBEDDING_DIM};
use std::collections::HashMap;

/// Pre-configured semantic text embedder for 384-dimensional vector projection
#[derive(Debug, Clone, Default)]
pub struct TextEmbedder {
    dim: usize,
}

impl TextEmbedder {
    /// Creates a new `TextEmbedder` with the standard 384 dimensions
    pub fn new() -> Self {
        Self { dim: EMBEDDING_DIM }
    }

    /// Creates a `TextEmbedder` with custom vector dimensionality
    pub fn with_dim(dim: usize) -> Self {
        Self { dim }
    }

    /// Embeds a text string into a normalized 384-dimensional semantic vector
    pub fn embed_text(&self, text: &str) -> VectorEmbedding {
        let mut vec = vec![0.0f32; self.dim];
        let tokens = tokenize_and_normalize(text);

        if tokens.is_empty() {
            return VectorEmbedding::new(vec);
        }

        // Compute token frequencies
        let mut tf = HashMap::new();
        for token in &tokens {
            *tf.entry(token.as_str()).or_insert(0usize) += 1;
        }

        // Project unigrams and semantic concepts into vector space
        for (token, count) in &tf {
            let weight = 1.0 + (*count as f32).ln();

            // 1. Activate dense semantic concept cluster if token matches known topic domain
            if let Some(concept) = get_semantic_concept(token) {
                self.hash_and_accumulate_dense(concept, weight * 4.0, &mut vec);
            }

            // 2. Lexical word identity projection
            self.hash_and_accumulate(token, weight, &mut vec);

            // 3. Subword character trigrams for typo-tolerance and morphological stems
            let chars: Vec<char> = token.chars().collect();
            if chars.len() >= 3 {
                for window in chars.windows(3) {
                    let trigram: String = window.iter().collect();
                    self.hash_and_accumulate(&trigram, weight * 0.25, &mut vec);
                }
            }
        }

        // 4. Bigram hashing to capture word order and phrases ("machine learning", "ai engineer")
        for window in tokens.windows(2) {
            let bigram = format!("{}_{}", window[0], window[1]);
            self.hash_and_accumulate(&bigram, 1.5, &mut vec);
        }

        VectorEmbedding::new(vec)
    }

    /// Hashes a token string into coordinate index and random sign projection
    #[inline]
    fn hash_and_accumulate(&self, token: &str, weight: f32, vec: &mut [f32]) {
        let h1 = hash_str(token, 0x9e3779b97f4a7c15);
        let h2 = hash_str(token, 0x517cc1b727220a95);

        let idx1 = (h1 as usize) % self.dim;
        let sign1 = if (h1 & 0x01) == 0 { 1.0f32 } else { -1.0f32 };
        vec[idx1] += sign1 * weight;

        let idx2 = (h2 as usize) % self.dim;
        let sign2 = if (h2 & 0x01) == 0 { 1.0f32 } else { -1.0f32 };
        vec[idx2] += sign2 * weight * 0.7;
    }

    /// Projects a semantic concept cluster across multiple contiguous coordinate dimensions
    #[inline]
    fn hash_and_accumulate_dense(&self, concept: &str, weight: f32, vec: &mut [f32]) {
        let base_hash = hash_str(concept, 0x123456789abcdef0);
        let num_dims = 16usize;
        let norm_weight = weight / (num_dims as f32).sqrt();

        for i in 0..num_dims {
            let h = base_hash.wrapping_add((i as u64).wrapping_mul(0x9e3779b97f4a7c15));
            let idx = (h as usize) % self.dim;
            vec[idx] += norm_weight;
        }
    }
}

/// Tokenizes text into lowercase alphanumeric words
fn tokenize_and_normalize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .filter_map(|word| {
            let trimmed = word.trim_matches(|c: char| !c.is_alphanumeric());
            if trimmed.len() >= 2 && !is_stopword(trimmed) {
                Some(trimmed.to_ascii_lowercase())
            } else {
                None
            }
        })
        .collect()
}

/// Fast FNV-1a hash with custom seed
#[inline]
fn hash_str(s: &str, seed: u64) -> u64 {
    let mut hash = seed;
    for &byte in s.as_bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Maps keywords and technical vocabulary into broad semantic concept clusters
fn get_semantic_concept(token: &str) -> Option<&'static str> {
    match token {
        "cv" | "resume" | "curriculum" | "vitae" | "portfolio" | "profile" => {
            Some("concept_resume")
        }
        "ai" | "ml" | "artificial" | "intelligence" | "machine" | "learning" | "deep"
        | "neural" | "network" | "networks" | "pytorch" | "tensorflow" | "model" | "models"
        | "rag" | "vector" | "vectors" | "llm" | "llms" | "embedding" | "embeddings" => {
            Some("concept_ai")
        }
        "engineer" | "engineering" | "developer" | "development" | "programmer" | "programming"
        | "software" | "systems" | "architect" | "lead" | "senior" => Some("concept_engineering"),
        "data" | "database" | "sql" | "sqlite" | "table" | "schema" | "record" | "storage"
        | "postgres" | "mysql" => Some("concept_data"),
        "audio" | "sound" | "voice" | "recording" | "recordings" | "speech" | "whisper"
        | "music" | "podcast" | "microphone" | "transcript" | "transcription" => {
            Some("concept_audio")
        }
        "image" | "images" | "photo" | "photos" | "picture" | "pictures" | "graphic"
        | "graphics" | "diagram" | "diagrams" | "chart" | "charts" | "figure" | "visual"
        | "vision" | "screenshot" => Some("concept_visual"),
        "realtor" | "estate" | "property" | "properties" | "valuation" | "realty" | "house"
        | "apartment" => Some("concept_real_estate"),
        "docker" | "dockerfile" | "container" | "containers" | "cloud" | "server" | "api"
        | "backend" | "frontend" | "web" | "http" => Some("concept_web_cloud"),
        "linux" | "kernel" | "ubuntu" | "debian" | "desktop" | "os" | "system" | "filesystem"
        | "inotify" | "posix" => Some("concept_linux_os"),
        "finance" | "financial" | "budget" | "invoice" | "quotation" | "price" | "cost"
        | "revenue" | "salary" | "proposal" => Some("concept_finance"),
        _ => None,
    }
}

/// Common English stopwords filter
fn is_stopword(word: &str) -> bool {
    matches!(
        word.to_ascii_lowercase().as_str(),
        "the"
            | "a"
            | "an"
            | "and"
            | "or"
            | "but"
            | "is"
            | "are"
            | "was"
            | "were"
            | "in"
            | "on"
            | "at"
            | "to"
            | "for"
            | "with"
            | "by"
            | "about"
            | "of"
            | "it"
            | "as"
            | "be"
            | "this"
            | "that"
    )
}
