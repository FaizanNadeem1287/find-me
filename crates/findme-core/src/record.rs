//! Compact file record structures, bitflags, and path reconstruction

use crate::directory::DirectoryTable;
use crate::pool::StringPool;
use serde::{Deserialize, Serialize};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};
use std::path::PathBuf;

/// Bitflags representing file attributes and metadata (16 bits)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FileFlags(pub u16);

impl FileFlags {
    pub const NONE: Self = Self(0);
    pub const IS_DIR: Self = Self(1 << 0);
    pub const IS_SYMLINK: Self = Self(1 << 1);
    pub const IS_HIDDEN: Self = Self(1 << 2);
    pub const IS_EXECUTABLE: Self = Self(1 << 3);
    pub const IS_READONLY: Self = Self(1 << 4);
    pub const IS_DELETED: Self = Self(1 << 5);
    pub const IS_NETWORK: Self = Self(1 << 6);

    #[inline(always)]
    pub fn is_dir(&self) -> bool {
        (self.0 & Self::IS_DIR.0) != 0
    }

    #[inline(always)]
    pub fn is_symlink(&self) -> bool {
        (self.0 & Self::IS_SYMLINK.0) != 0
    }

    #[inline(always)]
    pub fn is_hidden(&self) -> bool {
        (self.0 & Self::IS_HIDDEN.0) != 0
    }

    #[inline(always)]
    pub fn is_executable(&self) -> bool {
        (self.0 & Self::IS_EXECUTABLE.0) != 0
    }

    #[inline(always)]
    pub fn is_readonly(&self) -> bool {
        (self.0 & Self::IS_READONLY.0) != 0
    }

    #[inline(always)]
    pub fn is_deleted(&self) -> bool {
        (self.0 & Self::IS_DELETED.0) != 0
    }

    #[inline(always)]
    pub fn is_network(&self) -> bool {
        (self.0 & Self::IS_NETWORK.0) != 0
    }

    #[inline(always)]
    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline(always)]
    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    #[inline(always)]
    pub fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }
}

impl BitOr for FileFlags {
    type Output = Self;
    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for FileFlags {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for FileFlags {
    type Output = Self;
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for FileFlags {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl Not for FileFlags {
    type Output = Self;
    #[inline(always)]
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

/// Compact representation of a file record stored in contiguous memory (exactly 32 bytes)
///
/// Designed with natural alignment to ensure zero internal compiler padding:
/// - `dir_id`: 4 bytes (offset 0..4, align 4)
/// - `name_offset`: 4 bytes (offset 4..8, align 4)
/// - `name_len`: 2 bytes (offset 8..10, align 2)
/// - `flags`: 2 bytes (offset 10..12, align 2)
/// - `_reserved`: 4 bytes (offset 12..16, align 4) -> explicit initialized field
/// - `size_bytes`: 8 bytes (offset 16..24, align 8)
/// - `mtime_secs`: 8 bytes (offset 24..32, align 8)
///
/// Total = 32 bytes, 8-byte aligned (2 records fit per 64-byte CPU cache line).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C)]
pub struct FileRecord {
    /// Index into the DirectoryTable
    pub dir_id: u32,
    /// Byte offset into the interned string buffer
    pub name_offset: u32,
    /// Length of the file name in bytes
    pub name_len: u16,
    /// File attribute bitflags
    pub flags: FileFlags,
    /// Explicit initialized padding / future expansion field
    pub _reserved: u32,
    /// File size in bytes (0 for directories)
    pub size_bytes: u64,
    /// Modification timestamp in seconds since UNIX epoch
    pub mtime_secs: u64,
}

impl FileRecord {
    /// Creates a new `FileRecord` with `_reserved` initialized to zero
    pub fn new(
        dir_id: u32,
        name_offset: u32,
        name_len: u16,
        flags: FileFlags,
        size_bytes: u64,
        mtime_secs: u64,
    ) -> Self {
        Self {
            dir_id,
            name_offset,
            name_len,
            flags,
            _reserved: 0,
            size_bytes,
            mtime_secs,
        }
    }

    /// Creates a `FileRecord` with a custom `_reserved` value
    pub fn with_reserved(
        dir_id: u32,
        name_offset: u32,
        name_len: u16,
        flags: FileFlags,
        reserved: u32,
        size_bytes: u64,
        mtime_secs: u64,
    ) -> Self {
        Self {
            dir_id,
            name_offset,
            name_len,
            flags,
            _reserved: reserved,
            size_bytes,
            mtime_secs,
        }
    }

    #[inline(always)]
    pub fn is_dir(&self) -> bool {
        self.flags.is_dir()
    }

    #[inline(always)]
    pub fn is_symlink(&self) -> bool {
        self.flags.is_symlink()
    }

    #[inline(always)]
    pub fn is_hidden(&self) -> bool {
        self.flags.is_hidden()
    }

    #[inline(always)]
    pub fn is_executable(&self) -> bool {
        self.flags.is_executable()
    }

    #[inline(always)]
    pub fn is_readonly(&self) -> bool {
        self.flags.is_readonly()
    }
}

/// Reconstructs the full POSIX filesystem path for a `FileRecord`
///
/// Returns `None` if the directory ID is invalid or the filename range
/// is out of bounds in the string pool.
pub fn reconstruct_path(
    record: &FileRecord,
    dirs: &DirectoryTable,
    pool: &StringPool,
) -> Option<PathBuf> {
    let mut dir_path = dirs.get_path(record.dir_id)?;
    let file_name = pool.get(record.name_offset, record.name_len)?;
    if !file_name.is_empty() {
        dir_path.push(file_name);
    }
    Some(dir_path)
}

/// Result item materialized for UI and CLI display
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub id: u64,
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub mtime_secs: u64,
    pub is_dir: bool,
}

impl SearchResultItem {
    /// Formats the raw byte count into human-readable binary units (B, KB, MB, GB)
    pub fn formatted_size(&self) -> String {
        if self.is_dir {
            return String::new();
        }

        const KB: u64 = 1024;
        const MB: u64 = 1024 * KB;
        const GB: u64 = 1024 * MB;

        if self.size_bytes >= GB {
            format!("{:.2} GB", self.size_bytes as f64 / GB as f64)
        } else if self.size_bytes >= MB {
            format!("{:.2} MB", self.size_bytes as f64 / MB as f64)
        } else if self.size_bytes >= KB {
            format!("{:.1} KB", self.size_bytes as f64 / KB as f64)
        } else {
            format!("{} B", self.size_bytes)
        }
    }

    /// Formats the epoch timestamp into a human-readable date/time string
    pub fn formatted_mtime(&self) -> String {
        let secs = self.mtime_secs;
        // UTC breakdown without heavy chrono dependency
        let days = secs / 86400;
        let rem_secs = secs % 86400;
        let hours = rem_secs / 3600;
        let minutes = (rem_secs % 3600) / 60;
        let seconds = rem_secs % 60;

        let (year, month, day) = days_to_ymd(days);
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            year, month, day, hours, minutes, seconds
        )
    }
}

/// Converts days since 1970-01-01 to (year, month, day)
fn days_to_ymd(days: u64) -> (i32, u32, u32) {
    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_file_record_memory_layout() {
        assert_eq!(
            std::mem::size_of::<FileRecord>(),
            32,
            "FileRecord must be exactly 32 bytes for memory efficiency"
        );
        assert_eq!(
            std::mem::align_of::<FileRecord>(),
            8,
            "FileRecord must be 8-byte aligned"
        );
    }

    #[test]
    fn test_file_flags_ops() {
        let mut flags = FileFlags::NONE;
        assert!(!flags.is_dir());
        assert!(!flags.is_hidden());
        assert!(!flags.is_readonly());

        flags |= FileFlags::IS_DIR | FileFlags::IS_HIDDEN | FileFlags::IS_READONLY;
        assert!(flags.is_dir());
        assert!(flags.is_hidden());
        assert!(flags.is_readonly());
        assert!(!flags.is_symlink());
        assert!(!flags.is_executable());

        flags.remove(FileFlags::IS_HIDDEN);
        assert!(!flags.is_hidden());
        assert!(flags.contains(FileFlags::IS_DIR));
    }

    #[test]
    fn test_path_reconstruction() {
        let mut dirs = DirectoryTable::new();
        let mut pool = StringPool::new();

        let dir_id = dirs.get_or_insert(Path::new("/home/user/projects/Find-Me"));
        let (name_off, name_len) = pool.append("Cargo.toml").expect("append failed");

        let record = FileRecord::new(
            dir_id,
            name_off,
            name_len,
            FileFlags::NONE,
            1024,
            1763740000,
        );

        let reconstructed = reconstruct_path(&record, &dirs, &pool);
        assert_eq!(
            reconstructed,
            Some(PathBuf::from("/home/user/projects/Find-Me/Cargo.toml"))
        );
    }

    #[test]
    fn test_size_formatting() {
        let item = SearchResultItem {
            id: 1,
            name: "test.iso".into(),
            path: "/path/test.iso".into(),
            size_bytes: 1_500_000_000,
            mtime_secs: 1763740000,
            is_dir: false,
        };
        assert_eq!(item.formatted_size(), "1.40 GB");
    }
}
