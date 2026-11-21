//! Compact file record structures and search result types

use serde::{Deserialize, Serialize};

/// Bitflags representing file attributes and metadata
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FileFlags(pub u16);

impl FileFlags {
    pub const NONE: Self = Self(0);
    pub const IS_DIR: Self = Self(1 << 0);
    pub const IS_SYMLINK: Self = Self(1 << 1);
    pub const IS_HIDDEN: Self = Self(1 << 2);
    pub const IS_EXECUTABLE: Self = Self(1 << 3);

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
    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }
}

/// Compact representation of a file record stored in contiguous memory (32 bytes)
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
    /// File size in bytes (0 for directories)
    pub size_bytes: u64,
    /// Modification timestamp in seconds since UNIX epoch
    pub mtime_secs: u64,
}

impl FileRecord {
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
            size_bytes,
            mtime_secs,
        }
    }
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
        // Simple UTC breakdown without heavy chrono dependency
        let days = secs / 86400;
        let rem_secs = secs % 86400;
        let hours = rem_secs / 3600;
        let minutes = (rem_secs % 3600) / 60;
        let seconds = rem_secs % 60;

        // Approximate year/month from days (civil calendar calculation)
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

    #[test]
    fn test_file_record_size() {
        assert_eq!(
            std::mem::size_of::<FileRecord>(),
            32,
            "FileRecord must be exactly 32 bytes for memory efficiency"
        );
    }

    #[test]
    fn test_file_flags() {
        let mut flags = FileFlags::NONE;
        assert!(!flags.is_dir());
        assert!(!flags.is_hidden());

        flags.insert(FileFlags::IS_DIR);
        flags.insert(FileFlags::IS_HIDDEN);

        assert!(flags.is_dir());
        assert!(flags.is_hidden());
        assert!(!flags.is_symlink());
        assert!(!flags.is_executable());
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
