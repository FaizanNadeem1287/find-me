//! Linux desktop environment integration
//!
//! Provides FreeDesktop DBus `FileManager1` file highlighting, cross-protocol
//! clipboard management (Wayland / X11), and low-level POSIX file metadata extraction.

use findme_core::SearchResultItem;
use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Detailed POSIX file metadata for properties inspection
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileProperties {
    pub path: PathBuf,
    pub name: String,
    pub size_bytes: u64,
    pub formatted_size: String,
    pub is_dir: bool,
    pub mode: u32,
    pub mode_string: String,
    pub inode: u64,
    pub device_id: u64,
    pub mtime_secs: u64,
    pub mtime_formatted: String,
    pub flags_description: String,
}

impl FileProperties {
    /// Formats POSIX file mode bits into symbolic notation and octal (e.g. "-rwxr-xr-x (0755)")
    pub fn format_mode(mode: u32, is_dir: bool) -> String {
        let type_char = if is_dir { 'd' } else { '-' };
        let user_r = if (mode & 0o400) != 0 { 'r' } else { '-' };
        let user_w = if (mode & 0o200) != 0 { 'w' } else { '-' };
        let user_x = if (mode & 0o100) != 0 { 'x' } else { '-' };
        let group_r = if (mode & 0o040) != 0 { 'r' } else { '-' };
        let group_w = if (mode & 0o020) != 0 { 'w' } else { '-' };
        let group_x = if (mode & 0o010) != 0 { 'x' } else { '-' };
        let other_r = if (mode & 0o004) != 0 { 'r' } else { '-' };
        let other_w = if (mode & 0o002) != 0 { 'w' } else { '-' };
        let other_x = if (mode & 0o001) != 0 { 'x' } else { '-' };

        format!(
            "{}{}{}{}{}{}{}{}{}{} ({:04o})",
            type_char,
            user_r,
            user_w,
            user_x,
            group_r,
            group_w,
            group_x,
            other_r,
            other_w,
            other_x,
            mode & 0o777
        )
    }

    /// Formats epoch seconds into human-readable UTC timestamp
    pub fn format_timestamp(secs: u64) -> String {
        const SECS_PER_MINUTE: u64 = 60;
        const SECS_PER_HOUR: u64 = 3600;
        const SECS_PER_DAY: u64 = 86400;

        let days = secs / SECS_PER_DAY;
        let rem_day = secs % SECS_PER_DAY;
        let hours = rem_day / SECS_PER_HOUR;
        let rem_hour = rem_day % SECS_PER_HOUR;
        let mins = rem_hour / SECS_PER_MINUTE;
        let seconds = rem_hour % SECS_PER_MINUTE;

        // Approximate calendar conversion from epoch (1970-01-01)
        let mut year = 1970;
        let mut remaining_days = days;

        loop {
            let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
            let days_in_year = if is_leap { 366 } else { 365 };
            if remaining_days < days_in_year {
                break;
            }
            remaining_days -= days_in_year;
            year += 1;
        }

        let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let month_days = [
            31,
            if is_leap { 29 } else { 28 },
            31,
            30,
            31,
            30,
            31,
            31,
            30,
            31,
            30,
            31,
        ];

        let mut month = 1;
        for &m_days in &month_days {
            if remaining_days < m_days {
                break;
            }
            remaining_days -= m_days;
            month += 1;
        }
        let day = remaining_days + 1;

        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
            year, month, day, hours, mins, seconds
        )
    }

    /// Creates properties from an existing `SearchResultItem` and optional filesystem metadata
    pub fn from_item(item: &SearchResultItem, meta: Option<&Metadata>) -> Self {
        let (mode, inode, device_id) = if let Some(m) = meta {
            (m.mode(), m.ino(), m.dev())
        } else {
            (0o644, 0, 0)
        };

        let mode_string = Self::format_mode(mode, item.is_dir);
        let mtime_formatted = Self::format_timestamp(item.mtime_secs);

        let mut flags = Vec::new();
        if item.is_dir {
            flags.push("Directory");
        } else {
            flags.push("Regular File");
        }
        if item.name.starts_with('.') {
            flags.push("Hidden");
        }
        if (mode & 0o111) != 0 {
            flags.push("Executable");
        }
        if (mode & 0o200) == 0 {
            flags.push("Read-Only");
        }
        let flags_description = flags.join(", ");

        Self {
            path: PathBuf::from(&item.path),
            name: item.name.clone(),
            size_bytes: item.size_bytes,
            formatted_size: item.formatted_size(),
            is_dir: item.is_dir,
            mode,
            mode_string,
            inode,
            device_id,
            mtime_secs: item.mtime_secs,
            mtime_formatted,
            flags_description,
        }
    }
}

/// Reads filesystem metadata and builds a `FileProperties` record
pub fn get_file_properties(path: &Path) -> Result<FileProperties, String> {
    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());

    let meta = path
        .metadata()
        .map_err(|e| format!("Failed to read metadata for {}: {e}", path.display()))?;

    let is_dir = meta.is_dir();
    let size_bytes = if is_dir { 0 } else { meta.len() };
    let mtime_secs = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let item = SearchResultItem {
        id: 0,
        name,
        path: path.to_string_lossy().to_string(),
        size_bytes,
        mtime_secs,
        is_dir,
    };

    Ok(FileProperties::from_item(&item, Some(&meta)))
}

/// Highlights a file inside the user's default Linux file manager via FreeDesktop DBus `FileManager1`
///
/// Falls back gracefully to opening the parent folder via `open::that_detached` if the DBus
/// daemon or file manager is not responding.
pub fn show_in_file_manager(path: &Path) -> Result<(), String> {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let uri = format!("file://{}", canonical.display());

    // 1. Attempt DBus call to org.freedesktop.FileManager1.ShowItems
    let dbus_result = Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            "org.freedesktop.FileManager1",
            "--object-path",
            "/org/freedesktop/FileManager1",
            "--method",
            "org.freedesktop.FileManager1.ShowItems",
            &format!("['{}']", uri),
            "",
        ])
        .output();

    if let Ok(output) = dbus_result {
        if output.status.success() {
            return Ok(());
        }
    }

    // 2. Fallback: Open parent folder with default file manager via open crate
    if let Some(parent) = canonical.parent() {
        open::that_detached(parent)
            .map_err(|e| format!("Failed to open containing folder: {e}"))?;
        Ok(())
    } else {
        Err("Path has no parent directory".to_string())
    }
}

/// Copies string contents to system clipboard across Wayland and X11 sessions
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| format!("Failed to access clipboard: {e}"))?;
    clipboard
        .set_text(text)
        .map_err(|e| format!("Failed to write text to clipboard: {e}"))?;
    Ok(())
}

/// Copies the canonical file URL (`file://...`) to the system clipboard
pub fn copy_file_uri(path: &Path) -> Result<String, String> {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let uri = format!("file://{}", canonical.display());
    copy_to_clipboard(&uri)?;
    Ok(uri)
}
