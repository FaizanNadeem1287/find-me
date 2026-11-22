# Find-Me Desktop UI & Offline Semantic Search Engine

Find-Me is an ultra-fast, lightweight, high-performance, offline-first Linux file search application inspired by *Everything* for Windows, written in 100% pure Rust with an Iced GUI.

---

## 📚 Official Documentation

Comprehensive documentation is available in the [`docs/`](./docs/) directory:

* 🚀 [**Getting Started**](./docs/GETTING_STARTED.md): System requirements, installation, and first search.
* 🏛️ [**Architecture & Internals**](./docs/ARCHITECTURE.md): Compact 32-byte arena, parallel crawler, inotify engine, binary persistence.
* 🔍 [**Search Syntax & Operators**](./docs/SEARCH_SYNTAX.md): Everything operators, wildcards, boolean logic, size filters, date math.
* 🧠 [**Semantic Search & AI Pipeline**](./docs/SEMANTIC_SEARCH_AND_AI.md): Document extractors, 384-dim embeddings, HNSW vector store, BM25, and Hybrid RRF.
* ⌨️ [**Keyboard Navigation & Shortcuts**](./docs/KEYBOARD_SHORTCUTS.md): Global shortcuts, screen switching, row navigation, clipboard actions.
* 💻 [**CLI & Headless Automation**](./docs/CLI_AND_HEADLESS.md): Headless querying, JSON streaming, CSV/TXT/EFU exporters.
* 📦 [**Packaging & Distribution**](./docs/PACKAGING_AND_DISTRIBUTION.md): Debian packages, AppImage, Flatpak, Arch Linux AUR, FreeDesktop XDG.
* 🛠️ [**Developer & Contributor Guide**](./docs/DEVELOPER_GUIDE.md): Toolchain setup, testing pyramid, Criterion benchmarks, git branch rules.
* ❓ [**Troubleshooting & FAQ**](./docs/TROUBLESHOOTING_AND_FAQ.md): Inotify limits, permission issues, index rebuilds, and power throttling.

---

## Technology Constraints

- Programming language: Rust.
- GUI framework: Iced.
- All UI components must be implemented in Rust.
- Do not use React, JavaScript, HTML, Electron, Tauri, Python, or web-based UI frameworks.
- Keep the desktop UI separate from the core indexing and search engine.
- Use reusable components and modular screen architecture.

## Screens to Implement

### 1. Search
- Main search input.
- Real-time search results.
- Results table with name, location, type, size and modified date.
- Sidebar navigation.
- Sorting and filtering.
- File context menu.
- Search and indexing status.

### 2. Advanced Search
- Extension filtering.
- File size conditions.
- Modification date filters.
- Directory restrictions.
- Include/exclude conditions.
- Regular expression support.

### 3. Index Management
- Indexed directories.
- Excluded directories.
- Add/remove directory actions.
- Indexing progress.
- Indexed file and folder counts.
- Database size.
- Rebuild and refresh index actions.

### 4. Search History
- Recent queries.
- Repeat search.
- Delete individual queries.
- Clear history.
- Saved searches.

### 5. Settings
- General settings.
- Appearance settings.
- Search preferences.
- Indexing preferences.
- Privacy and security settings.

### 6. About
- Application name and logo.
- Version.
- Description.
- License.
- Repository link.

## Design Direction

Create a professional, minimal, modern Linux desktop application.

- Prioritize usability over decoration.
- Provide dark and light themes.
- Use consistent spacing and typography.
- Keep the search input visually prominent.
- Use clean tables instead of unnecessary cards.
- Support keyboard-first navigation.
- Provide proper loading, empty, error and success states.
- Avoid excessive gradients, animations and visual effects.
- Ensure the interface works at different window sizes.

## Architecture Requirements

- Separate screens from reusable UI components.
- Keep application state centralized and predictable.
- Use Iced's message-driven architecture.
- Keep long-running indexing and database operations off the UI thread.
- Communicate with the core engine through well-defined Rust interfaces.
- Handle errors explicitly.
- Avoid blocking the UI during search or indexing.

## Development Order

1. Build the application shell and sidebar.
2. Implement the main search screen using mock data.
3. Implement reusable search results and context menu components.
4. Implement index management.
5. Implement settings and remaining screens.
6. Integrate the real Rust indexing and search engine.
7. Add keyboard shortcuts, accessibility and responsive layouts.
8. Test the complete application.

## Important

Do not implement all screens as one giant Rust file.

Build reusable components, keep modules focused, and maintain a consistent design system.

Do not introduce fake functionality. If a backend feature is not yet implemented, clearly represent it as unavailable or use explicitly identified mock data during UI development.