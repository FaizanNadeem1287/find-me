# Keyboard Navigation & Shortcuts

Find-Me is engineered with a **keyboard-first philosophy**. Every screen, search mode, table row, preview snippet, and modal can be navigated and controlled entirely without a mouse.

---

## 1. Global Navigation & Screen Switching

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `Ctrl + 1` | **Search Screen** | Switches to the primary search results view. |
| `Ctrl + 2` | **Advanced Search** | Opens the visual multi-criteria query builder. |
| `Ctrl + 3` | **Index Management** | Opens folder roots, exclusion rules, and vector indexing status. |
| `Ctrl + 4` | **Search History** | Views past search queries, match counts, and timestamps. |
| `Ctrl + 5` | **Settings & Preferences** | Configures launch behavior, performance, and default actions. |
| `Ctrl + 6` | **About** | System architecture, version info, and developer credits. |

---

## 2. Search & Focus Controls

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `Ctrl + F` or `Ctrl + K` | **Focus Search Bar** | Moves cursor to the main search bar and selects all text. |
| `Esc` | **Clear / Dismiss** | Clears the active query, closes banners, or dismisses open modals. |
| `Ctrl + B` | **Toggle Sidebar** | Collapses or expands the navigation sidebar (compact mode). |
| `Ctrl + T` | **Toggle Theme** | Cycles between Ergonomic Dark Mode and Light Mode. |
| `Ctrl + R` | **Rebuild Index** | Triggers an immediate full index rebuild across monitored roots. |

---

## 3. Results Table Navigation

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `Up` / `Down` | **Navigate Rows** | Moves selection up or down through search results. |
| `Page Up` / `Page Down` | **Jump Page** | Jumps selection by 15 rows up or down. |
| `Home` / `End` | **Jump to Top / Bottom**| Selects the first or last search result. |
| `Enter` | **Open File** | Opens the selected file in its default Linux application (`xdg-open`). |
| `Alt + Enter` | **Inspect Properties** | Opens the POSIX properties modal (permissions, inode, size, timestamps). |
| `Space` | **Toggle Snippet** | Expands or collapses contextual matching snippets beneath the row. |

---

## 4. Clipboard & Path Operations

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `Ctrl + Shift + C` | **Copy Full Path** | Copies the absolute POSIX file path to the Wayland/X11 clipboard. |
| `Ctrl + Shift + N` | **Copy File Name** | Copies only the file name (basename) to the clipboard. |
| `Ctrl + Shift + U` | **Copy File URL** | Copies the `file://` URI schema to the clipboard. |

---

## 5. Linux Desktop Integration Actions

* **Highlight in File Manager**: Double-clicking or triggering "Show in File Manager" sends a DBus `org.freedesktop.FileManager1.ShowItems` call, opening Nautilus, Dolphin, Thunar, or Nemo with the file highlighted.
* **Audio Timestamp Playback**: In semantic audio results, clicking `[▶ Play from MM:SS]` automatically opens the audio player and jumps to that specific playback offset.
