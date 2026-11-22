# Find-Me — Instant Linux Desktop Search & Offline Semantic AI Engine

[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg?logo=rust)](https://www.rust-lang.org)
[![GUI Framework](https://img.shields.io/badge/GUI-Iced%20(Elm)-blue.svg)](https://github.com/iced-rs/iced)
[![Platform](https://img.shields.io/badge/platform-Linux%20(Wayland%2FX11)-green.svg)](https://www.kernel.org)
[![Release](https://img.shields.io/badge/release-v0.1.0-blueviolet.svg)](https://github.com/FaizanNadeem1287/find-me/releases/tag/v0.1.0)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Zero Web Tech](https://img.shields.io/badge/web%20tech-0%25%20(No%20Electron%2FNo%20JS)-success.svg)](#)

> **Find-Me** is an offline-first, high-performance Linux desktop file search utility inspired by Windows *Everything*, written in **100% pure Rust** with a GPU-accelerated **Iced** GUI.

Find-Me indexes millions of filesystem entries into a compact, in-memory 32-byte arena, synchronizes changes in real time via Linux `inotify`, and evaluates complex search queries in **under 2 milliseconds**. In addition, Find-Me integrates an **on-device neural vector search engine** for deep document content extraction, offline semantic retrieval, and multimodal discovery with zero cloud dependencies.

---

## 🚀 Key Highlights & Benchmarks

| Metric / Capability | Find-Me Performance | Industry / Web Comparison |
| :--- | :--- | :--- |
| **Query Latency** | **< 2 ms** (exact substring / fuzzy / AST) | 200–800 ms (Electron / WebViews) |
| **Index Throughput** | **> 200,000 files/sec** (`jwalk` + Rayon) | 10,000–30,000 files/sec |
| **RAM Footprint** | **~40 MB per 1,000,000 files** (32-byte arena) | 500 MB – 1.5 GB (Chromium/Node) |
| **Startup / Cold Load**| **< 35 ms** for 1M records (zero-copy binary) | 2–5 seconds |
| **Filesystem Sync** | **Real-Time Linux inotify** (150ms debounce) | Polling / Periodic indexing |
| **UI Rendering** | **Native Iced GUI** (locked at 60 FPS) | Web DOM reflows / lag |
| **Privilege Model** | **100% Unprivileged User Space** | No root / no sudo required |

---

## 📚 Official Documentation

Find-Me includes a complete, 10-part documentation suite in the [`docs/`](./docs/) directory:

* 🚀 [**Getting Started Guide**](./docs/GETTING_STARTED.md): System requirements, quick-start workflow, and user installation.
* 🏛️ [**Architecture & System Internals**](./docs/ARCHITECTURE.md): Multi-crate design, 32-byte arena layout, parallel crawler, inotify engine, and binary format.
* 🔍 [**Search Syntax & Query Language**](./docs/SEARCH_SYNTAX.md): Complete Everything-style query syntax, size filters, date math, wildcards, and boolean logic.
* 🧠 [**Semantic Search & Local AI**](./docs/SEMANTIC_SEARCH_AND_AI.md): Deep content extraction (PDF, Office, Code, Audio, Images), 384-dim embeddings, HNSW vector store, and BM25/RRF hybrid fusion.
* ⌨️ [**Keyboard Navigation Map**](./docs/KEYBOARD_SHORTCUTS.md): Keyboard-first shortcuts, screen switching, row navigation, and clipboard operations.
* 💻 [**CLI & Headless Automation**](./docs/CLI_AND_HEADLESS.md): Headless terminal search, JSON streaming, batch exporters (CSV, TXT, EFU), and GUI query pre-filling.
* 📦 [**Packaging & Distribution**](./docs/PACKAGING_AND_DISTRIBUTION.md): Debian packages, AppImage, Flatpak, Arch Linux AUR, and FreeDesktop XDG specifications.
* 🛠️ [**Developer & Contributor Guide**](./docs/DEVELOPER_GUIDE.md): Toolchain setup, build commands, testing pyramid, Criterion benchmarks, and Git branch retention rules.
* ❓ [**Troubleshooting & FAQ**](./docs/TROUBLESHOOTING_AND_FAQ.md): Inotify limits, permission issues, index rebuilds, and power throttling.

---

## 📦 Installation & Quick Start

### 1. Debian / Ubuntu (`.deb`)
Download the `.deb` release package from [GitHub Releases](https://github.com/FaizanNadeem1287/find-me/releases):
```bash
sudo dpkg -i find-me_0.1.0-1_amd64.deb || sudo apt-get install -f -y
```

### 2. Universal Portable AppImage
Run anywhere without installation on any Linux distribution:
```bash
chmod +x find-me-0.1.0-x86_64.AppImage
./find-me-0.1.0-x86_64.AppImage
```

### 3. Build from Source
```bash
# Clone the repository
git clone https://github.com/FaizanNadeem1287/find-me.git
cd find-me

# Install system build dependencies (Ubuntu/Debian)
sudo apt update && sudo apt install -y build-essential pkg-config libxkbcommon-dev libwayland-dev

# Compile optimized release binary
cargo build --release -p findme-cli

# Run Find-Me
./target/release/find-me
```

---

## 🔮 Future Works & Engineering Roadmap

Find-Me's architecture is built to continuously scale across Linux desktop environments and local AI workflows. The roadmap below outlines scheduled and planned milestones:

### 1. Near-Term Milestones (v0.2.0 — Desktop Shell & System Ergonomics)

* **Floating Spotlight / Raycast-Style Quick Launcher**:
  - Global background daemon (`find-me --daemon`) with a configurable global hotkey (`Super + Space` or `Ctrl + Space`) using Wayland Layer Shell (`wlr-layer-shell`) and X11 floating hints.
  - Centered minimalist search modal that automatically dismisses on file activation or `Escape`.
* **System Tray & Status Notifier Item (SNI)**:
  - Minimize-to-tray support via FreeDesktop StatusNotifierItem (AppIndicator).
  - Quick-access menu displaying live indexing status, RAM consumption, and 1-click "Pause/Resume Indexing" and "Rebuild Index".
* **Desktop Search Provider Integration**:
  - Implement the `org.gnome.Shell.SearchProvider2` DBus interface for GNOME Shell and KDE KRunner integration, routing native OS searches directly into Find-Me's in-memory arena in sub-5ms.
* **File Manager Context Menu Plugins**:
  - Native extensions for Nautilus (`nautilus-python`), Dolphin (KDE service menus), and Thunar custom actions allowing users to right-click any folder and select *"Search with Find-Me here..."*.

---

### 2. Medium-Term Milestones (v0.3.0 – v0.5.0 — Core Engine & AI Vector Evolutions)

* **OCR for Scanned Documents & Images**:
  - Local, unprivileged Optical Character Recognition to index text from scanned invoices, receipts, screenshots, and diagrams into the semantic chunk store.
* **Transparent Archive Content Search**:
  - Deep inspection and indexing of compressed containers (`.zip`, `.tar.gz`, `.tar.xz`, `.7z`) without decompressing them to disk.
* **Hardware Acceleration for Semantic Vectors (wgpu / Vulkan / OpenVINO)**:
  - Optional GPU/NPU acceleration for vector chunk embeddings using `wgpu` or Vulkan shaders, accelerating batch initial indexing of large multi-gigabyte document libraries without CUDA or heavy runtimes.
* **Duplicate File Finder & Storage Analyzer**:
  - Fast hash-based duplicate detection (multi-stage bucketing: size filter → 4KB header hash → full BLAKE3 hash) to find identical files and save disk space.
* **Interactive Media Previews & Audio Waveform Player**:
  - Inline audio snippet playback with highlighted timestamps for Whisper-transcribed audio segments.
  - Quick image lightbox preview modal directly inside the results view.

---

### 3. Long-Term Architectural Vision (v1.0.0+ — Local Intelligence & Sync)

* **Offline Conversational Document Q&A ("Ask Your Files" / RAG)**:
  - Combine Find-Me's Hybrid BM25 + HNSW vector retrieval engine with an ultra-lightweight on-device quantized Small Language Model (SLM, e.g. SmolLM2, Qwen2.5-0.5B, or Phi-3.5 via `candle`) for 100% private offline question answering and document summaries.
* **Local Network Remote Search (Optional Web/LAN Server)**:
  - Optional, PIN-protected embedded HTTP/WebSocket daemon (inspired by Windows Everything's HTTP server) enabling users to search and download files on their Linux workstation from their phone or laptop on the local network.
* **FreeDesktop Extended File Tagging (`user.xdg.tags`)**:
  - Tag files with visual color badges and custom keywords saved to filesystem extended attributes (`xattr`), synchronized with KDE and GNOME file managers.

---

### 4. Distribution & Packaging Roadmap

| Target | Channel / Platform | Status / Next Step |
| :--- | :--- | :--- |
| **Flathub** | Flathub Official Repo | Submit the verified Flatpak manifest ([`packaging/flatpak/`](./packaging/flatpak/)) for 1-click install on Fedora, SteamOS (Steam Deck), and Linux Mint |
| **Arch Linux AUR** | Arch User Repository | Publish `find-me` package using the verified [`packaging/aur/PKGBUILD`](./packaging/aur/) |
| **Ubuntu / Debian PPA** | Launchpad PPA | Set up automated PPA builds for `apt install find-me` across Ubuntu LTS releases |
| **Homebrew Linux** | Homebrew Core / Tap | Provide a Homebrew formula for Linux users managing CLI utilities via `brew` |

---

## 📜 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
