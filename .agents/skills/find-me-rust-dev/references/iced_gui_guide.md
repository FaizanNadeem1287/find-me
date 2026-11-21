# Iced Desktop GUI Architecture & Implementation Guide

This guide details the architecture, component hierarchy, virtualization techniques, and theming rules for building the **Find-Me** desktop interface using the **Iced** GUI framework (pure Rust).

---

## 1. Elm Architecture in Iced

Iced follows the strict Elm Architecture (Model-View-Update):

```
       +---------------------------------------------+
       |                                             |
       v                                             |
 [ User Event ] ---> [ Message ] ---> [ update() ] --+
                                            |
                                            v
                                       [ State ]
                                            |
                                            v
                                       [ view() ] ---> [ UI Output ]
```

### Core Architecture Components:
1. **Model / State**: Single source of truth containing active tab, search input, query filters, search result items, indexing status, and configuration.
2. **Message**: An enum describing all possible UI and background events (`SearchInputChanged(String)`, `ExecuteSearch`, `EngineEventReceived(EngineEvent)`, `SelectRow(usize)`, `ChangeScreen(ScreenId)`).
3. **Update**: Pure or command-returning function `fn update(&mut self, message: Message) -> Task<Message>`. Modifies state and dispatches asynchronous background tasks.
4. **View**: Pure function `fn view(&self) -> Element<Message>` producing the widget hierarchy.
5. **Subscription**: `fn subscription(&self) -> Subscription<Message>` listening to external event streams (Tokio channels from the engine, keyboard shortcuts, window events).

---

## 2. Screen & Component Architecture

```
src/
├── ui/
│   ├── mod.rs               # UI module entry point
│   ├── app.rs               # Top-level Application struct and update loop
│   ├── state.rs             # Application-wide state definition
│   ├── message.rs           # Master Message enum
│   ├── theme.rs             # Custom Light & Dark themes
│   ├── screens/             # Dedicated screen modules
│   │   ├── search.rs        # Main Search view (search bar, virtual table, status)
│   │   ├── advanced.rs      # Advanced Search filters (ext, size, dates, regex)
│   │   ├── index_mgmt.rs    # Index paths, exclusions, rebuild, DB size
│   │   ├── history.rs       # Recent queries, saved searches
│   │   ├── settings.rs      # Preferences, themes, startup, performance
│   │   └── about.rs         # Version, license, credits
│   └── components/          # Reusable UI widgets
│       ├── sidebar.rs       # Navigation sidebar with icons and badge counters
│       ├── search_bar.rs    # Prominent search input with clear and icon
│       ├── virtual_table.rs # Virtualized results table (Name, Path, Size, Modified)
│       ├── context_menu.rs  # Right-click file menu (Open, Show in Folder, Copy Path)
│       └── status_bar.rs    # Bottom status bar (Index stats, search time, match count)
```

---

## 3. High-Performance Virtualized Table (100k+ Rows)

Rendering thousands of individual widgets crashes UI memory and drops framerates. To achieve 60 FPS scrolling over millions of indexed results, the table **MUST** be virtualized:
- **`iced::widget::table` / `VirtualList`**: Only render rows currently visible in the window viewport plus a small buffer (e.g., 30 visible rows + 10 pre-rendered above and below).
- **Fixed Row Heights**: Enables $O(1)$ calculation of start/end indices based on `scroll_offset`.
- **Zero Allocations per Frame**: Format file sizes (`"12.4 MB"`) and timestamps lazily during cell rendering or cache them inside `SearchResultItem`.

---

## 4. Keyboard-First Navigation

Find-Me is engineered for power users who demand keyboard shortcuts:
- `/` or `Ctrl + F`: Focus the search input field.
- `Up` / `Down` Arrow keys: Select previous / next file in search results.
- `Enter`: Open selected file with default system application.
- `Alt + Enter`: Open file properties or enclosing folder in file manager.
- `Ctrl + Shift + C`: Copy full file path to clipboard.
- `Escape`: Clear search query or dismiss open context menu.
- `Ctrl + ,`: Open Settings screen.
- `Ctrl + 1` through `Ctrl + 6`: Switch between screens.

---

## 5. Design System & Theming

- **Dark Theme (Default)**: Deep slate/charcoal backgrounds (`#1E1E2E` or `#181825`), crisp text (`#CDD6F4`), subtle borders (`#313244`), accent blue/cyan (`#89B4FA`).
- **Light Theme**: Soft off-white backgrounds (`#EFF1F5`), high-contrast dark text (`#4C4F69`), crisp border lines.
- **Visual Discipline**:
  - No bloated animations, heavy drop-shadows, or blurry transparencies.
  - Consistent 8px spacing grid (`4px`, `8px`, `16px`, `24px`).
  - Native Linux look-and-feel that respects FreeDesktop desktop conventions.
