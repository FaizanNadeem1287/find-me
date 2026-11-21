---
name: find-me-rust-dev
description: >-
  Comprehensive guide and reference for developing the Find-Me desktop search application
  in pure Rust for Linux (Debian/Ubuntu). Use whenever working on the Find-Me codebase,
  including Rust systems programming, Iced GUI architecture, high-performance in-memory
  indexing, real-time filesystem monitoring with notify/inotify, query parsing, and Debian packaging.
---

# Find-Me Rust Development Skill

This skill provides the core reference knowledge, design patterns, architecture standards, and procedural workflows for developing **Find-Me**, a high-performance, offline-first Linux file search application inspired by Windows *Everything*, written in 100% pure Rust.

## Core Directives & Standards

1. **Pure Rust Stack**: No web technologies (no Electron, no Tauri, no WebViews, no JS/HTML). All UI is implemented using **Iced**. All engine code is pure native Rust.
2. **Zero Workarounds**: Every component must be built with production-grade, long-term engineering. No brittle shell hacks, no hardcoded `/proc` polling loops, no unsafe memory tampering, and no requiring root/sudo privileges for basic desktop operations.
3. **Decoupled Architecture**: The desktop UI must remain strictly separated from the core indexing and search engine. Communication occurs via asynchronous message passing (Tokio MPSC/Broadcast channels).
4. **Git Branching & History Retention Policy**:
   - The `main` branch is pristine and production-stable. **NEVER** write or commit code directly to `main`.
   - All tasks must be implemented in a dedicated branch branched from `main` (e.g., `feature/p0-workspace-init`, `feature/p1-memory-arena`).
   - Merge back into `main` using non-fast-forward merge (`git merge --no-ff`).
   - **PERMANENT BRANCH RETENTION**: **Never delete feature branches** after merging. Keep all branches preserved permanently to maintain a complete historical record of all completed tasks.

---

## Technical Reference Links

Consult the specialized references in this skill before querying external resources:

- [Rust Basics, Concurrency & Ecosystem](./references/rust_basics_and_ecosystem.md)
  * Rust memory model, ownership, error handling with `thiserror`/`anyhow`
  * Tokio async runtime, channels, thread pools, and SIMD string scanning
- [Iced Desktop GUI Architecture Guide](./references/iced_gui_guide.md)
  * Elm architecture: `State`, `Message`, `view()`, `update()`, `subscription()`
  * Virtualized table/list rendering for 100,000+ items with zero lag
  * Light and Dark theme styling, keyboard-first navigation, and context menus
- [Indexing & Search Engine Engineering](./references/indexing_and_search_engine.md)
  * Linux filesystem traversal using parallel `jwalk` (Rayon)
  * In-memory flat arena & string interning for compact memory footprint (<100MB for 1M files)
  * Sub-millisecond matching via `nucleo-matcher` (Smith-Waterman dual matrix) & regex
  * Real-time change tracking with Linux `inotify` via `notify` crate
  * High-speed zero-copy disk persistence with `rkyv` / `bincode`
- [Linux Desktop Integration & Debian Packaging](./references/linux_desktop_and_packaging.md)
  * FreeDesktop standards: `.desktop` entry, XDG base directories, app icons
  * Native file operations: opening files, opening enclosing folder in file manager (`xdg-open` / `dbus`)
  * Debian `.deb` packaging with `cargo-deb` for Ubuntu 22.04+ and Debian variants

---

## Standard Development Workflow

### 1. Starting a New Task
```bash
# Ensure main is up to date
git checkout main
git pull origin main  # if remote configured

# Create a dedicated feature branch
git checkout -b feature/<task-name>
```

### 2. Validating Code Quality
```bash
# Code formatting check
cargo fmt -- --check

# Strict linting (warnings as errors in CI)
cargo clippy --all-targets --all-features -- -D warnings

# Unit & integration testing
cargo test --all

# Release build validation
cargo build --release
```

### 3. Merging Work & Preserving Branch History
```bash
# Switch to main and merge with non-fast-forward merge
git checkout main
git merge --no-ff feature/<task-name> -m "feat: complete <task-name>"

# IMPORTANT: DO NOT delete the feature branch!
# Keep feature/<task-name> intact for complete task audit trail.
```
