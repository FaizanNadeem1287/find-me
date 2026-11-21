//! Unified in-memory index arena holding records, strings, and directory tables

use crate::directory::DirectoryTable;
use crate::error::{EngineError, Result};
use crate::pool::StringPool;
use crate::record::{reconstruct_path, FileFlags, FileRecord};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Unified in-memory file index container
///
/// Encapsulates the flat 32-byte `FileRecord` array, continuous `StringPool`,
/// and deduplicated `DirectoryTable`, providing high-speed insertion, path
/// reconstruction, and memory footprint accounting.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FileArena {
    records: Vec<FileRecord>,
    pool: StringPool,
    dirs: DirectoryTable,
}

impl FileArena {
    /// Creates a new empty `FileArena`
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            pool: StringPool::new(),
            dirs: DirectoryTable::new(),
        }
    }

    /// Creates a `FileArena` with pre-allocated capacities
    pub fn with_capacity(records: usize, string_bytes: usize, dirs: usize) -> Self {
        Self {
            records: Vec::with_capacity(records),
            pool: StringPool::with_capacity(string_bytes),
            dirs: DirectoryTable::with_capacity(dirs),
        }
    }

    /// Adds a file entry given its full path and metadata
    ///
    /// Resolves the parent directory into `DirectoryTable`, stores the
    /// file name in `StringPool`, and appends a `FileRecord`. Returns the
    /// index of the newly added record.
    pub fn add_file(
        &mut self,
        path: &Path,
        flags: FileFlags,
        size_bytes: u64,
        mtime_secs: u64,
    ) -> Result<u32> {
        let parent = match path.parent() {
            Some(p) if !p.as_os_str().is_empty() => p,
            _ => Path::new("/"),
        };
        let dir_id = self.dirs.get_or_insert(parent);

        let file_name = match path.file_name() {
            Some(name) => name.to_string_lossy(),
            None => std::borrow::Cow::Borrowed(""),
        };

        let (name_offset, name_len) = self.pool.append(&file_name)?;

        let record_idx = self.records.len();
        if record_idx > u32::MAX as usize {
            return Err(EngineError::InvalidRecordIndex(record_idx));
        }

        let record = FileRecord::new(dir_id, name_offset, name_len, flags, size_bytes, mtime_secs);
        self.records.push(record);

        Ok(record_idx as u32)
    }

    /// Adds an already constructed `FileRecord` directly to the arena
    pub fn add_record(&mut self, record: FileRecord) -> u32 {
        let idx = self.records.len() as u32;
        self.records.push(record);
        idx
    }

    /// Retrieves a file record by index
    #[inline(always)]
    pub fn get(&self, index: usize) -> Option<&FileRecord> {
        self.records.get(index)
    }

    /// Reconstructs the full path of a given `FileRecord`
    pub fn reconstruct_path(&self, record: &FileRecord) -> Option<PathBuf> {
        reconstruct_path(record, &self.dirs, &self.pool)
    }

    /// Reconstructs the full path of the record at the given index
    pub fn reconstruct_path_by_index(&self, index: usize) -> Option<PathBuf> {
        let record = self.get(index)?;
        self.reconstruct_path(record)
    }

    /// Returns the file name of the record at the given index
    #[inline(always)]
    pub fn get_name(&self, index: usize) -> Option<&str> {
        let record = self.records.get(index)?;
        self.pool.get(record.name_offset, record.name_len)
    }

    /// Returns the file name for a given record
    #[inline(always)]
    pub fn get_record_name(&self, record: &FileRecord) -> Option<&str> {
        self.pool.get(record.name_offset, record.name_len)
    }

    /// Read-only slice of all file records in the arena
    #[inline(always)]
    pub fn records(&self) -> &[FileRecord] {
        &self.records
    }

    /// Mutable access to file records (for inotify mutations or updates)
    #[inline(always)]
    pub fn records_mut(&mut self) -> &mut Vec<FileRecord> {
        &mut self.records
    }

    /// Read-only reference to the underlying string pool
    #[inline(always)]
    pub fn pool(&self) -> &StringPool {
        &self.pool
    }

    /// Mutable reference to the underlying string pool
    #[inline(always)]
    pub fn pool_mut(&mut self) -> &mut StringPool {
        &mut self.pool
    }

    /// Read-only reference to the directory table
    #[inline(always)]
    pub fn dirs(&self) -> &DirectoryTable {
        &self.dirs
    }

    /// Mutable reference to the directory table
    #[inline(always)]
    pub fn dirs_mut(&mut self) -> &mut DirectoryTable {
        &mut self.dirs
    }

    /// Total count of indexed records
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Whether the arena contains zero records
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Total heap memory consumption of the arena in bytes
    pub fn memory_usage_bytes(&self) -> usize {
        let records_mem = self.records.capacity() * std::mem::size_of::<FileRecord>();
        let pool_mem = self.pool.capacity();
        let dirs_mem = self.dirs.memory_usage_bytes();
        records_mem + pool_mem + dirs_mem
    }

    /// Average memory usage per record in bytes
    pub fn average_bytes_per_record(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        self.memory_usage_bytes() as f64 / self.records.len() as f64
    }

    /// Shrinks all internal buffers to fit their contents exactly
    pub fn shrink_to_fit(&mut self) {
        self.records.shrink_to_fit();
        self.pool.shrink_to_fit();
        self.dirs.shrink_to_fit();
    }

    /// Clears all indexed data
    pub fn clear(&mut self) {
        self.records.clear();
        self.pool.clear();
        self.dirs = DirectoryTable::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_arena_basic_operations() {
        let mut arena = FileArena::new();
        let idx = arena
            .add_file(
                Path::new("/home/user/document.pdf"),
                FileFlags::NONE,
                2048,
                1763741000,
            )
            .expect("add_file failed");

        assert_eq!(arena.len(), 1);
        let record = arena.get(idx as usize).expect("record must exist");
        assert_eq!(record.size_bytes, 2048);
        assert_eq!(record.mtime_secs, 1763741000);

        let path = arena.reconstruct_path_by_index(idx as usize);
        assert_eq!(path, Some(PathBuf::from("/home/user/document.pdf")));
    }

    #[test]
    fn test_file_arena_unicode_and_spaces() {
        let mut arena = FileArena::new();
        let idx = arena
            .add_file(
                Path::new("/home/user/Downloads/🦀_test file with spaces_🚀.tar.gz"),
                FileFlags::IS_READONLY,
                5000,
                1763742000,
            )
            .expect("add_file unicode failed");

        let path = arena.reconstruct_path_by_index(idx as usize);
        assert_eq!(
            path,
            Some(PathBuf::from(
                "/home/user/Downloads/🦀_test file with spaces_🚀.tar.gz"
            ))
        );
    }

    #[test]
    fn test_memory_footprint_budget() {
        let mut arena = FileArena::with_capacity(100_000, 1_500_000, 2_000);

        // Preload 1,000 distinct directories
        let mut dir_paths = Vec::with_capacity(1_000);
        for d in 0..1_000 {
            dir_paths.push(format!(
                "/home/user/projects/proj_{:03}/sub_{:02}",
                d / 10,
                d % 10
            ));
        }

        // Add 100,000 files with typical 10-character filenames (e.g. f_00001.rs, mod.rs, main.rs)
        for i in 0..100_000 {
            let dir = &dir_paths[i % 1_000];
            let file_name = format!("f_{:05}.rs", i);
            let full_path = format!("{}/{}", dir, file_name);
            arena
                .add_file(
                    Path::new(&full_path),
                    FileFlags::NONE,
                    (i * 100) as u64,
                    1763740000 + (i as u64),
                )
                .expect("failed to add file in memory benchmark");
        }

        arena.shrink_to_fit();

        let avg_bytes = arena.average_bytes_per_record();
        assert_eq!(arena.len(), 100_000);
        // Acceptance criterion: strictly < 45 bytes total per record including string heap
        assert!(
            avg_bytes < 45.0,
            "Memory budget exceeded: got {:.2} bytes/record, expected < 45.0 bytes",
            avg_bytes
        );
    }
}
