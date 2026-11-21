//! Zero-copy binary index persistence and cold startup storage
//!
//! Provides ultra-fast serialization of `FileArena` to disk using `bincode`
//! and memory-mapped deserialization via `memmap2`, with 32-byte header validation,
//! SIMD-accelerated CRC32 checksums, and atomic swap writes.

use crate::arena::FileArena;
use crate::error::{EngineError, Result};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Magic bytes identifying a valid Find-Me binary index file: "FINDME\x01\0"
pub const STORAGE_MAGIC: &[u8; 8] = b"FINDME\x01\0";

/// Current index schema version
pub const SCHEMA_VERSION: u32 = 1;

/// Fixed header size in bytes
pub const HEADER_SIZE: usize = 32;

/// Fixed-size 32-byte binary header for index file validation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StorageHeader {
    /// Magic 8-byte signature: `FINDME\x01`
    pub magic: [u8; 8],
    /// Binary schema version
    pub version: u32,
    /// Unix timestamp in seconds when the index was written
    pub timestamp: u64,
    /// Total count of records contained in the index
    pub record_count: u64,
    /// IEEE 802.3 CRC32 checksum of the serialized payload
    pub crc32: u32,
}

impl StorageHeader {
    /// Creates a new `StorageHeader` for the current timestamp and given record count and checksum
    pub fn new(record_count: u64, crc32: u32) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            magic: *STORAGE_MAGIC,
            version: SCHEMA_VERSION,
            timestamp,
            record_count,
            crc32,
        }
    }

    /// Serializes the header into a fixed 32-byte array
    pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut buf = [0u8; HEADER_SIZE];
        buf[0..8].copy_from_slice(&self.magic);
        buf[8..12].copy_from_slice(&self.version.to_le_bytes());
        buf[12..20].copy_from_slice(&self.timestamp.to_le_bytes());
        buf[20..28].copy_from_slice(&self.record_count.to_le_bytes());
        buf[28..32].copy_from_slice(&self.crc32.to_le_bytes());
        buf
    }

    /// Deserializes and validates a header from a byte slice of at least 32 bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < HEADER_SIZE {
            return Err(EngineError::Persistence(format!(
                "Index file header truncated: expected at least {HEADER_SIZE} bytes, found {}",
                bytes.len()
            )));
        }

        let mut magic = [0u8; 8];
        magic.copy_from_slice(&bytes[0..8]);
        if &magic != STORAGE_MAGIC {
            return Err(EngineError::Persistence(format!(
                "Invalid index magic bytes: expected {:?}, found {:?}",
                STORAGE_MAGIC, magic
            )));
        }

        let mut ver_bytes = [0u8; 4];
        ver_bytes.copy_from_slice(&bytes[8..12]);
        let version = u32::from_le_bytes(ver_bytes);
        if version != SCHEMA_VERSION {
            return Err(EngineError::Persistence(format!(
                "Unsupported index schema version {version}: expected {SCHEMA_VERSION}"
            )));
        }

        let mut time_bytes = [0u8; 8];
        time_bytes.copy_from_slice(&bytes[12..20]);
        let timestamp = u64::from_le_bytes(time_bytes);

        let mut count_bytes = [0u8; 8];
        count_bytes.copy_from_slice(&bytes[20..28]);
        let record_count = u64::from_le_bytes(count_bytes);

        let mut crc_bytes = [0u8; 4];
        crc_bytes.copy_from_slice(&bytes[28..32]);
        let crc32 = u32::from_le_bytes(crc_bytes);

        Ok(Self {
            magic,
            version,
            timestamp,
            record_count,
            crc32,
        })
    }
}

/// Persistent storage manager for binary file indexes
pub struct IndexStorage;

impl IndexStorage {
    /// Returns the standard FreeDesktop XDG cache directory for Find-Me
    ///
    /// Resolves `$XDG_CACHE_HOME/find-me` or `~/.cache/find-me`.
    pub fn default_cache_dir() -> PathBuf {
        if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
            if !xdg.trim().is_empty() {
                return PathBuf::from(xdg).join("find-me");
            }
        }

        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "findme", "find-me") {
            proj_dirs.cache_dir().to_path_buf()
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".cache").join("find-me")
        } else {
            PathBuf::from("/tmp/find-me-cache")
        }
    }

    /// Returns the default index file path: `$XDG_CACHE_HOME/find-me/index.bin`
    pub fn default_cache_path() -> PathBuf {
        Self::default_cache_dir().join("index.bin")
    }

    /// Atomically saves the `FileArena` to disk with header and CRC32 verification
    ///
    /// Writes to a temporary file in the same directory (`.index.bin.tmp.<pid>.<ts>`),
    /// issues `sync_all()`, and executes an atomic POSIX rename to prevent partial
    /// writes or corruption during sudden power loss.
    pub fn save_to_file(arena: &FileArena, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| EngineError::Io {
                    path: parent.to_path_buf(),
                    source: e,
                })?;
            }
        }

        // 1. Serialize DirectoryTable with bincode
        let dirs_bytes = bincode::serialize(arena.dirs()).map_err(|e| {
            EngineError::Persistence(format!("Failed to serialize DirectoryTable: {e}"))
        })?;

        // 2. String pool bytes
        let pool_bytes = arena.pool().as_bytes();

        // 3. Records slice
        let records = arena.records();
        let record_count = records.len() as u64;
        let record_byte_len = std::mem::size_of_val(records);
        let records_raw_slice =
            unsafe { std::slice::from_raw_parts(records.as_ptr() as *const u8, record_byte_len) };

        // Total payload layout:
        // dirs_len (8) + dirs_bytes + pool_len (8) + pool_bytes + record_count (8) + records_bytes
        let total_payload_len = 8 + dirs_bytes.len() + 8 + pool_bytes.len() + 8 + record_byte_len;
        let mut payload = Vec::with_capacity(total_payload_len);

        payload.extend_from_slice(&(dirs_bytes.len() as u64).to_le_bytes());
        payload.extend_from_slice(&dirs_bytes);

        payload.extend_from_slice(&(pool_bytes.len() as u64).to_le_bytes());
        payload.extend_from_slice(pool_bytes);

        payload.extend_from_slice(&record_count.to_le_bytes());
        payload.extend_from_slice(records_raw_slice);

        // Calculate SIMD CRC32 checksum of payload
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&payload);
        let checksum = hasher.finalize();

        // Construct 32-byte header
        let header = StorageHeader::new(arena.len() as u64, checksum);
        let header_bytes = header.to_bytes();

        // Create unique temporary file in the same target directory for atomic rename
        let parent_dir = path.parent().unwrap_or_else(|| Path::new("."));
        let file_stem = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("index.bin");
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let temp_filename = format!(".{file_stem}.tmp.{}.{nanos}", std::process::id());
        let temp_path = parent_dir.join(temp_filename);

        // Open temp file and write header + payload
        let write_res = (|| -> Result<()> {
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&temp_path)
                .map_err(|e| EngineError::Io {
                    path: temp_path.clone(),
                    source: e,
                })?;

            let mut writer = BufWriter::with_capacity(128 * 1024, file);

            writer
                .write_all(&header_bytes)
                .map_err(|e| EngineError::Io {
                    path: temp_path.clone(),
                    source: e,
                })?;

            writer.write_all(&payload).map_err(|e| EngineError::Io {
                path: temp_path.clone(),
                source: e,
            })?;

            writer.flush().map_err(|e| EngineError::Io {
                path: temp_path.clone(),
                source: e,
            })?;

            // Fsync data and metadata to disk before rename
            writer.get_ref().sync_all().map_err(|e| EngineError::Io {
                path: temp_path.clone(),
                source: e,
            })?;

            Ok(())
        })();

        if let Err(e) = write_res {
            let _ = std::fs::remove_file(&temp_path);
            return Err(e);
        }

        // Atomic swap
        std::fs::rename(&temp_path, path).map_err(|e| {
            let _ = std::fs::remove_file(&temp_path);
            EngineError::Io {
                path: path.to_path_buf(),
                source: e,
            }
        })?;

        Ok(())
    }

    /// Loads and verifies a `FileArena` from the specified binary index file
    ///
    /// Employs zero-copy memory mapping (`memmap2`), validates the 32-byte header,
    /// checks schema version, computes and validates the CRC32 checksum, deserializes
    /// the arena payload, and rebuilds fast lookup tables.
    pub fn load_from_file(path: &Path) -> Result<FileArena> {
        let file = File::open(path).map_err(|e| EngineError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        let metadata = file.metadata().map_err(|e| EngineError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        let file_len = metadata.len();
        if file_len < HEADER_SIZE as u64 {
            return Err(EngineError::Persistence(format!(
                "Index file '{}' is truncated: length {file_len} bytes < header size {HEADER_SIZE} bytes",
                path.display()
            )));
        }

        // Memory-map the file
        // Safety: We hold an open read-only File handle and will not truncate or modify it.
        let mmap = unsafe {
            memmap2::MmapOptions::new()
                .map(&file)
                .map_err(|e| EngineError::Io {
                    path: path.to_path_buf(),
                    source: e,
                })?
        };

        // Header validation
        let header = StorageHeader::from_bytes(&mmap[0..HEADER_SIZE])?;
        let payload = &mmap[HEADER_SIZE..];

        // CRC32 checksum validation
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(payload);
        let computed_crc = hasher.finalize();

        if computed_crc != header.crc32 {
            return Err(EngineError::Persistence(format!(
                "Index file '{}' CRC32 checksum mismatch: expected {:#010x}, calculated {:#010x}",
                path.display(),
                header.crc32,
                computed_crc
            )));
        }

        // Fast sectional deserialization
        if payload.len() < 24 {
            return Err(EngineError::Persistence(format!(
                "Index file '{}' payload truncated: length {} bytes < minimum 24 bytes",
                path.display(),
                payload.len()
            )));
        }

        let mut offset = 0;

        // 1. DirectoryTable
        let mut dirs_len_bytes = [0u8; 8];
        dirs_len_bytes.copy_from_slice(&payload[offset..offset + 8]);
        let dirs_len = u64::from_le_bytes(dirs_len_bytes) as usize;
        offset += 8;

        if offset
            .checked_add(dirs_len)
            .is_none_or(|end| end > payload.len())
        {
            return Err(EngineError::Persistence(
                "Directory section exceeds payload bounds".to_string(),
            ));
        }
        let dirs_bytes = &payload[offset..offset + dirs_len];
        offset += dirs_len;

        let mut dirs: crate::directory::DirectoryTable =
            bincode::deserialize(dirs_bytes).map_err(|e| {
                EngineError::Persistence(format!("Failed to deserialize DirectoryTable: {e}"))
            })?;
        dirs.rebuild_lookup();

        // 2. StringPool
        if offset + 8 > payload.len() {
            return Err(EngineError::Persistence(
                "String pool length exceeds payload bounds".to_string(),
            ));
        }
        let mut pool_len_bytes = [0u8; 8];
        pool_len_bytes.copy_from_slice(&payload[offset..offset + 8]);
        let pool_len = u64::from_le_bytes(pool_len_bytes) as usize;
        offset += 8;

        if offset
            .checked_add(pool_len)
            .is_none_or(|end| end > payload.len())
        {
            return Err(EngineError::Persistence(
                "String pool buffer exceeds payload bounds".to_string(),
            ));
        }
        let pool_bytes = &payload[offset..offset + pool_len];
        offset += pool_len;

        let pool = crate::pool::StringPool::from_bytes(pool_bytes.to_vec());

        // 3. FileRecord slice
        if offset + 8 > payload.len() {
            return Err(EngineError::Persistence(
                "Record count header exceeds payload bounds".to_string(),
            ));
        }
        let mut count_bytes = [0u8; 8];
        count_bytes.copy_from_slice(&payload[offset..offset + 8]);
        let record_count = u64::from_le_bytes(count_bytes) as usize;
        offset += 8;

        let expected_records_bytes = record_count
            .checked_mul(std::mem::size_of::<crate::record::FileRecord>())
            .ok_or_else(|| {
                EngineError::Persistence("Record payload length calculation overflow".to_string())
            })?;

        if offset
            .checked_add(expected_records_bytes)
            .is_none_or(|end| end != payload.len())
        {
            return Err(EngineError::Persistence(format!(
                "Record payload length mismatch: expected {expected_records_bytes} bytes, remaining {}",
                payload.len().saturating_sub(offset)
            )));
        }

        let mut records = Vec::with_capacity(record_count);
        if record_count > 0 {
            let src_ptr = payload[offset..].as_ptr();
            unsafe {
                std::ptr::copy_nonoverlapping(
                    src_ptr,
                    records.as_mut_ptr() as *mut u8,
                    expected_records_bytes,
                );
                records.set_len(record_count);
            }
        }

        let arena = FileArena::from_parts(records, pool, dirs);

        if (arena.len() as u64) != header.record_count {
            tracing::warn!(
                "Header record count mismatch for '{}': header={}, actual={}",
                path.display(),
                header.record_count,
                arena.len()
            );
        }

        Ok(arena)
    }

    /// Verifies the header and payload integrity of an index file without returning the full arena
    pub fn verify_file(path: &Path) -> Result<StorageHeader> {
        let file = File::open(path).map_err(|e| EngineError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        let metadata = file.metadata().map_err(|e| EngineError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        let file_len = metadata.len();
        if file_len < HEADER_SIZE as u64 {
            return Err(EngineError::Persistence(format!(
                "Index file '{}' is truncated: length {file_len} bytes < header size {HEADER_SIZE} bytes",
                path.display()
            )));
        }

        let mmap = unsafe {
            memmap2::MmapOptions::new()
                .map(&file)
                .map_err(|e| EngineError::Io {
                    path: path.to_path_buf(),
                    source: e,
                })?
        };

        let header = StorageHeader::from_bytes(&mmap[0..HEADER_SIZE])?;
        let payload = &mmap[HEADER_SIZE..];

        let mut hasher = crc32fast::Hasher::new();
        hasher.update(payload);
        let computed_crc = hasher.finalize();

        if computed_crc != header.crc32 {
            return Err(EngineError::Persistence(format!(
                "Index file '{}' CRC32 checksum mismatch: expected {:#010x}, calculated {:#010x}",
                path.display(),
                header.crc32,
                computed_crc
            )));
        }

        Ok(header)
    }

    /// Attempts to load the index from disk; on any error (missing, corrupt, outdated),
    /// logs a warning and returns `None` so the caller can seamlessly trigger a fresh crawl
    pub fn load_or_fallback(path: &Path) -> Option<FileArena> {
        match Self::load_from_file(path) {
            Ok(arena) => Some(arena),
            Err(err) => {
                tracing::warn!(
                    "Cache load failed for '{}': {err}. Triggering fresh background crawl.",
                    path.display()
                );
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::FileFlags;
    use tempfile::tempdir;

    #[test]
    fn test_storage_header_serialization() {
        let header = StorageHeader::new(42, 0x12345678);
        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), HEADER_SIZE);

        let parsed = StorageHeader::from_bytes(&bytes).expect("Valid header must parse");
        assert_eq!(parsed.magic, *STORAGE_MAGIC);
        assert_eq!(parsed.version, SCHEMA_VERSION);
        assert_eq!(parsed.record_count, 42);
        assert_eq!(parsed.crc32, 0x12345678);
    }

    #[test]
    fn test_storage_header_invalid_magic() {
        let mut bytes = [0u8; HEADER_SIZE];
        bytes[0..8].copy_from_slice(b"BADMAGIC");
        let res = StorageHeader::from_bytes(&bytes);
        assert!(res.is_err());
    }

    #[test]
    fn test_storage_header_unsupported_version() {
        let mut header = StorageHeader::new(10, 0);
        header.version = 999;
        let bytes = header.to_bytes();
        let res = StorageHeader::from_bytes(&bytes);
        assert!(res.is_err());
    }

    #[test]
    fn test_storage_roundtrip_empty() {
        let dir = tempdir().expect("tempdir");
        let cache_path = dir.path().join("index.bin");
        let arena = FileArena::new();

        IndexStorage::save_to_file(&arena, &cache_path).expect("Save empty arena");
        assert!(cache_path.exists());

        let loaded = IndexStorage::load_from_file(&cache_path).expect("Load empty arena");
        assert_eq!(loaded.len(), 0);
    }

    #[test]
    fn test_storage_roundtrip_with_records() {
        let dir = tempdir().expect("tempdir");
        let cache_path = dir.path().join("index.bin");
        let mut arena = FileArena::new();

        arena
            .add_file(
                Path::new("/home/user/document.pdf"),
                FileFlags::NONE,
                1024,
                1600000000,
            )
            .expect("add_file 1");
        arena
            .add_file(
                Path::new("/var/log/syslog"),
                FileFlags::IS_READONLY,
                2048,
                1600000100,
            )
            .expect("add_file 2");

        IndexStorage::save_to_file(&arena, &cache_path).expect("Save populated arena");
        let loaded = IndexStorage::load_from_file(&cache_path).expect("Load populated arena");

        assert_eq!(loaded.len(), 2);

        // Path reconstruction verification
        let p1 = loaded.reconstruct_path_by_index(0).expect("path 0");
        assert_eq!(p1, PathBuf::from("/home/user/document.pdf"));

        let p2 = loaded.reconstruct_path_by_index(1).expect("path 1");
        assert_eq!(p2, PathBuf::from("/var/log/syslog"));

        // Lookup table verification
        let idx0 = loaded
            .find_record_by_path(Path::new("/home/user/document.pdf"))
            .expect("find record 0");
        assert_eq!(idx0, 0);
    }
}
