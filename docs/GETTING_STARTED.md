# Getting Started with Find-Me

This guide walks you through system requirements, installation methods, launching Find-Me, and your first search workflow.

---

## 1. System Requirements

* **Operating System**: Linux (Debian 12+, Ubuntu 22.04+, Fedora 38+, Arch Linux, openSUSE, or any modern POSIX Linux distribution).
* **Display Server**: Wayland or X11.
* **Architecture**: `x86_64` (AMD64) or `aarch64` (ARM64).
* **Memory**: Minimum 512 MB RAM (Find-Me typically consumes ~15 MB for 200,000 files).
* **Privileges**: Runs 100% in unprivileged user space — **no root or `sudo` required**.

---

## 2. Quick Installation

### Option A: Local User Environment (Recommended)
From the repository root, execute the automated user installer:
```bash
bash scripts/install_user.sh
```
This script:
1. Compiles the optimized release binary (`target/release/find-me`).
2. Installs the binary to `~/.local/bin/find-me`.
3. Installs high-resolution SVG vector branding icons to `~/.local/share/icons/hicolor/scalable/apps/find-me.svg`.
4. Creates a FreeDesktop application launcher in `~/.local/share/applications/find-me.desktop`.
5. Creates a Desktop shortcut on `~/Desktop/find-me.desktop`.
6. Refreshes the system desktop database and icon caches.

### Option B: Debian / Ubuntu Package (`.deb`)
```bash
bash scripts/package_deb.sh
sudo dpkg -i target/debian/find-me_*.deb
```

### Option C: Standalone Portable AppImage
```bash
bash scripts/package_appimage.sh
chmod +x target/appimage/Find-Me-x86_64.AppImage
./target/appimage/Find-Me-x86_64.AppImage
```

---

## 3. Launching Find-Me

You can launch Find-Me using any of the following methods:
1. **Application Dash / Menu**: Press the `Super` key and search for **Find-Me**.
2. **Desktop Shortcut**: Double-click the **Find-Me** icon on your Desktop.
3. **Terminal**: Run `find-me` in any terminal:
   ```bash
   find-me
   ```
4. **Pre-filled Query**: Open the GUI with a pre-filled search term:
   ```bash
   find-me --gui --query "realtor *.pdf"
   ```

---

## 4. First Search Workflow

1. **Initial Full Scan**: On first startup, Find-Me automatically indexes your `$HOME` directory roots in the background using `jwalk` multithreading. Full scans take less than 1–2 seconds for 100,000+ files.
2. **Instant Search**: Type any file name, extension, or wildcard into the search bar:
   - `*.pdf`
   - `faizan resume`
   - `size:>100MB dm:lastmonth`
3. **Switch Search Modes**: Click the search mode pills above the search bar or toggle them to search file content or semantic concepts:
   - **`[Everything]`**: Instant filename, extension, and path search.
   - **`[Content]`**: Full-text keyword search across document bodies.
   - **`[Semantic]`**: AI neural vector search by meaning and topic.
   - **`[Hybrid]`**: Combined lexical BM25 and vector fusion ($RRF$).
4. **Inspect & Open**:
   - Press `Enter` to open the file in its default Linux application.
   - Press `Alt + Enter` to view POSIX file permissions, inode numbers, and metadata.
   - Press `Space` to expand contextual text snippets.
   - Press `Ctrl + Shift + C` to copy the full path to your Wayland/X11 clipboard.
