//! Append-only flat byte buffer arena for string interning and zero-copy borrowing

use crate::error::{EngineError, Result};
use serde::{Deserialize, Serialize};

/// Continuous append-only byte arena for file names and paths
///
/// Stores UTF-8 strings contiguously in a single buffer, returning `(offset, len)`
/// pairs. Borrowing via `get()` yields zero-allocation `&str` references.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StringPool {
    buffer: Vec<u8>,
}

impl StringPool {
    /// Creates a new empty string pool
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Creates a string pool pre-allocated with the specified byte capacity
    pub fn with_capacity(capacity_bytes: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity_bytes),
        }
    }

    /// Appends a UTF-8 string slice to the pool, returning `(offset, len)`
    ///
    /// # Errors
    /// - `EngineError::NameTooLong`: If `s.len() > u16::MAX` (65,535 bytes).
    /// - `EngineError::StringPoolOverflow`: If pool size exceeds `u32::MAX` (4 GB).
    pub fn append(&mut self, s: &str) -> Result<(u32, u16)> {
        let len = s.len();
        if len > u16::MAX as usize {
            return Err(EngineError::NameTooLong(len));
        }

        let offset = self.buffer.len();
        if offset
            .checked_add(len)
            .is_none_or(|end| end > u32::MAX as usize)
        {
            return Err(EngineError::StringPoolOverflow(offset + len));
        }

        self.buffer.extend_from_slice(s.as_bytes());
        Ok((offset as u32, len as u16))
    }

    /// Borrows a string slice from the pool by `offset` and `len`
    ///
    /// Returns `None` if the offset/length range is out of bounds or if
    /// the slice contains invalid UTF-8 (which cannot happen through `append`).
    #[inline(always)]
    pub fn get(&self, offset: u32, len: u16) -> Option<&str> {
        let start = offset as usize;
        let end = start.checked_add(len as usize)?;
        let bytes = self.buffer.get(start..end)?;
        std::str::from_utf8(bytes).ok()
    }

    /// Total number of bytes stored in the string pool
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Whether the string pool is empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Allocated byte capacity of the internal buffer
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Shrinks internal buffer capacity to match length exactly
    pub fn shrink_to_fit(&mut self) {
        self.buffer.shrink_to_fit();
    }

    /// Clears all strings from the pool
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_append_and_get() {
        let mut pool = StringPool::new();
        let (off1, len1) = pool.append("main.rs").expect("failed to append");
        let (off2, len2) = pool.append("Cargo.toml").expect("failed to append");

        assert_eq!(pool.get(off1, len1), Some("main.rs"));
        assert_eq!(pool.get(off2, len2), Some("Cargo.toml"));
    }

    #[test]
    fn test_pool_multibyte_utf8() {
        let mut pool = StringPool::new();
        // Emojis, CJK, German umlauts, spaces
        let test_strings = [
            "🦀_find_me_🚀.rs",
            "测试文件_日本語.txt",
            "über_größen_märchen.pdf",
            "file with multiple spaces and .dots.tar.gz",
        ];

        let mut handles = Vec::new();
        for s in &test_strings {
            let handle = pool.append(s).expect("failed to append unicode");
            handles.push(handle);
        }

        for (i, &(off, len)) in handles.iter().enumerate() {
            assert_eq!(pool.get(off, len), Some(test_strings[i]));
        }
    }

    #[test]
    fn test_pool_empty_string() {
        let mut pool = StringPool::new();
        let (off, len) = pool.append("").expect("failed to append empty string");
        assert_eq!(len, 0);
        assert_eq!(pool.get(off, len), Some(""));
    }

    #[test]
    fn test_pool_out_of_bounds() {
        let mut pool = StringPool::new();
        let _ = pool.append("hello").expect("append failed");

        assert_eq!(pool.get(100, 5), None);
        assert_eq!(pool.get(0, 100), None);
        assert_eq!(pool.get(u32::MAX, 1), None);
    }

    #[test]
    fn test_pool_name_too_long() {
        let mut pool = StringPool::new();
        let huge_name = "a".repeat(u16::MAX as usize + 1);
        let result = pool.append(&huge_name);
        assert!(matches!(result, Err(EngineError::NameTooLong(_))));
    }

    #[test]
    fn test_pool_capacity_and_shrink() {
        let mut pool = StringPool::with_capacity(1024);
        assert!(pool.capacity() >= 1024);
        let _ = pool.append("find-me").expect("append failed");
        pool.shrink_to_fit();
        assert_eq!(pool.capacity(), pool.len());
    }
}
