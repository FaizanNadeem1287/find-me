//! Application preferences, search defaults, and persistent settings

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Returns default indexed directory roots
pub fn default_indexed_roots() -> Vec<String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
    vec![home]
}

/// Returns default folder names to exclude everywhere
pub fn default_excluded_folders() -> Vec<String> {
    vec![
        "node_modules".to_string(),
        "target".to_string(),
        ".git".to_string(),
        "__pycache__".to_string(),
        ".venv".to_string(),
        ".cargo".to_string(),
        "build".to_string(),
        "dist".to_string(),
    ]
}

/// Returns default file extensions to exclude everywhere
pub fn default_excluded_extensions() -> Vec<String> {
    vec![
        "pyc".to_string(),
        "pyo".to_string(),
        "tmp".to_string(),
        "cache".to_string(),
        "log".to_string(),
    ]
}

/// Returns default directory paths to exclude
pub fn default_excluded_paths() -> Vec<String> {
    vec!["~/.cache".to_string(), "~/.local/share/Trash".to_string()]
}

/// Comprehensive application preferences and search defaults
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettings {
    // General
    pub autostart: bool,
    pub close_to_tray: bool,

    // Search Defaults
    pub default_regex: bool,
    pub default_case_sensitive: bool,
    pub max_results: usize,

    // Privacy & Exclusions
    pub exclude_hidden_files: bool,
    pub exclude_system_paths: bool,

    // Index Roots & Custom Exclusions (Phase 14)
    #[serde(default = "default_indexed_roots")]
    pub indexed_roots: Vec<String>,

    #[serde(default = "default_excluded_folders")]
    pub excluded_folders: Vec<String>,

    #[serde(default = "default_excluded_extensions")]
    pub excluded_extensions: Vec<String>,

    #[serde(default = "default_excluded_paths")]
    pub excluded_paths: Vec<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            autostart: false,
            close_to_tray: true,
            default_regex: false,
            default_case_sensitive: false,
            max_results: 1000,
            exclude_hidden_files: true,
            exclude_system_paths: true,
            indexed_roots: default_indexed_roots(),
            excluded_folders: default_excluded_folders(),
            excluded_extensions: default_excluded_extensions(),
            excluded_paths: default_excluded_paths(),
        }
    }
}

impl AppSettings {
    /// Returns the configuration directory for Find-Me
    pub fn config_dir() -> PathBuf {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            if !xdg.trim().is_empty() {
                return PathBuf::from(xdg).join("find-me");
            }
        }

        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config").join("find-me")
        } else {
            PathBuf::from("/tmp/find-me-config")
        }
    }

    /// Returns the standard settings configuration file path
    pub fn config_path() -> PathBuf {
        Self::config_dir().join("settings.json")
    }

    /// Loads settings from standard default path, or returns default on failure
    pub fn load_default() -> Self {
        Self::load_from_path(&Self::config_path()).unwrap_or_default()
    }

    /// Saves settings to standard default path
    pub fn save_default(&self) -> Result<(), String> {
        self.save_to_path(&Self::config_path())
    }

    /// Loads settings from a JSON file path, falling back to default on error
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    /// Saves settings to a JSON file path, creating parent directories if needed
    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }
}
