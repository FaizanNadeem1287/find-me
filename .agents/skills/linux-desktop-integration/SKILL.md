---
name: linux-desktop-integration
description: >-
  Comprehensive guide and reference for integrating Find-Me with Linux desktop environments
  (GNOME, KDE, XFCE). Covers FreeDesktop XDG specifications, DBus FileManager1 file highlighting,
  system dark/light theme detection, Wayland and X11 clipboard handling, and desktop entry creation.
---

# Linux Desktop Integration Skill

This skill provides architecture guidelines, DBus interfaces, FreeDesktop specifications, and code patterns for integrating **Find-Me** natively into modern Linux desktop environments.

---

## Core Desktop Capabilities

1. **DBus File Manager Highlighting (`FileManager1`)**:
   - Rather than merely opening the parent directory, call the FreeDesktop DBus interface `org.freedesktop.FileManager1.ShowItems` to launch the user's default file manager (Nautilus, Dolphin, Nemo, Thunar) and automatically select/highlight the exact file.
2. **OS Dark / Light Theme Detection**:
   - Query the FreeDesktop appearance portal (`org.freedesktop.appearance.color-scheme`) to automatically match the desktop theme when the user switches their OS dark/light mode.
3. **Cross-Protocol System Clipboard**:
   - Seamlessly copy file paths or file buffers to both Wayland data device buffers and X11 clipboard selections using `arboard`.
4. **Standard FreeDesktop Application Packaging**:
   - Standardized `find-me.desktop` launcher in `/usr/share/applications/`.
   - Scalable SVG and multi-resolution PNG icons in `/usr/share/icons/hicolor/`.
   - AppStream metadata in `/usr/share/metainfo/`.

---

## Detailed Reference Guides

- [DBus File Manager Integration Guide](./references/dbus_file_manager.md): How to call `org.freedesktop.FileManager1` and fallback to `xdg-open`.
- [XDG Base Directories & Desktop Entry Guide](./references/xdg_and_desktop_entry.md): FreeDesktop `.desktop` entry standards, icon paths, and MIME type handling.
- [System Theme Auto-Detection Guide](./references/system_theme_detection.md): Reading `color-scheme` from portals and gsettings.
