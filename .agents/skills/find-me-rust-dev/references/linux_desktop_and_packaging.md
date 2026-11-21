# Linux Desktop Integration & Debian Packaging Guide

This guide covers standards compliance, packaging, and desktop integration for Ubuntu 22.04+ and Debian-based Linux distributions.

---

## 1. FreeDesktop Standards Compliance

To integrate natively into GNOME, KDE, and other Linux desktop environments:

### XDG Base Directory Specification
- **Configuration**: `$XDG_CONFIG_HOME/find-me/config.toml` (defaults to `~/.config/find-me/config.toml`)
- **Cache & Index Database**: `$XDG_CACHE_HOME/find-me/index.bin` (defaults to `~/.cache/find-me/index.bin`)
- **Data & History**: `$XDG_DATA_HOME/find-me/history.json` (defaults to `~/.local/share/find-me/history.json`)

### Desktop Entry (`find-me.desktop`)
Must be placed in `/usr/share/applications/`:
```ini
[Desktop Entry]
Name=Find-Me
Comment=High-performance instant desktop file search
Exec=find-me %F
Icon=find-me
Terminal=false
Type=Application
Categories=Utility;Filesystem;Core;
Keywords=search;find;everything;files;fast;index;
StartupNotify=true
StartupWMClass=find-me
```

### Application Icons
Scalable SVG and standard PNG resolutions placed in `/usr/share/icons/hicolor/`:
- `/usr/share/icons/hicolor/scalable/apps/find-me.svg`
- `/usr/share/icons/hicolor/48x48/apps/find-me.png`
- `/usr/share/icons/hicolor/128x128/apps/find-me.png`
- `/usr/share/icons/hicolor/256x256/apps/find-me.png`

---

## 2. Native File & Desktop Operations

When the user interacts with search results, Find-Me integrates with the desktop shell:
1. **Open File (`xdg-open`)**:
   - Executes `xdg-open <path>` or uses `open::that_detached(path)` to launch the default system application without blocking or inheriting stdout/stderr.
2. **Open Enclosing Folder (`dbus` / File Manager)**:
   - Prefer DBus `org.freedesktop.FileManager1.ShowItems` to highlight the exact selected file inside Nautilus, Dolphin, or Nemo.
   - Fallback: `xdg-open <parent_directory>`.
3. **Copy to Clipboard**:
   - Handled via `arboard` or `copypasta` crate (Wayland and X11 native clipboard protocols).

---

## 3. Debian Package Generation (`cargo-deb`)

### Prerequisites on Ubuntu/Debian
```bash
sudo apt-get install -y build-essential libssl-dev pkg-config libxkbcommon-dev libwayland-dev
cargo install cargo-deb
```

### Cargo.toml Debian Metadata Configuration
```toml
[package.metadata.deb]
maintainer = "Find-Me Team <dev@find-me.local>"
copyright = "2026 Find-Me Authors"
license-file = ["LICENSE", "4"]
extended-description = """\
Find-Me is a lightweight, high-performance, offline-first Linux file search \
application inspired by Everything for Windows. It provides real-time search \
across millions of files with instant indexing and low resource usage."""
depends = "$auto, libxkbcommon0, libwayland-client0"
section = "utils"
priority = "optional"
assets = [
    ["target/release/find-me", "usr/bin/", "755"],
    ["assets/find-me.desktop", "usr/share/applications/", "644"],
    ["assets/icons/find-me.svg", "usr/share/icons/hicolor/scalable/apps/", "644"],
]
```

### Building the `.deb` Package
```bash
cargo build --release
cargo deb
# Generates target/debian/find-me_<version>_<arch>.deb
```

### Testing the Package
```bash
# Test installation
sudo dpkg -i target/debian/find-me_*.deb

# Verify installation files
dpkg -L find-me

# Launch application
find-me
```
