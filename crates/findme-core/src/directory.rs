//! Deduplicated directory tree and path resolution table

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A directory entry in the deduplicated hierarchy
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectoryEntry {
    /// Index of parent directory entry (ROOT_ID points to itself)
    pub parent_id: u32,
    /// Absolute or relative segment path
    pub path: PathBuf,
}

/// The Root directory ID constant
pub const ROOT_DIR_ID: u32 = 0;

/// Fast deduplicated directory table
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DirectoryTable {
    entries: Vec<DirectoryEntry>,
}

impl DirectoryTable {
    pub fn new() -> Self {
        let mut table = Self {
            entries: Vec::with_capacity(1024),
        };
        // Insert dummy root entry at index 0
        table.entries.push(DirectoryEntry {
            parent_id: ROOT_DIR_ID,
            path: PathBuf::from("/"),
        });
        table
    }

    /// Inserts or retrieves a directory ID for the given path
    pub fn get_or_insert(&mut self, path: &Path) -> u32 {
        if path == Path::new("/") {
            return ROOT_DIR_ID;
        }

        // Check if path already exists
        if let Some((id, _)) = self
            .entries
            .iter()
            .enumerate()
            .find(|(_, e)| e.path == path)
        {
            return id as u32;
        }

        // Resolve parent
        let parent_id = if let Some(parent) = path.parent() {
            self.get_or_insert(parent)
        } else {
            ROOT_DIR_ID
        };

        let id = self.entries.len() as u32;
        self.entries.push(DirectoryEntry {
            parent_id,
            path: path.to_path_buf(),
        });
        id
    }

    /// Resolves a full directory path by ID
    pub fn get_path(&self, id: u32) -> Option<&Path> {
        self.entries.get(id as usize).map(|e| e.path.as_path())
    }

    /// Total number of unique directories indexed
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the directory table contains only root
    pub fn is_empty(&self) -> bool {
        self.entries.len() <= 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directory_deduplication() {
        let mut table = DirectoryTable::new();
        assert_eq!(table.len(), 1);

        let id1 = table.get_or_insert(Path::new("/home/user/docs"));
        let id2 = table.get_or_insert(Path::new("/home/user/docs"));
        assert_eq!(id1, id2, "Same directory must return identical ID");

        let path = table.get_path(id1);
        assert_eq!(path, Some(Path::new("/home/user/docs")));
    }
}
