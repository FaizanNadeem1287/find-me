# Find-Me Documentation Master Index

Welcome to the official documentation for **Find-Me**, an ultra-fast, offline-first Linux desktop file search engine and semantic neural retrieval system inspired by Windows *Everything*, written in 100% pure Rust with an Iced GUI.

---

## 📚 Documentation Table of Contents

| Guide | File | Summary |
| :--- | :--- | :--- |
| **Getting Started** | [GETTING_STARTED.md](./GETTING_STARTED.md) | Installation, first launch, system requirements, and quick-start workflow. |
| **Architecture & Systems** | [ARCHITECTURE.md](./ARCHITECTURE.md) | 4-crate workspace design, 32-byte arena memory layout, parallel crawler, inotify engine, and zero-copy binary serialization. |
| **Search Syntax & Query Language** | [SEARCH_SYNTAX.md](./SEARCH_SYNTAX.md) | Complete reference for Everything operators, size filters, date math, wildcards, regex, and boolean logic. |
| **Semantic Vector Search & AI** | [SEMANTIC_SEARCH_AND_AI.md](./SEMANTIC_SEARCH_AND_AI.md) | Deep content extraction (PDF, DOCX, XLSX, PPTX, Code, Audio, Images), 384-dim local embeddings, HNSW index, BM25, and Hybrid RRF fusion. |
| **Keyboard Navigation & Shortcuts** | [KEYBOARD_SHORTCUTS.md](./KEYBOARD_SHORTCUTS.md) | Comprehensive keyboard-first navigation map, screen switching, results navigation, and clipboard operations. |
| **CLI & Headless Automation** | [CLI_AND_HEADLESS.md](./CLI_AND_HEADLESS.md) | Command-line search mode, JSON streaming, batch exporters (CSV, TXT, EFU), and GUI query pre-filling. |
| **Packaging & Universal Linux** | [PACKAGING_AND_DISTRIBUTION.md](./PACKAGING_AND_DISTRIBUTION.md) | Debian `.deb`, AppImage, Flatpak, Arch Linux AUR, FreeDesktop XDG specifications, and desktop integration. |
| **Developer & Contributor Guide** | [DEVELOPER_GUIDE.md](./DEVELOPER_GUIDE.md) | Toolchain setup, build commands, testing pyramid, Criterion benchmarks, and Git branch retention rules. |
| **Troubleshooting & FAQ** | [TROUBLESHOOTING_AND_FAQ.md](./TROUBLESHOOTING_AND_FAQ.md) | Common diagnostics: Linux inotify limits, permission issues, index rebuilds, and power throttling. |

---

## 🎯 Key Architectural Highlights

* **100% Pure Rust Stack**: Zero web technologies, zero Electron, zero WebViews, zero JavaScript/CSS runtimes. Built natively with [Iced](https://github.com/iced-rs/iced) on Linux Wayland/X11.
* **Instantaneous Search**: Sub-millisecond (< 2 ms) query evaluation across 100,000+ files via SIMD-accelerated substring and Smith-Waterman fuzzy matching (`nucleo-matcher`).
* **Minimal Memory Footprint**: Compact 32-byte `FileRecord` arena with path deduplication and interned string pooling (`~15 MB` RAM for 200,000 indexed records).
* **Live Inotify Filesystem Sync**: Real-time event monitor with adaptive 150ms debouncing and queue overflow auto-recovery.
* **Offline Local Multimodal Semantic AI**: Deep content indexing of PDF, Word, PowerPoint, Spreadsheets, Code, Audio headers, and Image perceptual hashing with zero cloud dependencies or external API keys.
* **Universal Linux Packaging**: Native Debian packages (`.deb`), standalone AppImages, sandbox-hardened Flatpaks, and Arch Linux AUR PKGBUILDs.
