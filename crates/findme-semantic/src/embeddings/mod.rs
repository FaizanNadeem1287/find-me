//! Dense vector representation and mathematical vector operations

use serde::{Deserialize, Serialize};

/// Standard dimensionality of text semantic embeddings (all-MiniLM-L6-v2 standard)
pub const EMBEDDING_DIM: usize = 384;

/// A dense, normalized floating-point vector embedding
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorEmbedding {
    pub values: Vec<f32>,
}

impl VectorEmbedding {
    /// Creates a new vector embedding with given values
    pub fn new(values: Vec<f32>) -> Self {
        let mut emb = Self { values };
        emb.normalize();
        emb
    }

    /// Creates an empty zero-vector of specified dimension
    pub fn zeros(dim: usize) -> Self {
        Self {
            values: vec![0.0; dim],
        }
    }

    /// Computes dot product with another vector
    #[inline]
    pub fn dot(&self, other: &Self) -> f32 {
        let min_len = self.values.len().min(other.values.len());
        let mut sum = 0.0f32;
        for i in 0..min_len {
            sum += self.values[i] * other.values[i];
        }
        sum
    }

    /// Computes cosine similarity between two vectors (-1.0 to 1.0)
    #[inline]
    pub fn cosine_similarity(&self, other: &Self) -> f32 {
        let dot = self.dot(other);
        let norm_a = self.norm();
        let norm_b = other.norm();
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
        }
    }

    /// Computes Euclidean distance (L2 norm) between two vectors
    #[inline]
    pub fn euclidean_distance(&self, other: &Self) -> f32 {
        let min_len = self.values.len().min(other.values.len());
        let mut sum_sq = 0.0f32;
        for i in 0..min_len {
            let diff = self.values[i] - other.values[i];
            sum_sq += diff * diff;
        }
        sum_sq.sqrt()
    }

    /// Computes L2 norm (magnitude) of the vector
    #[inline]
    pub fn norm(&self) -> f32 {
        let mut sum_sq = 0.0f32;
        for &v in &self.values {
            sum_sq += v * v;
        }
        sum_sq.sqrt()
    }

    /// In-place L2 normalization to unit length
    pub fn normalize(&mut self) {
        let n = self.norm();
        if n > 0.0 {
            let inv = 1.0 / n;
            for v in &mut self.values {
                *v *= inv;
            }
        }
    }

    /// Returns vector dimension
    #[inline]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns whether vector is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

pub mod text;
pub use text::TextEmbedder;
