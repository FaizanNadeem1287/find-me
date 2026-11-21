# Indexing & Search Engine Architecture: Linux Implementation

This guide documents how Find-Me achieves the speed of Windows *Everything* on Linux without relying on NTFS-specific APIs, kernel hacks, or security workarounds.

---

## 1. The Windows Everything Architecture vs. Linux Challenge

### How Windows "Everything" Works:
1. **Direct MFT Parsing**: Windows Everything opens NTFS volumes directly and reads the Master File Table ($MFT) records. Because NTFS stores all directory hierarchy and file metadata in a single contiguous table, reading 1 million files requires reading ~1 GB of sequential blocks, which takes 1-2 seconds on an SSD.
2. **USN Change Journal**: Everything reads NTFS update logs to capture file creations, deletions, and renames instantly.

### Why Direct Disk Parsing Cannot Be Used on Linux:
1. **Security & Permissions**: Reading block devices directly (`/dev/nvme0n1p2`, `/dev/sda1`) requires `root` privileges (`CAP_SYS_RAWIO`). Running a desktop GUI search tool as root is a massive security hazard.
2. **Filesystem Fragmentation**: Linux systems employ a multitude of filesystems simultaneously: `ext4`, `btrfs` (subvolumes & copy-on-write trees), `xfs`, `zfs`, `f2fs`, `tmpfs`, and network mounts (`nfs`, `cifs`). There is no single universal on-disk format.
3. **Encryption & Abstraction**: Modern Linux desktops use LUKS (dm-crypt), LVM, or user-level fscrypt. Direct disk reading cannot parse encrypted inodes without decrypting blocks.

---

## 2. The Production-Grade Linux Solution (Zero Workarounds)

Find-Me achieves sub-second crawling and sub-millisecond search through a 4-tier architecture:

```
[ Tier 1: Multithreaded Parallel Crawler (jwalk / Rayon) ]
                       |
                       v
[ Tier 2: Flat In-Memory Arena & String Interner (Compact Structs) ]
       |                                              |
       v                                              v
[ Tier 3: nucleo-matcher / Substring ]       [ Tier 4: inotify Watcher (notify) ]
       |                                              |
       v                                              v
[ Query Engine (<5ms response) ]             [ Incremental Real-Time Sync ]
```

### Tier 1: Multithreaded Directory Traversal (`jwalk`)
- Standard single-threaded `find` or `walkdir` bottlenecks on sequential directory syscalls (`getdents64`).
- `jwalk` uses Rayon's work-stealing thread pool to parallelize directory traversal across all CPU cores.
- On modern NVMe SSDs, `jwalk` can traverse **1,000,000 files in under 1.5 seconds**.

### Tier 2: Compact Flat In-Memory Arena
To hold 2,000,000 files in under 80 MB of RAM:
- Do **not** allocate separate `String` or `PathBuf` for every file (which adds 24 bytes overhead + allocator metadata per string).
- Store directory paths in a deduplicated tree/pool of folder IDs (`u32`).
- Store file names in an interned string pool or flat byte buffer.
- Represent each file entry in a compact 32-byte struct:

```rust
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct FileRecord {
    pub dir_id: u32,        // Index into directory table
    pub name_offset: u32,   // Offset into string buffer
    pub name_len: u16,      // Length of filename
    pub flags: u16,         // Directory, Hidden, Symlink, Executable
    pub size_bytes: u64,    // File size in bytes
    pub mtime_secs: u64,    // Modification timestamp (epoch seconds)
}
```

### Tier 3: Search & Matching Engine
- **`nucleo-matcher`**: Helix/fzf-grade matching engine. Evaluates queries with high parallelism and early exit heuristics.
- **Tokenized Query Parser**:
  - `ext:rs` -> filters extension using fast integer/hash lookup.
  - `size:>100mb` -> filters `size_bytes > 104_857_600`.
  - `dm:today` -> filters `mtime_secs >= start_of_day`.
  - `path:/home/user/code` -> restricts search to specific directory prefix.
  - `regex:^test_.*\.json$` -> executes optimized Rust `regex` matcher.
  - Exact Substring: SIMD-accelerated byte-level substring matching (`memchr` / `aho-corasick`).

### Tier 4: Real-Time Monitoring (`notify` / `inotify`)
- Find-Me monitors configured index root directories (e.g., `/home/user`) using `notify` with the Linux `inotify` backend.
- Watches directory creations, deletions, renames, and modifications.
- Debounced event queue batches filesystem events every 200ms to prevent thrashing during high-volume I/O operations (e.g., `git checkout` or `cargo build`).
- Handles `IN_Q_OVERFLOW` safely by scheduling an incremental re-scan of the affected tree.

---

## 3. Instant Startup via Memory-Mapped Cache

On application launch, re-crawling millions of files is unnecessary if the cache is valid:
- On exit or index completion, write the flat arena to disk (`~/.cache/find-me/index.bin`).
- On startup, memory-map (`memmap2`) or deserialize using `rkyv` (zero-copy).
- Result: **Find-Me launches with 1,000,000+ files indexed in <50 milliseconds**, then runs a quiet background incremental check for changes that occurred while the application was closed.
