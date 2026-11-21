# FreeDesktop XDG Specifications & Desktop Entry Standards

This reference details the installation paths, desktop entry format, and icon requirements for Ubuntu and Debian systems.

---

## 1. Desktop Entry Specification (`find-me.desktop`)

Location: `/usr/share/applications/find-me.desktop`

```ini
[Desktop Entry]
Version=1.5
Type=Application
Name=Find-Me
GenericName=File Search Utility
Comment=Instant, lightweight Linux desktop file search inspired by Everything
Exec=find-me %F
Icon=find-me
Terminal=false
Categories=Utility;Filesystem;Core;
Keywords=search;find;everything;files;fast;index;
StartupNotify=true
StartupWMClass=find-me
MimeType=inode/directory;
Actions=NewWindow;RebuildIndex;

[Desktop Action NewWindow]
Name=New Search Window
Exec=find-me --new-window

[Desktop Action RebuildIndex]
Name=Rebuild Search Index
Exec=find-me --rebuild-index
```

### Validating Desktop Entries
```bash
desktop-file-validate assets/find-me.desktop
```

---

## 2. Icon Directory Hierarchy

Icons must be placed in standard FreeDesktop theme directories:
- Scalable vector: `/usr/share/icons/hicolor/scalable/apps/find-me.svg`
- Standard rasters:
  - `/usr/share/icons/hicolor/48x48/apps/find-me.png`
  - `/usr/share/icons/hicolor/128x128/apps/find-me.png`
  - `/usr/share/icons/hicolor/256x256/apps/find-me.png`

Update icon cache post-installation:
```bash
gtk-update-icon-cache -f -t /usr/share/icons/hicolor
```

---

## 3. XDG Base Directory Specification

Path Resolution in Rust using `directories` crate:
```rust
use directories::ProjectDirs;

pub fn get_app_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("io.github", "FindMe", "find-me")
}
```
- **Configuration**: `~/.config/find-me/config.toml` (`dirs.config_dir()`)
- **Persistent Cache**: `~/.cache/find-me/index.bin` (`dirs.cache_dir()`)
- **User Data / History**: `~/.local/share/find-me/history.json` (`dirs.data_dir()`)
