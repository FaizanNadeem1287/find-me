//! Hierarchical Navigable Small World (HNSW) vector similarity index
//!
//! Provides sub-5ms approximate nearest neighbor search over dense vector embeddings
//! with disk persistence and CRC32 integrity validation.

use crate::embeddings::VectorEmbedding;
use crate::error::SemanticError;
use crc32fast::Hasher;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

const HNSW_MAGIC: &[u8; 4] = b"FMHV";
const HNSW_VERSION: u32 = 1;

/// A node in the HNSW multi-layer graph
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HnswNode {
    pub id: u64,
    pub vector: VectorEmbedding,
    pub level: usize,
    /// neighbors[level] is list of internal node indices in that level
    pub neighbors: Vec<Vec<usize>>,
}

/// Candidate pair (distance, node_index) for priority queues
#[derive(Debug, Clone, Copy, PartialEq)]
struct Candidate {
    dist: f32,
    idx: usize,
}

impl Eq for Candidate {}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering so BinaryHeap acts as a min-heap
        other
            .dist
            .partial_cmp(&self.dist)
            .unwrap_or(Ordering::Equal)
    }
}

/// Max-heap candidate pair
#[derive(Debug, Clone, Copy, PartialEq)]
struct MaxCandidate {
    dist: f32,
    idx: usize,
}

impl Eq for MaxCandidate {}

impl PartialOrd for MaxCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MaxCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.dist
            .partial_cmp(&other.dist)
            .unwrap_or(Ordering::Equal)
    }
}

/// High-performance Hierarchical Navigable Small World vector index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswVectorIndex {
    m: usize,
    m0: usize,
    ef_construction: usize,
    ef_search: usize,
    ml: f32,
    nodes: Vec<HnswNode>,
    entry_point: Option<usize>,
    max_level: usize,
}

impl Default for HnswVectorIndex {
    fn default() -> Self {
        Self::new(16, 64, 32)
    }
}

impl HnswVectorIndex {
    /// Creates a new HNSW vector index with specified hyperparameters
    pub fn new(m: usize, ef_construction: usize, ef_search: usize) -> Self {
        let m = m.max(4);
        let m0 = m * 2;
        let ml = 1.0 / (m as f32).ln().max(1.0);
        Self {
            m,
            m0,
            ef_construction,
            ef_search,
            ml,
            nodes: Vec::new(),
            entry_point: None,
            max_level: 0,
        }
    }

    /// Returns the total number of indexed vectors
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns whether the index is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Computes cosine distance (0.0 = identical, 2.0 = opposite)
    #[inline]
    fn distance(&self, a: &VectorEmbedding, b: &VectorEmbedding) -> f32 {
        1.0 - a.cosine_similarity(b)
    }

    /// Samples a random level for a new node using pseudo-random exponential distribution
    fn sample_level(&self, seed: u64) -> usize {
        let mut h = seed;
        h = h.wrapping_mul(0x517cc1b727220a95).wrapping_add(1);
        let uniform = ((h as f64) / (u64::MAX as f64)).clamp(0.000001, 0.999999);
        let r = (-uniform.ln() * (self.ml as f64)).floor() as usize;
        r.min(16)
    }

    /// Inserts a vector into the HNSW index
    pub fn insert(&mut self, id: u64, vector: VectorEmbedding) {
        let new_idx = self.nodes.len();
        let target_level = self.sample_level(id.wrapping_add(new_idx as u64));

        let mut new_node = HnswNode {
            id,
            vector,
            level: target_level,
            neighbors: vec![Vec::new(); target_level + 1],
        };

        if self.nodes.is_empty() || self.entry_point.is_none() {
            self.nodes.push(new_node);
            self.entry_point = Some(new_idx);
            self.max_level = target_level;
            return;
        }

        let mut curr_entry = match self.entry_point {
            Some(ep) => ep,
            None => {
                self.nodes.push(new_node);
                self.entry_point = Some(new_idx);
                self.max_level = target_level;
                return;
            }
        };

        let curr_max_level = self.max_level;

        // 1. Greedily zoom down from top layer to insertion level
        if target_level < curr_max_level {
            for level in (target_level + 1..=curr_max_level).rev() {
                let mut changed = true;
                while changed {
                    changed = false;
                    let curr_dist = self.distance(&new_node.vector, &self.nodes[curr_entry].vector);

                    if let Some(neighbors) = self.nodes[curr_entry].neighbors.get(level) {
                        for &neighbor in neighbors {
                            if neighbor < self.nodes.len() {
                                let dist =
                                    self.distance(&new_node.vector, &self.nodes[neighbor].vector);
                                if dist < curr_dist {
                                    curr_entry = neighbor;
                                    changed = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. Insert and connect in levels from min(target_level, curr_max_level) down to 0
        let connect_level = target_level.min(curr_max_level);
        let mut ep_set = vec![curr_entry];

        for level in (0..=connect_level).rev() {
            let candidates =
                self.search_level(&new_node.vector, &ep_set, self.ef_construction, level);
            let m_max = if level == 0 { self.m0 } else { self.m };
            let neighbors = self.select_neighbors(&new_node.vector, &candidates, m_max);

            new_node.neighbors[level] = neighbors.clone();

            // Connect back from neighbors
            for &nbr in &neighbors {
                if nbr < self.nodes.len() && level < self.nodes[nbr].neighbors.len() {
                    self.nodes[nbr].neighbors[level].push(new_idx);
                    if self.nodes[nbr].neighbors[level].len() > m_max {
                        let nbr_candidates: Vec<Candidate> = self.nodes[nbr].neighbors[level]
                            .iter()
                            .map(|&idx| {
                                let dist = if idx == new_idx {
                                    self.distance(&self.nodes[nbr].vector, &new_node.vector)
                                } else {
                                    self.distance(&self.nodes[nbr].vector, &self.nodes[idx].vector)
                                };
                                Candidate { dist, idx }
                            })
                            .collect();

                        self.nodes[nbr].neighbors[level] =
                            self.select_neighbors(&self.nodes[nbr].vector, &nbr_candidates, m_max);
                    }
                }
            }

            ep_set = neighbors;
            if ep_set.is_empty() {
                ep_set.push(curr_entry);
            }
        }

        // Update entry point if new node has strictly higher level
        if target_level > self.max_level {
            self.max_level = target_level;
            self.entry_point = Some(new_idx);
        }

        self.nodes.push(new_node);
    }

    /// Performs beam search within a single graph layer
    fn search_level(
        &self,
        query: &VectorEmbedding,
        enter_points: &[usize],
        ef: usize,
        level: usize,
    ) -> Vec<Candidate> {
        let mut visited = HashSet::new();
        let mut candidates = BinaryHeap::new();
        let mut best = BinaryHeap::new();

        for &ep in enter_points {
            if ep < self.nodes.len() && visited.insert(ep) {
                let dist = self.distance(query, &self.nodes[ep].vector);
                candidates.push(Candidate { dist, idx: ep });
                best.push(MaxCandidate { dist, idx: ep });
            }
        }

        while let Some(current) = candidates.pop() {
            if let Some(furthest) = best.peek() {
                if current.dist > furthest.dist {
                    break;
                }
            }

            if let Some(neighbors) = self.nodes[current.idx].neighbors.get(level) {
                for &nbr in neighbors {
                    if nbr < self.nodes.len() && visited.insert(nbr) {
                        let dist = self.distance(query, &self.nodes[nbr].vector);
                        let furthest_dist = best.peek().map_or(f32::INFINITY, |c| c.dist);

                        if dist < furthest_dist || best.len() < ef {
                            candidates.push(Candidate { dist, idx: nbr });
                            best.push(MaxCandidate { dist, idx: nbr });

                            if best.len() > ef {
                                best.pop();
                            }
                        }
                    }
                }
            }
        }

        best.into_iter()
            .map(|c| Candidate {
                dist: c.dist,
                idx: c.idx,
            })
            .collect()
    }

    /// Selects the closest `m` neighbors from candidate list
    fn select_neighbors(
        &self,
        _query: &VectorEmbedding,
        candidates: &[Candidate],
        m: usize,
    ) -> Vec<usize> {
        let mut sorted = candidates.to_vec();
        sorted.sort_by(|a, b| a.dist.partial_cmp(&b.dist).unwrap_or(Ordering::Equal));
        sorted.into_iter().take(m).map(|c| c.idx).collect()
    }

    /// Searches for the `top_k` closest nearest neighbors to the query vector.
    ///
    /// Returns vector of `(id, cosine_similarity)` sorted by similarity descending.
    pub fn search(&self, query: &VectorEmbedding, top_k: usize) -> Vec<(u64, f32)> {
        if self.nodes.is_empty() || top_k == 0 {
            return Vec::new();
        }

        let mut curr_entry = match self.entry_point {
            Some(ep) if ep < self.nodes.len() => ep,
            _ => 0,
        };

        // 1. Zoom down from top level to Layer 0
        for level in (1..=self.max_level).rev() {
            let mut changed = true;
            while changed {
                changed = false;
                let curr_dist = self.distance(query, &self.nodes[curr_entry].vector);

                if let Some(neighbors) = self.nodes[curr_entry].neighbors.get(level) {
                    for &nbr in neighbors {
                        if nbr < self.nodes.len() {
                            let dist = self.distance(query, &self.nodes[nbr].vector);
                            if dist < curr_dist {
                                curr_entry = nbr;
                                changed = true;
                                break;
                            }
                        }
                    }
                }
            }
        }

        // 2. Beam search on Layer 0 with ef_search
        let ef = self.ef_search.max(top_k);
        let candidates = self.search_level(query, &[curr_entry], ef, 0);

        let mut results: Vec<(u64, f32)> = candidates
            .into_iter()
            .filter(|c| c.idx < self.nodes.len())
            .map(|c| {
                let id = self.nodes[c.idx].id;
                let sim = 1.0 - c.dist;
                (id, sim.clamp(-1.0, 1.0))
            })
            .collect();

        // Sort by similarity descending
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
        results.truncate(top_k);
        results
    }

    /// Serializes the HNSW vector index to disk with CRC32 integrity check
    pub fn save_to_file(&self, path: &Path) -> Result<(), SemanticError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let payload = bincode::serialize(self).map_err(|e| {
            SemanticError::Extraction(format!("Failed to serialize HNSW index: {e}"))
        })?;

        let mut hasher = Hasher::new();
        hasher.update(&payload);
        let checksum = hasher.finalize();

        let mut file = File::create(path)?;
        file.write_all(HNSW_MAGIC)?;
        file.write_all(&HNSW_VERSION.to_le_bytes())?;
        file.write_all(&checksum.to_le_bytes())?;
        file.write_all(&payload)?;
        file.flush()?;

        Ok(())
    }

    /// Loads the HNSW vector index from disk and validates CRC32 integrity
    pub fn load_from_file(path: &Path) -> Result<Self, SemanticError> {
        let mut file = File::open(path)?;
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;
        if &magic != HNSW_MAGIC {
            return Err(SemanticError::Extraction(
                "Invalid HNSW file magic".to_string(),
            ));
        }

        let mut ver_bytes = [0u8; 4];
        file.read_exact(&mut ver_bytes)?;
        let version = u32::from_le_bytes(ver_bytes);
        if version != HNSW_VERSION {
            return Err(SemanticError::Extraction(format!(
                "Unsupported HNSW version: {version}"
            )));
        }

        let mut crc_bytes = [0u8; 4];
        file.read_exact(&mut crc_bytes)?;
        let expected_crc = u32::from_le_bytes(crc_bytes);

        let mut payload = Vec::new();
        file.read_to_end(&mut payload)?;

        let mut hasher = Hasher::new();
        hasher.update(&payload);
        let actual_crc = hasher.finalize();

        if actual_crc != expected_crc {
            return Err(SemanticError::Extraction(
                "HNSW index CRC32 checksum mismatch (corrupted file)".to_string(),
            ));
        }

        bincode::deserialize(&payload).map_err(|e| {
            SemanticError::Extraction(format!("Failed to deserialize HNSW index: {e}"))
        })
    }
}
