# System Theme Auto-Detection (Dark / Light)

This guide documents how to query the desktop environment's current color scheme preference on Linux.

---

## 1. The XDG Desktop Portal `color-scheme` Standard

Modern Linux desktops (GNOME 42+, KDE Plasma 5.24+, elementary OS, modern XFCE) standardize color-scheme signaling through the **XDG Desktop Portal**:

### Portal Settings Interface
- **Destination**: `org.freedesktop.portal.Desktop`
- **Object Path**: `/org/freedesktop/portal/desktop`
- **Interface**: `org.freedesktop.portal.Settings`
- **Namespace**: `org.freedesktop.appearance`
- **Key**: `color-scheme`

### Values:
- `0`: No preference (default to dark)
- `1`: Prefer dark theme
- `2`: Prefer light theme

---

## 2. Querying Theme in Rust

```rust
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemTheme {
    Dark,
    Light,
}

pub fn detect_system_theme() -> SystemTheme {
    // 1. Try XDG Desktop Portal via gdbus
    let output = Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest", "org.freedesktop.portal.Desktop",
            "--object-path", "/org/freedesktop/portal/desktop",
            "--method", "org.freedesktop.portal.Settings.Read",
            "org.freedesktop.appearance",
            "color-scheme",
        ])
        .output();

    if let Ok(out) = output {
        let text = String::from_utf8_lossy(&out.stdout);
        if text.contains("uint32 1") {
            return SystemTheme::Dark;
        } else if text.contains("uint32 2") {
            return SystemTheme::Light;
        }
    }

    // 2. GNOME fallback via gsettings
    let gnome_out = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output();

    if let Ok(out) = gnome_out {
        let text = String::from_utf8_lossy(&out.stdout);
        if text.contains("prefer-light") {
            return SystemTheme::Light;
        }
    }

    // Default to Dark theme for Find-Me
    SystemTheme::Dark
}
```
