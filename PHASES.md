# Find-Me Implementation Phases & Task Specification

This document defines the complete engineering roadmap for **Find-Me**, an offline-first, high-performance Linux desktop file search application written in 100% pure Rust.

Each phase contains granular, sequential tasks. Each task specifies its branch name, objective, input/output requirements, acceptance criteria, and verification steps.

---

## Git Workflow & Branching Policy (Strict)

1. **`main` Branch Protection**:
   - `main` is the production branch.
   - **NO DIRECT CODE COMMITS ON `main`**. Only documentation, specifications, and merged feature PRs are permitted.
2. **Feature Branching Lifecycle**:
   - For every task:
     ```bash
     git checkout main
     git checkout -b feature/<task-id>-<task-name>
     ```
   - Complete implementation, tests, and formatting within the feature branch.
   - Verify code compiles without warnings (`cargo clippy -- -D warnings`) and passes tests (`cargo test`).
   - Merge into `main` using non-fast-forward merge:
     ```bash
     git checkout main
     git merge --no-ff feature/<task-id>-<task-name> -m "feat(<task-id>): <task description>"
     ```
3. **Permanent Branch Retention (Never Delete Feature Branches)**:
   - **DO NOT REMOVE OR DELETE BRANCHES** after merging (`git branch -d` / `git branch -D` must never be executed).
   - Every feature branch must be permanently retained in the Git repository so there is a complete, immutable historical audit trail of every task, commit, and code change completed throughout the project.

---

## Todoist Task Board Management Workflow

Project tasks are synchronized in real-time with the Todoist project **[Everything](https://app.todoist.com/app/project/everything-6j45wf6xPVCM2RJx)** (`id: 6j45wf6xPVCM2RJx`):

1. **`To Do` (`6j45x63GH3R3vWCQ`)**: Contains all scheduled phase tasks awaiting development. Each task documents its feature branch name, input/output requirements, and acceptance criteria.
2. **`In Progress` (`6j45x632fhrVMmvQ`)**: When a task begins and its feature branch (`git checkout -b feature/...`) is created, the task is moved into `In Progress`. Exactly one task should be active at any given time.
3. **`Paused` (`6j45x634PcW9V5wQ`)**: If work on a task is temporarily halted (e.g. awaiting user input, external package installations, or design clarifications), move the task to `Paused` with notes explaining the block.
4. **`Completed` (`6j45x63Fqh349Gxx`)**: When a task passes the automated QA gate (`verify_task.sh`), completes architectural review, and merges into `main` via `git merge --no-ff`, move it to `Completed`.
5. **`Future Ideas` (`6j45xVg6MVfrmpWQ`)**: Any enhancement ideas, additional packaging targets (Flatpak, AppImage, AUR), or post-v1 features suggested during development are tracked here.

---

## Phase Overview

```
+---------------------------------------------------------------------------------+
| Phase 0: Workspace Initialization & Cargo Scaffolding                           |
+---------------------------------------------------------------------------------+
                                      |
                                      v
+---------------------------------------------------------------------------------+
| Phase 1: Core Engine - Multithreaded Crawler & In-Memory Arena                  |
+---------------------------------------------------------------------------------+
                                      |
                                      v
+---------------------------------------------------------------------------------+
| Phase 2: Core Engine - Query Parser, Nucleo-Matcher & Filtering                 |
+---------------------------------------------------------------------------------+
                                      |
                                      v
+---------------------------------------------------------------------------------+
| Phase 3: Core Engine - Inotify File Watcher & Binary Persistence                |
+---------------------------------------------------------------------------------+
                                      |
                                      v
+---------------------------------------------------------------------------------+
| Phase 4: Desktop UI - Iced Shell, Sidebar Navigation & Theming                  |
+---------------------------------------------------------------------------------+
                                      |
                                      v
+---------------------------------------------------------------------------------+
| Phase 5: Desktop UI - Main Search Screen & High-Performance Virtual Table       |
+---------------------------------------------------------------------------------+
                                      |
                                      v
+---------------------------------------------------------------------------------+
| Phase 6: Desktop UI - Secondary Screens (Advanced, Index Mgmt, History, etc.)  |
+---------------------------------------------------------------------------------+
                                      |
                                      v
+---------------------------------------------------------------------------------+
| Phase 7: System Integration - Async Tokio Channels & Engine Live Streaming      |
+---------------------------------------------------------------------------------+
                                      |
                                      v
+---------------------------------------------------------------------------------+
| Phase 8: Desktop Shell Integration - Context Menus, XDG Actions & Keybindings   |
+---------------------------------------------------------------------------------+
                                      |
                                      v
+---------------------------------------------------------------------------------+
| Phase 9: Debian Packaging (.deb), Performance Benchmarks & QA Release           |
+---------------------------------------------------------------------------------+
```

---

## Detailed Task Breakdown

### Phase 0: Workspace Initialization & Cargo Scaffolding

#### Task 0.1: Initialize Cargo Workspace Structure
- **Branch**: `feature/p0-workspace-init`
- **Objective**: Establish a multi-crate Cargo workspace to cleanly decouple the core indexing engine from the Iced desktop GUI.
- **Workspace Layout**:
  - `crates/findme-core`: Core data structures, parallel crawler, query parser, matcher, inotify watcher.
  - `crates/findme-ui`: Pure Rust Iced desktop application, screens, widgets, themes.
  - `crates/findme-cli`: Lightweight CLI testing utility for benchmarking indexing and search without GUI overhead.
  - Root `Cargo.toml`: Workspace configuration, unified dependency management.
- **Acceptance Criteria**:
  - `cargo check --workspace` passes cleanly.
  - Clean separation of dependencies (`findme-core` has no GUI/Iced dependencies).

---

### Phase 1: Core Engine - Multithreaded Crawler & In-Memory Arena

#### Task 1.1: Compact In-Memory Arena & String Pool
- **Branch**: `feature/p1-memory-arena`
- **Objective**: Implement the compact 32-byte `FileRecord` struct, `DirectoryTable`, and deduplicated string arena.
- **Requirements**:
  - Minimal heap allocation overhead (<40 MB per 1,000,000 files).
  - Bitflags for file types: Directory, Regular File, Symlink, Hidden, Executable.
  - Reconstruction of full file path on demand from `dir_id` + `name_offset`.
- **Acceptance Criteria**:
  - Unit tests verify path reconstruction from nested directories.
  - Memory footprint benchmark confirms <45 bytes total per record including string heap.

#### Task 1.2: Parallel Directory Crawler (`jwalk` / Rayon)
- **Branch**: `feature/p1-parallel-crawler`
- **Objective**: Implement ultra-fast filesystem traversal across configured root paths.
- **Requirements**:
  - Uses `jwalk` with Rayon work-stealing thread pool to saturate NVMe/SSD read queues.
  - Honors directory exclusions (e.g., `/proc`, `/sys`, `/dev`, `.git`, or user exclusions).
  - Captures file size, modification timestamp (`mtime`), and metadata in single stat pass.
  - Emits real-time indexing progress batches (files scanned, folders scanned).
- **Acceptance Criteria**:
  - Benchmark traverses test directories at >200,000 files/sec on SSD.
  - Gracefully ignores permission denied (`EACCES`) directories without aborting traversal.

---

### Phase 2: Core Engine - Query Parser, Nucleo-Matcher & Filtering

#### Task 2.1: Search Query Tokenizer & Parser
- **Branch**: `feature/p2-query-parser`
- **Objective**: Implement a robust parser for search syntax supporting power-user filters.
- **Syntax Supported**:
  - Exact substring / fuzzy terms: `foo bar`
  - Extension filter: `ext:rs;toml`
  - Size operators: `size:>10mb`, `size:<500kb`, `size:100kb..5mb`
  - Modification date operators: `dm:today`, `dm:yesterday`, `dm:lastweek`, `dm:2026-11-20`
  - Directory restriction: `path:/home/user/projects`
  - Exclusion operator: `!node_modules`
  - Regex filter: `regex:^find_me_.*\.rs$`
- **Acceptance Criteria**:
  - Comprehensive unit test suite testing complex nested and chained query tokens.
  - Graceful handling of malformed queries without errors or panics.

#### Task 2.2: High-Performance Search Matching Engine
- **Branch**: `feature/p2-search-matcher`
- **Objective**: Integrate `nucleo-matcher` and fast SIMD substring scanning for sub-millisecond query evaluation.
- **Requirements**:
  - Two-stage filter pipeline:
    1. Fast predicate filter: size, date, extension, path prefix bitmasks.
    2. Name matcher: exact substring scan or `nucleo-matcher` Smith-Waterman scoring.
  - Early-exit limit handling (e.g., return top 1,000 results quickly for UI).
  - Multi-threaded parallel query evaluation via Rayon for datasets >100,000 items.
- **Acceptance Criteria**:
  - Query evaluation over 1,000,000 preloaded records completes in <10ms.

---

### Phase 3: Core Engine - Inotify File Watcher & Binary Persistence

#### Task 3.1: Real-Time Filesystem Monitoring (`notify` / `inotify`)
- **Branch**: `feature/p3-fs-watcher`
- **Objective**: Keep the in-memory index synchronized in real-time as files are created, renamed, modified, and deleted.
- **Requirements**:
  - Integrates `notify` crate targeting Linux `inotify` subsystem.
  - Debounces rapid bursts of filesystem events (e.g. `cargo build`, git checkout) into 150ms windows.
  - Mutates the in-memory arena in an atomic write pass.
  - Safely handles `IN_Q_OVERFLOW` by scheduling targeted incremental tree rescanning.
- **Acceptance Criteria**:
  - Creating, moving, or deleting a file in watched directory updates index within 200ms.
  - Unit/integration test verifies live index updates during continuous modifications.

#### Task 3.2: Persistent Index Storage & Instant Startup
- **Branch**: `feature/p3-index-persistence`
- **Objective**: Serialize in-memory index to disk (`$XDG_CACHE_HOME/find-me/index.bin`) for instant subsequent launches.
- **Requirements**:
  - High-speed binary serialization using `rkyv` or `bincode`.
  - Integrity check via header magic bytes and CRC32/Blake3 checksum.
  - Startup loading benchmark: Loads 1,000,000 files in <50ms.
- **Acceptance Criteria**:
  - Cold startup with cached index takes <50ms to reach ready state.
  - Corrupted cache files are detected and automatically trigger fresh re-indexing.

---

### Phase 4: Desktop UI - Iced Shell, Sidebar Navigation & Theming

#### Task 4.1: Iced Application Shell & Navigation
- **Branch**: `feature/p4-ui-shell`
- **Objective**: Scaffold the primary Iced application window, Elm-architecture state loop, and sidebar navigation.
- **Requirements**:
  - Modular screen routing: Search, Advanced Search, Index Management, Search History, Settings, About.
  - Sidebar with icon labels, active tab indicators, and clean layout.
  - Responsive window sizing (minimum window dimensions: 800x600, optimal: 1200x800).
- **Acceptance Criteria**:
  - Window launches smoothly, sidebar navigation transitions between placeholder screens cleanly.
  - Zero UI lag or stutter during navigation.

#### Task 4.2: Cohesive Dark & Light Theming
- **Branch**: `feature/p4-ui-theming`
- **Objective**: Implement clean, professional Dark and Light themes adhering to the design specification.
- **Requirements**:
  - Dark Theme: Deep charcoal surfaces, crisp readable typography, modern accent color.
  - Light Theme: Clean off-white surfaces, high-contrast dark text, subtle borders.
  - Eliminates excessive animations, heavy gradients, or blurry drop shadows.
  - Theme toggling persists in application state.
- **Acceptance Criteria**:
  - User can toggle themes dynamically; all widgets update instantly without graphical artifacts.

---

### Phase 5: Desktop UI - Main Search Screen & Virtual Table

#### Task 5.1: Main Search Input & Status Indicators
- **Branch**: `feature/p5-search-input`
- **Objective**: Build the prominent search bar component with real-time text input, clear button, and status indicators.
- **Requirements**:
  - Real-time debounced query emission as user types (0ms delay for exact match, 50ms for regex/heavy filters).
  - Status display showing: Index status ("Indexed 420,120 files"), Search latency ("0.8 ms"), Match count ("1,240 matches").
- **Acceptance Criteria**:
  - Search input automatically gains focus on startup.
  - Escape key clears input.

#### Task 5.2: Virtualized Tabular Results Component
- **Branch**: `feature/p5-virtual-table`
- **Objective**: Implement the high-performance virtualized table displaying Name, Path, Type, Size, and Date Modified.
- **Requirements**:
  - Uses `VirtualList` or `iced::widget::table` to render only the visible rows.
  - Column headers with sorting toggles (Ascending/Descending by Name, Size, Modified Date, Path).
  - Row selection highlighting with keyboard arrow navigation.
  - Handles mock datasets with 100,000+ items at steady 60 FPS scrolling.
- **Acceptance Criteria**:
  - Scrolling through 100,000 items is butter-smooth with zero frame drops or memory growth.

---

### Phase 6: Desktop UI - Secondary Screens

#### Task 6.1: Advanced Search Screen
- **Branch**: `feature/p6-advanced-screen`
- **Objective**: Implement the Advanced Search form with specialized filter controls.
- **Fields**:
  - Extension filter checkboxes & text input (e.g., Documents, Images, Audio, Code).
  - File size range picker (Min size, Max size with units: KB, MB, GB).
  - Date Modified date picker / range dropdown.
  - Directory inclusion/exclusion pickers with native folder browsing dialog.
  - Regular expression toggle and validation feedback.
- **Acceptance Criteria**:
  - Form interactions construct valid engine query strings and submit back to the Search screen.

#### Task 6.2: Index Management Screen
- **Branch**: `feature/p6-index-mgmt`
- **Objective**: Build the interface for managing indexed paths, exclusions, and index maintenance.
- **Controls**:
  - Indexed root folders list with "Add Folder" and "Remove Folder" buttons.
  - Excluded folders list.
  - Live statistics: Total files indexed, total folders, database size in MB, last updated time.
  - Actions: "Rebuild Index" (full scan), "Refresh Index" (incremental scan).
- **Acceptance Criteria**:
  - Adding/removing folders updates state immediately and dispatches crawler commands.

#### Task 6.3: Search History, Settings & About Screens
- **Branch**: `feature/p6-remaining-screens`
- **Objective**: Implement Search History, Settings, and About screens.
- **Features**:
  - **History**: Recent query list, one-click re-run, delete individual items, clear history, saved search bookmarks.
  - **Settings**: General preferences (run in background, auto-start), appearance (dark/light, font scale), search behavior (case sensitivity, regex default), privacy (exclude hidden files/folders).
  - **About**: Find-Me logo, version, license (MIT/Apache), project repository link.
- **Acceptance Criteria**:
  - All screens match the unified design system and handle empty/loading states gracefully.

---

### Phase 7: System Integration - Async Tokio Channels & Engine Streaming

#### Task 7.1: Asynchronous UI <-> Engine Bridge
- **Branch**: `feature/p7-engine-integration`
- **Objective**: Connect the real `findme-core` indexing engine with the `findme-ui` Iced interface using Tokio channels.
- **Requirements**:
  - Replace all mock data with live engine query evaluation.
  - Background crawler streams live progress events to the UI status bar without locking the UI thread.
  - Debounced search query triggers background matching worker and streams top results back to UI.
- **Acceptance Criteria**:
  - Full system indexing and live real-time typing search operates seamlessly with zero UI stutter.

---

### Phase 8: Desktop Shell Integration - Context Menus, XDG Actions & Keybindings

#### Task 8.1: File Context Menu & FreeDesktop Desktop Integration
- **Branch**: `feature/p8-desktop-integration`
- **Objective**: Enable full desktop environment interaction on right-clicking search results.
- **Actions**:
  - **Open**: Launch default system handler via `open::that_detached` / `xdg-open`.
  - **Open Containing Folder**: Highlight item in file manager (via DBus `FileManager1` or fallback).
  - **Copy Path**: Copy absolute file path to Wayland / X11 system clipboard.
  - **Copy File**: Copy file buffer to clipboard.
  - **File Properties**: Display file permissions, inode, exact byte size, and full paths.
- **Acceptance Criteria**:
  - Right-clicking any search result opens the context menu; clicking actions launches corresponding desktop apps cleanly.

#### Task 8.2: Comprehensive Keyboard Navigation
- **Branch**: `feature/p8-keyboard-nav`
- **Objective**: Implement complete keyboard-first shortcuts across the application.
- **Keybindings**:
  - `Ctrl + F` / `/`: Focus search input.
  - `Up` / `Down`: Navigate search result rows.
  - `Enter`: Open selected file.
  - `Alt + Enter`: Open enclosing directory.
  - `Ctrl + Shift + C`: Copy path.
  - `Escape`: Clear search or dismiss modals/menus.
  - `Ctrl + 1` .. `Ctrl + 6`: Switch navigation screens.
- **Acceptance Criteria**:
  - Application can be completely navigated and operated without a mouse.

---

### Phase 9: Debian Packaging (.deb), Performance Benchmarks & QA Release

#### Task 9.1: Debian Packaging with `cargo-deb`
- **Branch**: `feature/p9-debian-packaging`
- **Objective**: Package Find-Me into an installable `.deb` package targeting Ubuntu 22.04+ and Debian variants.
- **Artifacts Generated**:
  - `/usr/bin/find-me`
  - `/usr/share/applications/find-me.desktop`
  - `/usr/share/icons/hicolor/scalable/apps/find-me.svg`
  - Standard Debian control files and dependencies (`libxkbcommon0`, `libwayland-client0`).
- **Acceptance Criteria**:
  - `cargo deb` succeeds and outputs `target/debian/find-me_*.deb`.
  - Package installs cleanly via `dpkg -i` on Ubuntu without dependency errors.

#### Task 9.2: Performance Benchmarking & QA Verification
- **Branch**: `feature/p9-qa-benchmarks`
- **Objective**: Validate performance against targets on large real-world filesystem trees.
- **Target Metrics**:
  - Initial crawl speed: >200,000 files/second on SSD.
  - Memory usage: <50 MB for 1,000,000 indexed files.
  - Query latency: <5 ms for substring/exact queries on 1M files.
  - UI Framerate: Steady 60 FPS during fast scrolling and typing.
- **Acceptance Criteria**:
  - Benchmark suite validates all target metrics are satisfied with zero test failures.
