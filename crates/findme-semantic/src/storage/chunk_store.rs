//! Document chunk metadata storage and fast vector-to-snippet lookup

use crate::error::SemanticError;
use crc32fast::Hasher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const CHUNK_STORE_MAGIC: &[u8; 4] = b"FMCS";
const CHUNK_STORE_VERSION: u32 = 1;

/// A stored chunk record linking a vector ID to its source file and contextual text snippet
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredChunk {
    pub chunk_id: u64,
    pub path: PathBuf,
    pub chunk_index: usize,
    pub location: String,
    pub snippet_text: String,
    pub vector_id: u64,
    pub mtime: u64,
    pub size_bytes: u64,
}

/// In-memory chunk metadata repository with disk persistence
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChunkStore {
    chunks: Vec<StoredChunk>,
    #[serde(skip)]
    lookup_by_vector_id: HashMap<u64, usize>,
    #[serde(skip)]
    lookup_by_path: HashMap<PathBuf, Vec<usize>>,
}

impl ChunkStore {
    /// Creates a new empty `ChunkStore`
    pub fn new() -> Self {
        Self {
            chunks: Vec::new(),
            lookup_by_vector_id: HashMap::new(),
            lookup_by_path: HashMap::new(),
        }
    }

    /// Total number of stored chunks
    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    /// Whether store is empty
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    /// Read-only slice of all stored chunks
    pub fn all_chunks(&self) -> &[StoredChunk] {
        &self.chunks
    }

    /// Rebuilds fast in-memory hash lookups from current chunk list
    pub fn rebuild_lookups(&mut self) {
        self.lookup_by_vector_id.clear();
        self.lookup_by_path.clear();

        for (idx, chunk) in self.chunks.iter().enumerate() {
            self.lookup_by_vector_id.insert(chunk.vector_id, idx);
            self.lookup_by_path
                .entry(chunk.path.clone())
                .or_default()
                .push(idx);
        }
    }

    /// Adds a chunk to the store and updates indices
    pub fn add_chunk(&mut self, chunk: StoredChunk) {
        let idx = self.chunks.len();
        self.lookup_by_vector_id.insert(chunk.vector_id, idx);
        self.lookup_by_path
            .entry(chunk.path.clone())
            .or_default()
            .push(idx);
        self.chunks.push(chunk);
    }

    /// Retrieves a chunk given its vector index ID in O(1)
    pub fn get_by_vector_id(&self, vector_id: u64) -> Option<&StoredChunk> {
        let &idx = self.lookup_by_vector_id.get(&vector_id)?;
        self.chunks.get(idx)
    }

    /// Retrieves all chunks associated with a specific file path
    pub fn get_chunks_for_path(&self, path: &Path) -> Vec<&StoredChunk> {
        match self.lookup_by_path.get(path) {
            Some(indices) => indices.iter().filter_map(|&i| self.chunks.get(i)).collect(),
            None => Vec::new(),
        }
    }

    /// Removes all chunks for a path (used when a file is modified, deleted, or re-indexed)
    pub fn remove_path(&mut self, path: &Path) -> usize {
        let initial_len = self.chunks.len();
        self.chunks.retain(|c| c.path != path);
        let removed = initial_len - self.chunks.len();
        if removed > 0 {
            self.rebuild_lookups();
        }
        removed
    }

    /// Clears the chunk store
    pub fn clear(&mut self) {
        self.chunks.clear();
        self.lookup_by_vector_id.clear();
        self.lookup_by_path.clear();
    }

    /// Serializes chunk store to disk with CRC32 integrity check
    pub fn save_to_file(&self, path: &Path) -> Result<(), SemanticError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let payload = bincode::serialize(&self.chunks).map_err(|e| {
            SemanticError::Extraction(format!("Failed to serialize chunk store: {e}"))
        })?;

        let mut hasher = Hasher::new();
        hasher.update(&payload);
        let checksum = hasher.finalize();

        let mut file = File::create(path)?;
        file.write_all(CHUNK_STORE_MAGIC)?;
        file.write_all(&CHUNK_STORE_VERSION.to_le_bytes())?;
        file.write_all(&checksum.to_le_bytes())?;
        file.write_all(&payload)?;
        file.flush()?;

        Ok(())
    }

    /// Loads chunk store from disk and validates CRC32 integrity
    pub fn load_from_file(path: &Path) -> Result<Self, SemanticError> {
        let mut file = File::open(path)?;
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;
        if &magic != CHUNK_STORE_MAGIC {
            return Err(SemanticError::Extraction(
                "Invalid chunk store magic bytes".to_string(),
            ));
        }

        let mut ver_bytes = [0u8; 4];
        file.read_exact(&mut ver_bytes)?;
        let version = u32::from_le_bytes(ver_bytes);
        if version != CHUNK_STORE_VERSION {
            return Err(SemanticError::Extraction(format!(
                "Unsupported chunk store version: {version}"
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
                "Chunk store CRC32 checksum mismatch (corrupted file)".to_string(),
            ));
        }

        let chunks: Vec<StoredChunk> = bincode::deserialize(&payload).map_err(|e| {
            SemanticError::Extraction(format!("Failed to deserialize chunk store: {e}"))
        })?;

        let mut store = Self {
            chunks,
            lookup_by_vector_id: HashMap::new(),
            lookup_by_path: HashMap::new(),
        };
        store.rebuild_lookups();
        Ok(store)
    }
}
