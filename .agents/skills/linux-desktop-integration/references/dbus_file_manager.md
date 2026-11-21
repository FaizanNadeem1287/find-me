# DBus File Manager Integration (`FileManager1`)

This reference explains how to highlight specific files in the user's default Linux file manager using standard DBus protocols.

---

## 1. The `org.freedesktop.FileManager1` Standard

When a user right-clicks a search result and chooses **"Open Containing Folder"**, standard desktop apps shouldn't merely open the parent directory. They should highlight the target file directly in the file manager:

### DBus Specification
- **Destination**: `org.freedesktop.FileManager1`
- **Object Path**: `/org/freedesktop/FileManager1`
- **Interface**: `org.freedesktop.FileManager1`
- **Method**: `ShowItems(array of URIs: as, startup_id: s) -> void`

---

## 2. Implementation Pattern in Rust

Using the standard `zbus` crate or lightweight fallback command:

```rust
use std::process::Command;

pub fn show_item_in_folder(path: &std::path::Path) -> std::io::Result<()> {
    let canonical = path.canonicalize()?;
    let uri = format!("file://{}", canonical.display());

    // 1. Try DBus FileManager1 via gdbus command (standard on Ubuntu/Debian)
    let status = Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest", "org.freedesktop.FileManager1",
            "--object-path", "/org/freedesktop/FileManager1",
            "--method", "org.freedesktop.FileManager1.ShowItems",
            &format!("['{}']", uri),
            "",
        ])
        .status();

    if let Ok(s) = status {
        if s.success() {
            return Ok(());
        }
    }

    // 2. Fallback: Open parent folder via xdg-open / open crate
    if let Some(parent) = path.parent() {
        let _ = open::that_detached(parent);
    }

    Ok(())
}
```

---

## 3. Supported File Managers
The `FileManager1` interface is natively implemented by:
- **Nautilus** (GNOME / Ubuntu default)
- **Dolphin** (KDE Plasma)
- **Nemo** (Cinnamon / Linux Mint)
- **Thunar** (XFCE)
- **PCManFM-Qt** (LXQt)
