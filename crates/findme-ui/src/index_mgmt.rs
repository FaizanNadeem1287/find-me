//! Index management state, directory roots, exclusion patterns, and statistics models

use std::path::{Path, PathBuf};

/// Status of an indexed directory root
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RootStatus {
    /// Fully indexed and synchronized
    Indexed,
    /// Currently being crawled or rescanned
    Scanning,
    /// Encountered an error (e.g. permission denied or missing path)
    Error(String),
}

impl RootStatus {
    /// Returns a short display string for the status
    pub fn as_str(&self) -> &str {
        match self {
            RootStatus::Indexed => "Indexed",
            RootStatus::Scanning => "Scanning...",
            RootStatus::Error(_) => "Error",
        }
    }
}

/// An indexed root directory configured by the user
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedRoot {
    pub path: PathBuf,
    pub status: RootStatus,
    pub file_count: u64,
    pub folder_count: u64,
}

impl IndexedRoot {
    /// Creates a new indexed root with default status
    pub fn new(path: impl Into<PathBuf>, file_count: u64, folder_count: u64) -> Self {
        Self {
            path: path.into(),
            status: RootStatus::Indexed,
            file_count,
            folder_count,
        }
    }

    /// Formatted display path
    pub fn display_path(&self) -> String {
        self.path.to_string_lossy().to_string()
    }
}

/// Directory or filename pattern excluded from indexing
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExcludedPattern {
    pub pattern: String,
    pub is_builtin: bool,
}

impl ExcludedPattern {
    /// Creates a new exclusion pattern
    pub fn new(pattern: impl Into<String>, is_builtin: bool) -> Self {
        Self {
            pattern: pattern.into(),
            is_builtin,
        }
    }
}

/// Live indexing metrics and system statistics
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexStats {
    pub total_files: u64,
    pub total_folders: u64,
    pub memory_bytes: u64,
    pub last_scan_secs: Option<u64>,
    pub watcher_active: bool,
    pub watcher_watch_count: usize,
}

impl Default for IndexStats {
    fn default() -> Self {
        Self {
            total_files: 142_850,
            total_folders: 18_420,
            memory_bytes: 14_680_064, // ~14.0 MB
            last_scan_secs: Some(1_700_000_000),
            watcher_active: true,
            watcher_watch_count: 24,
        }
    }
}

impl IndexStats {
    /// Formats memory consumption in compact human-readable units (e.g. "14.0 MB", "850 KB")
    pub fn formatted_memory(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = 1024 * KB;
        const GB: u64 = 1024 * MB;

        if self.memory_bytes >= GB {
            format!("{:.1} GB", self.memory_bytes as f64 / GB as f64)
        } else if self.memory_bytes >= MB {
            format!("{:.1} MB", self.memory_bytes as f64 / MB as f64)
        } else if self.memory_bytes >= KB {
            format!("{:.1} KB", self.memory_bytes as f64 / KB as f64)
        } else {
            format!("{} B", self.memory_bytes)
        }
    }

    /// Formats total file count with thousands separators
    pub fn formatted_files(&self) -> String {
        crate::state::format_number(self.total_files)
    }

    /// Formats total folder count with thousands separators
    pub fn formatted_folders(&self) -> String {
        crate::state::format_number(self.total_folders)
    }

    /// Formats last scan timestamp
    pub fn formatted_last_scan(&self) -> String {
        match self.last_scan_secs {
            Some(secs) if secs > 0 => {
                // In unprivileged environment, produce a stable human-friendly format
                "Recent (Up to date)".to_string()
            }
            _ => "Never".to_string(),
        }
    }

    /// Watcher status label
    pub fn watcher_status_label(&self) -> String {
        if self.watcher_active {
            format!("● Active (inotify: {} watches)", self.watcher_watch_count)
        } else {
            "○ Inactive (Polling only)".to_string()
        }
    }
}

/// Current indexing activity state
#[derive(Debug, Clone, PartialEq)]
pub enum IndexingState {
    /// Idle, no active crawl
    Idle,
    /// Active crawl in progress
    Scanning {
        current_path: String,
        files_found: u64,
        percent: f32,
    },
    /// Error occurred during index maintenance
    Error(String),
}

impl IndexingState {
    /// Returns true if an active crawl is running
    pub fn is_scanning(&self) -> bool {
        matches!(self, IndexingState::Scanning { .. })
    }
}

/// Central state model for the Index Management screen
#[derive(Debug, Clone, PartialEq)]
pub struct IndexManagementState {
    pub stats: IndexStats,
    pub roots: Vec<IndexedRoot>,
    pub exclusions: Vec<ExcludedPattern>,
    pub indexing_state: IndexingState,
    pub new_root_input: String,
    pub new_exclusion_input: String,
}

impl Default for IndexManagementState {
    fn default() -> Self {
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
        let roots = vec![IndexedRoot::new(home_dir, 142_850, 18_420)];

        let exclusions = vec![
            ExcludedPattern::new("/proc", true),
            ExcludedPattern::new("/sys", true),
            ExcludedPattern::new("/dev", true),
            ExcludedPattern::new("~/.cache", false),
            ExcludedPattern::new("node_modules", false),
            ExcludedPattern::new("target", false),
            ExcludedPattern::new(".git", false),
            ExcludedPattern::new("~/.local/share/Trash", false),
        ];

        Self {
            stats: IndexStats::default(),
            roots,
            exclusions,
            indexing_state: IndexingState::Idle,
            new_root_input: String::new(),
            new_exclusion_input: String::new(),
        }
    }
}

impl IndexManagementState {
    /// Adds a new root directory to the indexed list, deduplicating paths
    pub fn add_root(&mut self, path: PathBuf) -> bool {
        let clean_path = PathBuf::from(path.to_string_lossy().trim().trim_end_matches('/'));
        if clean_path.as_os_str().is_empty() {
            return false;
        }

        if self.roots.iter().any(|r| r.path == clean_path) {
            return false; // Already present
        }

        self.roots.push(IndexedRoot::new(clean_path, 0, 0));
        true
    }

    /// Removes an indexed root directory
    pub fn remove_root(&mut self, path: &Path) -> bool {
        let original_len = self.roots.len();
        self.roots.retain(|r| r.path != path);
        self.roots.len() < original_len
    }

    /// Adds a new exclusion pattern, avoiding duplicates
    pub fn add_exclusion(&mut self, pattern: String) -> bool {
        let trimmed = pattern.trim().to_string();
        if trimmed.is_empty() {
            return false;
        }

        if self
            .exclusions
            .iter()
            .any(|e| e.pattern.eq_ignore_ascii_case(&trimmed))
        {
            return false;
        }

        self.exclusions.push(ExcludedPattern::new(trimmed, false));
        true
    }

    /// Removes an exclusion pattern (only allowed for non-builtin patterns)
    pub fn remove_exclusion(&mut self, pattern: &str) -> bool {
        let original_len = self.exclusions.len();
        self.exclusions
            .retain(|e| !(e.pattern == pattern && !e.is_builtin));
        self.exclusions.len() < original_len
    }

    /// Adds default common development and system exclusion patterns
    pub fn add_default_exclusions(&mut self) {
        let defaults = [
            ".git",
            "node_modules",
            "target",
            "__pycache__",
            ".venv",
            "~/.cache",
            "~/.local/share/Trash",
            ".cargo",
        ];

        for def in defaults {
            if !self.exclusions.iter().any(|e| e.pattern == def) {
                self.exclusions.push(ExcludedPattern::new(def, false));
            }
        }
    }
}
