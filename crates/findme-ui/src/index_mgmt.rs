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

/// Preset category identifiers for 1-click .gitignore exclusion toggles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExclusionPreset {
    NodeModules,
    Git,
    PythonCache,
    RustTarget,
    BuildArtifacts,
    SystemCache,
    VirtualEnvs,
    TempLogs,
    HiddenFiles,
}

impl ExclusionPreset {
    pub const ALL: [ExclusionPreset; 9] = [
        ExclusionPreset::NodeModules,
        ExclusionPreset::Git,
        ExclusionPreset::PythonCache,
        ExclusionPreset::RustTarget,
        ExclusionPreset::BuildArtifacts,
        ExclusionPreset::SystemCache,
        ExclusionPreset::VirtualEnvs,
        ExclusionPreset::TempLogs,
        ExclusionPreset::HiddenFiles,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            ExclusionPreset::NodeModules => "node_modules/",
            ExclusionPreset::Git => ".git/",
            ExclusionPreset::PythonCache => "Python (*.pyc, __pycache__)",
            ExclusionPreset::RustTarget => "Rust (target/)",
            ExclusionPreset::BuildArtifacts => "Build (dist/, build/)",
            ExclusionPreset::SystemCache => "Caches (~/.cache)",
            ExclusionPreset::VirtualEnvs => "Venvs (.venv)",
            ExclusionPreset::TempLogs => "Logs & Temp (*.log, *.tmp)",
            ExclusionPreset::HiddenFiles => "Hidden Files (.*)",
        }
    }
}

/// Central state model for the Index Management screen
#[derive(Debug, Clone, PartialEq)]
pub struct IndexManagementState {
    pub stats: IndexStats,
    pub roots: Vec<IndexedRoot>,
    pub exclusions: Vec<ExcludedPattern>,
    pub excluded_folders: Vec<String>,
    pub excluded_extensions: Vec<String>,
    pub excluded_paths: Vec<String>,
    pub indexing_state: IndexingState,
    pub new_root_input: String,
    pub new_exclusion_input: String,
    pub new_extension_input: String,
    pub new_folder_input: String,
    pub view_gitignore_raw: bool,
}

impl Default for IndexManagementState {
    fn default() -> Self {
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
        let roots = vec![IndexedRoot::new(home_dir, 0, 0)];

        let mut state = Self {
            stats: IndexStats::default(),
            roots,
            exclusions: Vec::new(),
            excluded_folders: crate::settings::default_excluded_folders(),
            excluded_extensions: crate::settings::default_excluded_extensions(),
            excluded_paths: crate::settings::default_excluded_paths(),
            indexing_state: IndexingState::Idle,
            new_root_input: String::new(),
            new_exclusion_input: String::new(),
            new_extension_input: String::new(),
            new_folder_input: String::new(),
            view_gitignore_raw: false,
        };
        state.rebuild_flat_exclusions();
        state
    }
}

impl IndexManagementState {
    /// Internal helper to rebuild the flat `exclusions` vector for backwards compatibility
    pub fn rebuild_flat_exclusions(&mut self) {
        let mut list = vec![
            ExcludedPattern::new("/proc", true),
            ExcludedPattern::new("/sys", true),
            ExcludedPattern::new("/dev", true),
        ];

        for path in &self.excluded_paths {
            list.push(ExcludedPattern::new(path.clone(), false));
        }

        for folder in &self.excluded_folders {
            list.push(ExcludedPattern::new(folder.clone(), false));
        }

        for ext in &self.excluded_extensions {
            let pat = if ext.starts_with('.') || ext.starts_with('*') {
                ext.clone()
            } else {
                format!("*.{ext}")
            };
            list.push(ExcludedPattern::new(pat, false));
        }

        self.exclusions = list;
    }

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

    /// Smart rule parser that auto-routes user input into folders, extensions, or paths
    pub fn add_smart_rule(&mut self, input: &str) -> bool {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return false;
        }

        if trimmed.starts_with('/') || trimmed.starts_with('~') {
            self.add_excluded_path(trimmed)
        } else if trimmed.starts_with('.') || trimmed.starts_with("*.") {
            self.add_excluded_extension(trimmed)
        } else if trimmed.ends_with('/') {
            self.add_excluded_folder(trimmed.trim_end_matches('/'))
        } else {
            self.add_excluded_folder(trimmed)
        }
    }

    /// Adds a specific folder name to exclude everywhere
    pub fn add_excluded_folder(&mut self, folder: &str) -> bool {
        let clean = folder.trim().trim_end_matches('/').trim_start_matches('/');
        if clean.is_empty() {
            return false;
        }
        if self
            .excluded_folders
            .iter()
            .any(|f| f.eq_ignore_ascii_case(clean))
        {
            return false;
        }
        self.excluded_folders.push(clean.to_string());
        self.rebuild_flat_exclusions();
        true
    }

    /// Removes a specific folder name from exclusions
    pub fn remove_excluded_folder(&mut self, folder: &str) -> bool {
        let orig = self.excluded_folders.len();
        self.excluded_folders
            .retain(|f| !f.eq_ignore_ascii_case(folder));
        if self.excluded_folders.len() < orig {
            self.rebuild_flat_exclusions();
            true
        } else {
            false
        }
    }

    /// Adds a file extension to exclude everywhere (e.g. "pyc", ".tmp", "*.log")
    pub fn add_excluded_extension(&mut self, ext: &str) -> bool {
        let clean = ext
            .trim()
            .trim_start_matches('*')
            .trim_start_matches('.')
            .to_ascii_lowercase();
        if clean.is_empty() {
            return false;
        }
        if self.excluded_extensions.iter().any(|e| e == &clean) {
            return false;
        }
        self.excluded_extensions.push(clean);
        self.rebuild_flat_exclusions();
        true
    }

    /// Removes a file extension from exclusions
    pub fn remove_excluded_extension(&mut self, ext: &str) -> bool {
        let clean = ext
            .trim()
            .trim_start_matches('*')
            .trim_start_matches('.')
            .to_ascii_lowercase();
        let orig = self.excluded_extensions.len();
        self.excluded_extensions.retain(|e| e != &clean);
        if self.excluded_extensions.len() < orig {
            self.rebuild_flat_exclusions();
            true
        } else {
            false
        }
    }

    /// Adds a specific directory path to exclude
    pub fn add_excluded_path(&mut self, path: &str) -> bool {
        let clean = path.trim().trim_end_matches('/').to_string();
        if clean.is_empty() {
            return false;
        }
        if self.excluded_paths.iter().any(|p| p == &clean) {
            return false;
        }
        self.excluded_paths.push(clean);
        self.rebuild_flat_exclusions();
        true
    }

    /// Removes a specific directory path from exclusions
    pub fn remove_excluded_path(&mut self, path: &str) -> bool {
        let clean = path.trim().trim_end_matches('/');
        let orig = self.excluded_paths.len();
        self.excluded_paths.retain(|p| p != clean);
        if self.excluded_paths.len() < orig {
            self.rebuild_flat_exclusions();
            true
        } else {
            false
        }
    }

    /// Checks if a preset rule is active
    pub fn is_preset_active(&self, preset: ExclusionPreset) -> bool {
        match preset {
            ExclusionPreset::NodeModules => {
                self.excluded_folders.iter().any(|f| f == "node_modules")
            }
            ExclusionPreset::Git => self.excluded_folders.iter().any(|f| f == ".git"),
            ExclusionPreset::PythonCache => {
                self.excluded_folders.iter().any(|f| f == "__pycache__")
                    || self.excluded_extensions.iter().any(|e| e == "pyc")
            }
            ExclusionPreset::RustTarget => self.excluded_folders.iter().any(|f| f == "target"),
            ExclusionPreset::BuildArtifacts => self
                .excluded_folders
                .iter()
                .any(|f| f == "dist" || f == "build"),
            ExclusionPreset::SystemCache => {
                self.excluded_paths.iter().any(|p| p.contains(".cache"))
            }
            ExclusionPreset::VirtualEnvs => self
                .excluded_folders
                .iter()
                .any(|f| f == ".venv" || f == "venv"),
            ExclusionPreset::TempLogs => self
                .excluded_extensions
                .iter()
                .any(|e| e == "log" || e == "tmp"),
            ExclusionPreset::HiddenFiles => self.excluded_folders.iter().any(|f| f == ".*"),
        }
    }

    /// Toggles a preset on or off
    pub fn toggle_preset(&mut self, preset: ExclusionPreset) {
        let is_active = self.is_preset_active(preset);
        if is_active {
            match preset {
                ExclusionPreset::NodeModules => {
                    self.remove_excluded_folder("node_modules");
                }
                ExclusionPreset::Git => {
                    self.remove_excluded_folder(".git");
                }
                ExclusionPreset::PythonCache => {
                    self.remove_excluded_folder("__pycache__");
                    self.remove_excluded_extension("pyc");
                    self.remove_excluded_extension("pyo");
                }
                ExclusionPreset::RustTarget => {
                    self.remove_excluded_folder("target");
                }
                ExclusionPreset::BuildArtifacts => {
                    self.remove_excluded_folder("dist");
                    self.remove_excluded_folder("build");
                }
                ExclusionPreset::SystemCache => {
                    self.remove_excluded_path("~/.cache");
                }
                ExclusionPreset::VirtualEnvs => {
                    self.remove_excluded_folder(".venv");
                    self.remove_excluded_folder("venv");
                }
                ExclusionPreset::TempLogs => {
                    self.remove_excluded_extension("log");
                    self.remove_excluded_extension("tmp");
                }
                ExclusionPreset::HiddenFiles => {
                    self.remove_excluded_folder(".*");
                }
            }
        } else {
            match preset {
                ExclusionPreset::NodeModules => {
                    self.add_excluded_folder("node_modules");
                }
                ExclusionPreset::Git => {
                    self.add_excluded_folder(".git");
                }
                ExclusionPreset::PythonCache => {
                    self.add_excluded_folder("__pycache__");
                    self.add_excluded_extension("pyc");
                    self.add_excluded_extension("pyo");
                }
                ExclusionPreset::RustTarget => {
                    self.add_excluded_folder("target");
                }
                ExclusionPreset::BuildArtifacts => {
                    self.add_excluded_folder("dist");
                    self.add_excluded_folder("build");
                }
                ExclusionPreset::SystemCache => {
                    self.add_excluded_path("~/.cache");
                }
                ExclusionPreset::VirtualEnvs => {
                    self.add_excluded_folder(".venv");
                    self.add_excluded_folder("venv");
                }
                ExclusionPreset::TempLogs => {
                    self.add_excluded_extension("log");
                    self.add_excluded_extension("tmp");
                }
                ExclusionPreset::HiddenFiles => {
                    self.add_excluded_folder(".*");
                }
            }
        }
        self.rebuild_flat_exclusions();
    }

    /// Renders current active exclusion rules into standard .gitignore syntax
    pub fn to_gitignore_string(&self) -> String {
        let mut out = String::new();
        out.push_str("# Find-Me Exclusion Rules (.gitignore syntax)\n\n");

        if !self.excluded_folders.is_empty() {
            out.push_str("# Excluded Folders\n");
            for f in &self.excluded_folders {
                out.push_str(&format!("{f}/\n"));
            }
            out.push('\n');
        }

        if !self.excluded_extensions.is_empty() {
            out.push_str("# Excluded File Types\n");
            for ext in &self.excluded_extensions {
                out.push_str(&format!("*.{ext}\n"));
            }
            out.push('\n');
        }

        if !self.excluded_paths.is_empty() {
            out.push_str("# Excluded Paths\n");
            for p in &self.excluded_paths {
                out.push_str(&format!("{p}/\n"));
            }
            out.push('\n');
        }

        out
    }

    /// Legacy compatibility: adds an exclusion pattern string
    pub fn add_exclusion(&mut self, pattern: String) -> bool {
        self.add_smart_rule(&pattern)
    }

    /// Legacy compatibility: removes an exclusion pattern string
    pub fn remove_exclusion(&mut self, pattern: &str) -> bool {
        let p = pattern.trim();
        if p.starts_with('/') || p.starts_with('~') {
            self.remove_excluded_path(p)
        } else if p.starts_with('.') || p.starts_with("*.") {
            self.remove_excluded_extension(p)
        } else {
            self.remove_excluded_folder(p)
        }
    }

    /// Legacy compatibility: restores default exclusion rules
    pub fn add_default_exclusions(&mut self) {
        for f in crate::settings::default_excluded_folders() {
            self.add_excluded_folder(&f);
        }
        for e in crate::settings::default_excluded_extensions() {
            self.add_excluded_extension(&e);
        }
        for p in crate::settings::default_excluded_paths() {
            self.add_excluded_path(&p);
        }
        self.rebuild_flat_exclusions();
    }

    /// Synchronizes state from AppSettings
    pub fn sync_from_settings(&mut self, settings: &crate::settings::AppSettings) {
        self.roots = settings
            .indexed_roots
            .iter()
            .map(|r| IndexedRoot::new(r, 0, 0))
            .collect();
        self.excluded_folders = settings.excluded_folders.clone();
        self.excluded_extensions = settings.excluded_extensions.clone();
        self.excluded_paths = settings.excluded_paths.clone();
        self.rebuild_flat_exclusions();
    }

    /// Synchronizes state into AppSettings
    pub fn sync_to_settings(&self, settings: &mut crate::settings::AppSettings) {
        settings.indexed_roots = self.roots.iter().map(|r| r.display_path()).collect();
        settings.excluded_folders = self.excluded_folders.clone();
        settings.excluded_extensions = self.excluded_extensions.clone();
        settings.excluded_paths = self.excluded_paths.clone();
    }
}
