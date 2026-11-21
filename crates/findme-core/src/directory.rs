//! Deduplicated directory tree and path resolution table

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A directory entry in the deduplicated hierarchy
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectoryEntry {
    /// Index of parent directory entry (ROOT_DIR_ID points to itself)
    pub parent_id: u32,
    /// Directory segment name (empty for root "/")
    pub name: String,
}

/// The Root directory ID constant
pub const ROOT_DIR_ID: u32 = 0;

/// Fast deduplicated directory table with $O(1)$ lookups and parent pointer resolution
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirectoryTable {
    entries: Vec<DirectoryEntry>,
    #[serde(skip)]
    lookup: HashMap<PathBuf, u32>,
}

impl Default for DirectoryTable {
    fn default() -> Self {
        Self::new()
    }
}

impl DirectoryTable {
    /// Creates a new directory table initialized with the root directory (`/`)
    pub fn new() -> Self {
        Self::with_capacity(1024)
    }

    /// Creates a directory table with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let mut table = Self {
            entries: Vec::with_capacity(capacity),
            lookup: HashMap::with_capacity(capacity),
        };
        // Insert root entry at index 0
        table.entries.push(DirectoryEntry {
            parent_id: ROOT_DIR_ID,
            name: String::new(),
        });
        table.lookup.insert(PathBuf::from("/"), ROOT_DIR_ID);
        table
    }

    /// Retrieves a directory ID for the given path if already present in the lookup table
    pub fn get_id(&self, path: &Path) -> Option<u32> {
        if path == Path::new("/") || path.as_os_str().is_empty() {
            return Some(ROOT_DIR_ID);
        }
        self.lookup.get(path).copied()
    }

    /// Inserts or retrieves a directory ID for the given path with $O(1)$ lookup
    pub fn get_or_insert(&mut self, path: &Path) -> u32 {
        if path == Path::new("/") || path.as_os_str().is_empty() {
            return ROOT_DIR_ID;
        }

        // Fast O(1) lookup
        if let Some(&id) = self.lookup.get(path) {
            return id;
        }

        // Resolve parent ID recursively
        let parent_id = match path.parent() {
            Some(parent) if parent != path && !parent.as_os_str().is_empty() => {
                self.get_or_insert(parent)
            }
            _ => ROOT_DIR_ID,
        };

        // Extract directory segment name
        let comp_name = match path.file_name() {
            Some(os_name) => os_name.to_string_lossy().into_owned(),
            None => match path.components().next_back() {
                Some(std::path::Component::ParentDir) => "..".to_string(),
                Some(std::path::Component::CurDir) => ".".to_string(),
                _ => String::new(),
            },
        };

        let id = self.entries.len() as u32;
        self.entries.push(DirectoryEntry {
            parent_id,
            name: comp_name,
        });
        self.lookup.insert(path.to_path_buf(), id);
        id
    }

    /// Resolves a full directory path by ID by walking parent pointers
    pub fn get_path(&self, id: u32) -> Option<PathBuf> {
        let idx = id as usize;
        if idx >= self.entries.len() {
            return None;
        }

        if id == ROOT_DIR_ID {
            return Some(PathBuf::from("/"));
        }

        let mut components = Vec::new();
        let mut curr = id;
        let mut depth = 0;
        const MAX_DEPTH: usize = 1024; // Cycle & stack overflow guard

        while curr != ROOT_DIR_ID {
            depth += 1;
            if depth > MAX_DEPTH {
                return None; // Guard against corrupt cycle
            }
            let entry = self.entries.get(curr as usize)?;
            components.push(&entry.name);
            curr = entry.parent_id;
        }

        let mut full_path = PathBuf::from("/");
        for comp in components.iter().rev() {
            full_path.push(comp);
        }
        Some(full_path)
    }

    /// Looks up a directory ID if already registered
    #[inline(always)]
    pub fn find_id(&self, path: &Path) -> Option<u32> {
        if path == Path::new("/") || path.as_os_str().is_empty() {
            return Some(ROOT_DIR_ID);
        }
        self.lookup.get(path).copied()
    }

    /// Retrieves a reference to a directory entry by ID
    #[inline(always)]
    pub fn get_entry(&self, id: u32) -> Option<&DirectoryEntry> {
        self.entries.get(id as usize)
    }

    /// Total number of unique directories indexed
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the directory table contains only root
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.entries.len() <= 1
    }

    /// Rebuilds the internal fast lookup map after deserialization
    pub fn rebuild_lookup(&mut self) {
        self.lookup.clear();
        self.lookup.reserve(self.entries.len());
        for (idx, _) in self.entries.iter().enumerate() {
            let id = idx as u32;
            if let Some(path) = self.get_path(id) {
                self.lookup.insert(path, id);
            }
        }
    }

    /// Estimates memory usage of the directory table in bytes
    pub fn memory_usage_bytes(&self) -> usize {
        let entries_mem = self.entries.capacity() * std::mem::size_of::<DirectoryEntry>()
            + self
                .entries
                .iter()
                .map(|e| e.name.capacity())
                .sum::<usize>();
        let lookup_mem = self.lookup.capacity()
            * (std::mem::size_of::<PathBuf>() + std::mem::size_of::<u32>() + 16);
        entries_mem + lookup_mem
    }

    /// Shrinks internal allocations to fit current contents exactly
    pub fn shrink_to_fit(&mut self) {
        self.entries.shrink_to_fit();
        self.lookup.shrink_to_fit();
    }

    /// Returns a slice of all directory entries
    #[inline(always)]
    pub fn entries(&self) -> &[DirectoryEntry] {
        &self.entries
    }

    /// Creates a directory table directly from directory entries
    pub fn from_entries(entries: Vec<DirectoryEntry>) -> Self {
        let mut table = Self {
            entries,
            lookup: HashMap::new(),
        };
        table.rebuild_lookup();
        table
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
        assert_eq!(path, Some(PathBuf::from("/home/user/docs")));
    }

    #[test]
    fn test_hierarchical_parent_pointers() {
        let mut table = DirectoryTable::new();
        let home_id = table.get_or_insert(Path::new("/home"));
        let user_id = table.get_or_insert(Path::new("/home/user"));
        let docs_id = table.get_or_insert(Path::new("/home/user/docs"));

        assert_eq!(
            table.get_entry(home_id).map(|e| e.parent_id),
            Some(ROOT_DIR_ID)
        );
        assert_eq!(table.get_entry(user_id).map(|e| e.parent_id), Some(home_id));
        assert_eq!(table.get_entry(docs_id).map(|e| e.parent_id), Some(user_id));

        assert_eq!(
            table.get_path(docs_id),
            Some(PathBuf::from("/home/user/docs"))
        );
    }

    #[test]
    fn test_root_path_handling() {
        let mut table = DirectoryTable::new();
        let id = table.get_or_insert(Path::new("/"));
        assert_eq!(id, ROOT_DIR_ID);
        assert_eq!(table.get_path(ROOT_DIR_ID), Some(PathBuf::from("/")));
    }

    #[test]
    fn test_rebuild_lookup() {
        let mut table = DirectoryTable::new();
        let id = table.get_or_insert(Path::new("/var/log/find-me"));
        table.lookup.clear();
        assert_eq!(table.find_id(Path::new("/var/log/find-me")), None);

        table.rebuild_lookup();
        assert_eq!(table.find_id(Path::new("/var/log/find-me")), Some(id));
    }

    #[test]
    fn test_invalid_id() {
        let table = DirectoryTable::new();
        assert_eq!(table.get_path(9999), None);
    }
}
