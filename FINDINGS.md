# Find-Me Technical Findings, Research & Architecture Decisions

This document consolidates the technical research, architectural evaluations, library benchmarks, and engineering decisions for **Find-Me**, a pure-Rust, high-performance Linux desktop search engine inspired by Windows *Everything*.

---

## 1. Deep Dive: Windows "Everything" vs. Linux System Architecture

### 1.1 How Windows "Everything" Operates
*Everything* (by Voidtools) is renowned for indexing millions of files in 1–2 seconds and returning search results within milliseconds as the user types. It accomplishes this through Windows-specific mechanisms:
1. **Direct NTFS MFT (Master File Table) Reading**:
   - Instead of recursively traversing directories using Win32 APIs (`FindFirstFileW` / `FindNextFileW`), *Everything* requests volume handles (`\\.\C:`) and queries the Master File Table directly using `FSCTL_GET_NTFS_VOLUME_DATA`.
   - The MFT is a contiguous on-disk database where each file and directory record is exactly 1,024 bytes.
   - Sequential read speeds on modern SSDs (1–7 GB/s) allow reading the entire MFT into memory in seconds.
2. **USN Change Journal (Update Sequence Number)**:
   - NTFS maintains an internal journal logging every creation, deletion, renaming, and metadata change.
   - *Everything* reads this stream to keep its in-memory database synchronized without running directory scans.
3. **In-Memory Flat Table**:
   - All indexed records are stored in a contiguous in-memory structure with custom string heaps, enabling sub-millisecond query filtering.

### 1.2 Why Direct Block Device Parsing Cannot Be Used on Linux
A naive attempt to replicate *Everything* by opening raw disk devices (e.g., `/dev/nvme0n1p2`, `/dev/sda1`) and parsing filesystem structures directly fails in real-world Linux environments for fundamental reasons:
1. **Permission & Security Hazards**:
   - Reading `/dev/sd*` or `/dev/nvme*` requires `root` privileges (`sudo` or `CAP_SYS_RAWIO`).
   - Running an unprivileged user desktop search utility as root violates standard Linux security practices and packaging policies for Debian/Ubuntu.
2. **Filesystem Heterogeneity**:
   - Linux systems rarely run on a single filesystem format. A standard Linux setup frequently mounts:
     - `ext4` (traditional inode tables, block groups)
     - `btrfs` (B-trees, subvolumes, copy-on-write trees)
     - `xfs` (allocation groups, B+ trees)
     - `zfs` (merkle trees, storage pools)
     - `tmpfs`, `devtmpfs`, `overlayfs` (Docker/Flatpak)
     - Network filesystems (`nfs`, `cifs/smb`)
   - Attempting to implement kernel-level filesystem parsers for all formats inside a user-space application would introduce enormous complexity and severe vulnerability risks.
3. **Storage Abstraction & Encryption**:
   - Modern Linux distributions use LUKS (dm-crypt), LVM (Logical Volume Manager), and software RAID (mdadm). Inode tables on raw devices are encrypted ciphertext; only the kernel's decrypted block device layer can read them.

### 1.3 The Production-Grade Linux Solution (Zero Workarounds)
To deliver *Everything*-level speed without security compromises or fragile hacks, Find-Me utilizes a 4-tier native Linux architecture:

| Tier | Component | Implementation | Performance Metric |
| :--- | :--- | :--- | :--- |
| **Tier 1: Traversal** | Multithreaded Parallel Crawler | `jwalk` (Rayon work-stealing pool over directory fds) | Traverses 1,000,000 files in **~0.8 – 1.4 seconds** on NVMe SSDs |
| **Tier 2: In-Memory Storage** | Flat Arena + String Interner | Deduplicated directory tree + interned filenames | Consumes **<40 MB RAM per 1,000,000 files** |
| **Tier 3: Query Engine** | Fuzzy, Substring & Regex Matcher | `nucleo-matcher` (dual-matrix Smith-Waterman) + SIMD scan | Evaluates queries across 1M files in **<5 milliseconds** |
| **Tier 4: Live Updates** | Kernel Filesystem Watcher | `notify` (Linux `inotify` backend) with debounced queue | Real-time updates with **<200ms latency** |

---

## 2. GUI Framework Evaluation: Why Iced is Selected

The user specification mandates:
- **100% Pure Rust**: No JavaScript, TypeScript, HTML, CSS, Electron, Tauri, or WebViews.
- **Desktop UI strictly separated from indexing engine**.
- **Reusable components & modular screens**.
- **Dark & Light theme support with keyboard-first navigation**.

### 2.1 Framework Comparison Matrix

| Framework | Architecture | Strengths | Limitations | Verdict for Find-Me |
| :--- | :--- | :--- | :--- | :--- |
| **Iced** | **Elm (Model-View-Update)** | • 100% Pure Rust<br>• Asynchronous command/subscription loop built-in (`tokio`)<br>• Extremely low memory (~25-40 MB)<br>• Native virtualization (`VirtualList`, `table`)<br>• Strong ecosystem stability | • Custom widgets require understanding Elm architecture | **SELECTED (Optimal)**: Strictly aligns with specification, clean decoupling, excellent virtualization. |
| **Slint** | Declarative DSL + Rust | • High performance<br>• Native style backends<br>• Good visual tooling | • Requires proprietary/custom `.slint` DSL files (not pure Rust code)<br>• Dual-licensing considerations | **Rejected**: Requires non-Rust DSL files. |
| **Relm4 / gtk4-rs** | GTK4 + Component Model | • Native GNOME look and feel<br>• System accessibility | • Heavy dynamic C library dependencies (`libgtk-4-dev`)<br>• Packaging complexity across non-GNOME desktops<br>• Slower build times | **Rejected**: Heavy C runtime dependencies and less portable outside GNOME. |
| **egui** | Immediate Mode | • Simple single-file prototyping<br>• Highly customizable rendering | • High CPU/GPU usage (redraws every frame)<br>• Poor native desktop text rendering and clipboard integration<br>• Not ideal for complex tabular applications | **Rejected**: Immediate mode GUI is ill-suited for power-saving desktop search utilities. |

### 2.2 Iced Architecture Highlights for Find-Me
1. **Asynchronous Engine Coupling**:
   - Iced supports asynchronous commands and subscriptions natively. Tokio background workers emit `EngineEvent` messages directly into Iced's event loop via MPSC channels without polling.
2. **Virtualized Tabular Rendering**:
   - `iced::widget::table` and `VirtualList` render only the visible 30–50 rows intersecting the viewport, enabling seamless 60 FPS scrolling through 500,000+ search results.
3. **Decoupled Module Structure**:
   - Screens (`Search`, `Advanced`, `IndexManagement`, `History`, `Settings`, `About`) are completely isolated modules that communicate through typed messages.

---

## 3. Core Engine Architecture & Data Structures

### 3.1 Memory Layout for Million-File Scale
Storing millions of `String` or `PathBuf` objects in memory causes severe heap fragmentation:
- A standard `String` or `PathBuf` incurs a 24-byte struct overhead plus allocator heap chunk padding.
- 1,000,000 strings with average 40 characters consume ~80 MB just in struct overhead and heap waste.

#### Find-Me Compact In-Memory Schema:
```rust
/// Compact representation of a file or directory entry (32 bytes)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct FileRecord {
    pub parent_dir_id: u32,  // Pointer to DirectoryEntry
    pub name_offset: u32,    // Byte offset into global string arena
    pub name_len: u16,       // Length of filename in bytes
    pub flags: u16,          // Bitflags: IS_DIR, IS_SYMLINK, IS_HIDDEN, IS_EXEC
    pub size_bytes: u64,     // File size in bytes (0 for directories)
    pub mtime_secs: u64,     // Modified timestamp in epoch seconds
}

/// Directory path table to deduplicate deep paths
pub struct DirectoryTable {
    pub directories: Vec<DirectoryEntry>,
}

pub struct DirectoryEntry {
    pub parent_id: u32,      // Index of parent directory (0 for root)
    pub name_offset: u32,    // Offset into string buffer
    pub name_len: u16,
}
```

### 3.2 Query Syntax Parser & Matcher
Find-Me implements a recursive-descent query tokenizer and parser supporting standard power-user search operators:
- `ext:rs;toml` — Filters by file extension.
- `size:>10mb` or `size:<500kb` — Size range comparisons.
- `dm:today` or `dm:lastweek` — Modified date constraints.
- `path:/home/user/projects` — Path prefix scoping.
- `regex:^find_me_[a-z0-9]+\.rs$` — Regular expression evaluation using linear-time `regex::Regex`.
- Fuzzy Substring — Evaluated using `nucleo-matcher` with Smith-Waterman scoring for instant typographical tolerance.

### 3.3 Linux Filesystem Event Monitoring (`inotify`)
- **Watch Strategy**:
  - Find-Me monitors user-configured indexed directories using `notify` with inotify backend.
  - Linux systems enforce `fs.inotify.max_user_watches` (typically 524,288). To prevent exhaustion on deeply nested trees, Find-Me prioritizes user home and data directories.
  - An event debouncer batches events in 150ms windows. When a batch completes, changes are applied to the in-memory arena in a single atomic transaction.
- **Handling `IN_Q_OVERFLOW`**:
  - If a massive bulk operation (e.g. `rm -rf node_modules` or `tar -xzf`) triggers an inotify queue overflow, the engine safely flags the affected root and performs a fast incremental rescan, guaranteeing data consistency.

### 3.4 Instant Startup via Persistent Cache
- On shutdown or index completion, Find-Me serializes the `DirectoryTable`, string arena, and `Vec<FileRecord>` to `$XDG_CACHE_HOME/find-me/index.bin`.
- The format utilizes zero-copy binary layout (`rkyv` / memory-mapped file via `memmap2`).
- On subsequent launches, the index loads in **<50 milliseconds**, providing instant search readiness while a lightweight background task reconciles any filesystem modifications made while Find-Me was closed.

---

## 4. Linux Packaging & Distribution Strategy

### Primary Target: Debian / Ubuntu (`.deb`)
- Built using `cargo-deb`, which creates compliant Debian binary packages natively without needing complex Debian packaging build environments.
- Target versions: Ubuntu 22.04 LTS, Ubuntu 24.04 LTS, Ubuntu 26.04 LTS, and Debian 12+.
- Assets installed by `.deb`:
  - Binary: `/usr/bin/find-me`
  - Desktop Entry: `/usr/share/applications/find-me.desktop`
  - Application Icons: `/usr/share/icons/hicolor/{scalable,48x48,128x128,256x256}/apps/find-me.svg`
  - FreeDesktop AppStream Metadata: `/usr/share/metainfo/io.github.findme.metainfo.xml`
- Future Package Formats:
  - AppImage (standalone single-file bundle)
  - Flatpak (sandboxed desktop distribution via Flathub)
  - Arch Linux (`PKGBUILD` for AUR)

---

## 5. Architectural Non-Negotiables (Zero Workarounds Policy)

1. **No Polling Loops**: Never run `sleep()` loops checking for filesystem updates. Use kernel event notifications (`inotify`) and channel subscriptions.
2. **No Elevated Privileges**: The application must run 100% in user-space without `sudo`, setuid binaries, or root daemon requirements.
3. **No UI Freezes**: All filesystem traversal, searching, sorting, and serialization must execute on dedicated Tokio/Rayon thread pools. The Iced UI thread must remain strictly at 60 FPS under all workloads.
4. **No Shell Invocations for Core Operations**: Path resolution, file metadata queries, and filtering must use standard Rust POSIX APIs (`std::fs`, `nix`, `libc`), never shelling out to `find`, `ls`, or `grep`.
5. **No Blind Unwraps**: All potential I/O, parsing, or filesystem errors must be explicitly mapped to strongly-typed error enums (`thiserror`) and gracefully communicated to the user.

---

## 6. Comprehensive Testing, Debugging & QA Architecture

To guarantee that Find-Me is rock-solid and stable for thousands of users, the project implements a rigorous testing pyramid across every layer:

```
                  / \
                 /   \       Manual Smoke & Release Packaging (.deb)
                /     \      (Installed binary, desktop entry, Wayland/X11)
               /       \
              /---------\    Performance & Benchmarking (criterion)
             /           \   (Throughput >200k files/s, query latency <5ms)
            /-------------\  Headless UI Testing (iced_test Simulator)
           /               \ (Deterministic message/state loops, no window)
          /-----------------\ Integration Testing (tempfile, notify, jwalk)
         /                   \ (Real directory trees, live inotify events)
        /---------------------\ Property-Based Fuzzing (proptest)
       /                       \ (Fuzzing query parser with millions of inputs)
      /-------------------------\ Unit Testing (cargo test --lib)
     /                           \ (Data structures, compact arena, tokenizers)
```

### 6.1 Testing Disciplines Explained for Non-QA Developers

1. **Unit Testing (`cargo test --lib`)**:
   - Tests individual functions in total isolation.
   - Example: Testing that a file size string like `"15.4 MB"` parses into exact bytes, or that `FileRecord` bitflags correctly identify hidden files.

2. **Property-Based Fuzz Testing (`proptest`)**:
   - Instead of testing hand-crafted strings, `proptest` generates thousands of bizarre, random, and edge-case inputs (weird Unicode, control characters, null bytes, huge numbers) to prove that the parser never crashes or panics under any input.

3. **Integration Testing (`tests/` with `tempfile`)**:
   - Creates real directories and files in temporary isolated folders, tests multithreaded crawling with `jwalk`, and verifies that creating or deleting files triggers real-time `inotify` updates within 200ms.

4. **Headless UI Testing (`iced_test::simulator`)**:
   - Tests user interface interactions (typing into search box, clicking sidebar navigation, pressing Escape, sorting table columns) **without needing a physical monitor, GPU, or X11/Wayland display server**.
   - Because Iced uses the Elm architecture, UI state transitions are pure functions that can be tested in automated CLI environments.

5. **Performance & Regression Benchmarking (`criterion`)**:
   - Mathematically measures execution time, preventing slow code from slipping in. Enforces crawl speeds >200k files/sec and search latencies <5ms.

6. **Flaky Test Prevention**:
   - Eliminates arbitrary `thread::sleep()` delays. All tests use Tokio asynchronous channels with explicit bounded timeouts (`tokio::time::timeout`).

7. **Pre-Merge Verification Gate (`verify_task.sh`)**:
   - An automated script that enforces formatting (`cargo fmt`), strict zero-warning linting (`cargo clippy -- -D warnings`), all unit/integration tests (`cargo test`), and documentation tests before any branch can be merged into `main`.

---

## 7. Autonomous Engineering Roster (10 Subagents & 9 Skills)

To enable 90% autonomous agentic execution with uncompromising software quality, Find-Me operates with 10 specialized subagents and 9 domain skills:

### 7.1 The 10 Specialized Subagents
| # | Subagent Identifier | Domain & Responsibility |
| :-: | :--- | :--- |
| 1 | **`findme-core-engine`** | Multithreaded parallel directory crawler (`jwalk`), compact 32-byte in-memory arena, `nucleo-matcher`, and `inotify` watcher. |
| 2 | **`findme-ui-developer`** | Pure Rust **Iced** GUI, Elm architecture, virtualized tables (100k+ rows at 60 FPS), themes, and keyboard navigation. |
| 3 | **`findme-qa-tester`** | Automated unit tests, property-based fuzzing (`proptest`), and headless UI test suites (`iced_test`). |
| 4 | **`findme-performance-profiler`** | Throughput (>200k files/s) and latency (<5ms) profiling, Criterion benchmarks, and memory leak audits. |
| 5 | **`findme-code-reviewer`** | Senior Architect auditing code for memory safety, zero workarounds, zero unwraps, and non-blocking UI. |
| 6 | **`findme-packager-qa`** | Debian `.deb` packaging (`cargo-deb`), FreeDesktop standards, and system installation checks. |
| 7 | **`findme-desktop-integrator`** | FreeDesktop standards, DBus file highlighting (`FileManager1`), Wayland/X11 clipboard, and OS theme sync. |
| 8 | **`findme-query-architect`** | Query language lexing, parsing, boolean expression trees, and SIMD predicate evaluation. |
| 9 | **`findme-researcher`** | Deep technical internet research (Stack Overflow, Reddit `r/rust`, GitHub, Linux man pages) with zero-workaround vetting. |
| 10 | **`findme-todoist-manager`** | Todoist task and board lifecycle management, moving cards across sections, and logging progress. |

### 7.2 The 9 Specialized Skills
1. **`find-me-rust-dev`**: Pure Rust systems programming, Iced Elm architecture, and packaging.
2. **`find-me-qa-verification`**: 6-tier testing pyramid, headless UI testing, and pre-merge quality gate.
3. **`find-me-autonomous-dev`**: 8-step autonomous execution cycle ("vibe coding"), self-healing compiler guide.
4. **`todoist-task-workflow`**: Todoist MCP runbook, project & section IDs, lifecycle transitions.
5. **`find-me-git-workflow`**: Branch lifecycles, Conventional Commits, non-fast-forward merges, permanent branch retention.
6. **`linux-desktop-integration`**: DBus `FileManager1` file highlighting, OS theme auto-detection, Wayland/X11 clipboard, and desktop entries.
7. **`query-syntax-and-parser`**: Full Everything-style query grammar (EBNF), tokenization, size units, date math, and AST predicate compilation.
8. **`linux-filesystem-internals`**: POSIX metadata extraction, symlink cycle prevention, virtual filesystem exclusion, and kernel inotify overflow recovery.
9. **`technical-internet-research`**: High-precision research methodology across Stack Overflow, Reddit, GitHub, Linux man pages, and Google.
