# POSIX Stat & Metadata Extraction

This guide details extracting Linux file metadata with maximum efficiency using native POSIX system calls.

---

## 1. Extracting Inodes and Device IDs in Rust

On Linux, every file and directory is uniquely identified across the operating system by the tuple:
$$(\text{st\_dev}, \text{st\_ino})$$

Using Rust's standard UNIX extension `std::os::unix::fs::MetadataExt`:

```rust
use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FileIdentifier {
    pub dev: u64,
    pub ino: u64,
}

impl FileIdentifier {
    #[inline(always)]
    pub fn from_metadata(meta: &Metadata) -> Self {
        Self {
            dev: meta.dev(),
            ino: meta.ino(),
        }
    }
}
```

---

## 2. Fast POSIX Mode Inspection

File types and permissions are packed inside `st_mode`:

```rust
use std::os::unix::fs::MetadataExt;

pub fn extract_flags(meta: &std::fs::Metadata, is_hidden: bool) -> findme_core::FileFlags {
    let mut flags = findme_core::FileFlags::NONE;
    let mode = meta.mode();

    if meta.is_dir() {
        flags.insert(findme_core::FileFlags::IS_DIR);
    }
    if meta.is_symlink() {
        flags.insert(findme_core::FileFlags::IS_SYMLINK);
    }
    if is_hidden {
        flags.insert(findme_core::FileFlags::IS_HIDDEN);
    }
    // Check if executable by user/group/other (S_IXUSR | S_IXGRP | S_IXOTH)
    if (mode & 0o111) != 0 {
        flags.insert(findme_core::FileFlags::IS_EXECUTABLE);
    }

    flags
}
```
