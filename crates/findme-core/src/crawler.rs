//! High-performance parallel filesystem crawler powered by jwalk and Rayon

use crate::arena::FileArena;
use crate::error::Result;
use crate::record::FileFlags;
use std::collections::HashSet;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Matches a simple wildcard glob against a target string (e.g. `*.pyc`, `*~`, `test-*`, `*cache*`)
pub fn glob_matches(pattern: &str, target: &str) -> bool {
    let p = pattern.trim();
    if p.is_empty() {
        return false;
    }
    if p == "*" {
        return true;
    }
    if p.starts_with('*') && p.ends_with('*') && p.len() > 2 {
        let needle = &p[1..p.len() - 1];
        return target.contains(needle);
    }
    if let Some(suffix) = p.strip_prefix('*') {
        return target.ends_with(suffix);
    }
    if let Some(prefix) = p.strip_suffix('*') {
        return target.starts_with(prefix);
    }
    p.eq_ignore_ascii_case(target)
}

/// Expands leading `~` into user's home directory
pub fn expand_tilde(p: &Path) -> PathBuf {
    if let Ok(stripped) = p.strip_prefix("~") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(stripped);
        }
    }
    p.to_path_buf()
}

/// Configuration options for the filesystem crawler
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrawlerConfig {
    /// Root directory paths to crawl
    pub root_paths: Vec<PathBuf>,
    /// Explicit directory paths to exclude
    pub excluded_paths: Vec<PathBuf>,
    /// Directory names to exclude everywhere (e.g. ".git", "node_modules", "target")
    pub excluded_names: Vec<String>,
    /// Explicit file extensions to exclude everywhere (e.g. "pyc", "tmp", "cache")
    pub excluded_extensions: Vec<String>,
    /// Wildcard glob patterns to exclude everywhere (e.g. "*~", ".*.swp")
    pub excluded_globs: Vec<String>,
    /// Whether to follow directory symlinks (default: false)
    pub follow_symlinks: bool,
    /// Maximum directory traversal depth (default: usize::MAX)
    pub max_depth: usize,
    /// Whether to skip hidden files starting with '.' (default: false)
    pub skip_hidden: bool,
    /// Automatically skip virtual Linux kernel filesystems (default: true)
    pub skip_virtual_filesystems: bool,
    /// Explicitly identified network share roots
    pub network_shares: Vec<PathBuf>,
    /// Whether to detect and mark files residing on network shares (default: true)
    pub mark_network_shares: bool,
}

impl Default for CrawlerConfig {
    fn default() -> Self {
        Self {
            root_paths: Vec::new(),
            excluded_paths: Vec::new(),
            excluded_names: vec![
                ".git".into(),
                ".svn".into(),
                ".hg".into(),
                "node_modules".into(),
                "target".into(),
                "__pycache__".into(),
                ".venv".into(),
                ".cargo".into(),
            ],
            excluded_extensions: vec![
                "pyc".into(),
                "pyo".into(),
                "tmp".into(),
                "swp".into(),
                "bak".into(),
            ],
            excluded_globs: Vec::new(),
            follow_symlinks: false,
            max_depth: usize::MAX,
            skip_hidden: false,
            skip_virtual_filesystems: true,
            network_shares: Vec::new(),
            mark_network_shares: true,
        }
    }
}

impl CrawlerConfig {
    /// Creates a new default crawler configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a crawler configuration with specified root paths
    pub fn with_roots(roots: Vec<PathBuf>) -> Self {
        Self {
            root_paths: roots,
            ..Default::default()
        }
    }

    /// Adds a root path to index
    pub fn add_root<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.root_paths.push(path.into());
        self
    }

    /// Adds an explicit path to exclude from indexing
    pub fn add_exclusion<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.excluded_paths.push(path.into());
        self
    }

    /// Adds a folder name to skip wherever encountered
    pub fn add_excluded_name<S: Into<String>>(mut self, name: S) -> Self {
        self.excluded_names.push(name.into());
        self
    }

    /// Adds an explicit file extension to exclude everywhere (e.g. "pyc", "tmp")
    pub fn add_excluded_extension<S: Into<String>>(mut self, ext: S) -> Self {
        let cleaned = ext
            .into()
            .trim()
            .trim_start_matches('.')
            .to_ascii_lowercase();
        if !cleaned.is_empty() && !self.excluded_extensions.contains(&cleaned) {
            self.excluded_extensions.push(cleaned);
        }
        self
    }

    /// Adds a wildcard glob pattern to exclude everywhere (e.g. "*~", ".*.swp")
    pub fn add_excluded_glob<S: Into<String>>(mut self, glob: S) -> Self {
        let pattern = glob.into().trim().to_string();
        if !pattern.is_empty() && !self.excluded_globs.contains(&pattern) {
            self.excluded_globs.push(pattern);
        }
        self
    }

    /// Sets whether to follow directory symlinks
    pub fn follow_symlinks(mut self, follow: bool) -> Self {
        self.follow_symlinks = follow;
        self
    }

    /// Sets the maximum directory depth
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Sets whether to skip hidden files starting with '.'
    pub fn skip_hidden(mut self, skip: bool) -> Self {
        self.skip_hidden = skip;
        self
    }

    /// Sets whether to skip virtual Linux kernel mounts
    pub fn skip_virtual_filesystems(mut self, skip: bool) -> Self {
        self.skip_virtual_filesystems = skip;
        self
    }

    /// Adds an explicit network share root path
    pub fn add_network_share<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.network_shares.push(path.into());
        self
    }

    /// Sets whether to detect and mark network files
    pub fn mark_network_shares(mut self, mark: bool) -> Self {
        self.mark_network_shares = mark;
        self
    }

    /// Checks if a path matches virtual filesystem or user exclusion rules
    pub fn is_excluded_path(&self, path: &Path) -> bool {
        // Explicitly configured roots are never excluded
        for root in &self.root_paths {
            if path == root {
                return false;
            }
        }

        if self.skip_virtual_filesystems {
            const VIRTUAL_MOUNTS: &[&str] = &[
                "/proc",
                "/sys",
                "/dev",
                "/run",
                "/sys/fs/cgroup",
                "/var/lib/docker",
                "/var/lib/containerd",
            ];
            for &vm in VIRTUAL_MOUNTS {
                let vm_path = Path::new(vm);
                let root_inside_vm = self.root_paths.iter().any(|r| r.starts_with(vm_path));
                if !root_inside_vm && path.starts_with(vm_path) {
                    return true;
                }
            }
        }

        // 1. Explicit path prefix matching with ~ expansion
        for excl in &self.excluded_paths {
            let resolved = expand_tilde(excl);
            if path.starts_with(&resolved) {
                return true;
            }
        }

        // 2. Ancestor component matching (checks all folder names in the entire path)
        for component in path.components() {
            if let std::path::Component::Normal(os_str) = component {
                let comp_str = os_str.to_string_lossy();
                for excl_name in &self.excluded_names {
                    if comp_str.eq_ignore_ascii_case(excl_name) {
                        return true;
                    }
                }
            }
        }

        // 3. File extension matching
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_ascii_lowercase();
            for excl_ext in &self.excluded_extensions {
                if ext_str == *excl_ext {
                    return true;
                }
            }
            // Compatibility: support "*.ext" or ".ext" inside excluded_names
            for excl_name in &self.excluded_names {
                let cleaned = excl_name.trim_start_matches('*').trim_start_matches('.');
                if !cleaned.is_empty()
                    && (excl_name.starts_with('*') || excl_name.starts_with('.'))
                    && ext_str == cleaned.to_ascii_lowercase()
                {
                    return true;
                }
            }
        }

        // 4. Wildcard glob matching on filename
        if let Some(file_name) = path.file_name() {
            let name_str = file_name.to_string_lossy();
            for glob in &self.excluded_globs {
                if glob_matches(glob, &name_str) {
                    return true;
                }
            }
            for excl_name in &self.excluded_names {
                if excl_name.contains('*') && glob_matches(excl_name, &name_str) {
                    return true;
                }
            }
        }

        false
    }
}

/// Real-time progress update emitted during crawler traversal
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CrawlerProgress {
    pub files_indexed: u64,
    pub dirs_scanned: u64,
    pub current_directory: Option<PathBuf>,
}

/// Final summary statistics produced after crawling completes
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CrawlerStats {
    pub files_indexed: u64,
    pub dirs_scanned: u64,
    pub errors_encountered: u64,
    pub elapsed: Duration,
    pub throughput_files_per_sec: f64,
}

/// Cycle detector tracking visited filesystem inode and device pairs
#[derive(Default)]
pub struct CycleDetector {
    visited: HashSet<(u64, u64)>,
}

impl CycleDetector {
    pub fn new() -> Self {
        Self {
            visited: HashSet::with_capacity(4096),
        }
    }

    /// Checks if a directory target has been visited. If not, records it.
    /// Returns `true` if a cycle is detected.
    pub fn check_or_record(&mut self, meta: &std::fs::Metadata) -> bool {
        let key = (meta.dev(), meta.ino());
        !self.visited.insert(key)
    }

    pub fn len(&self) -> usize {
        self.visited.len()
    }

    pub fn is_empty(&self) -> bool {
        self.visited.is_empty()
    }
}

/// Extracts `FileFlags` bitmask from POSIX metadata
pub fn extract_flags(meta: &std::fs::Metadata, is_hidden: bool) -> FileFlags {
    let mut flags = FileFlags::NONE;
    let mode = meta.mode();

    if meta.is_dir() {
        flags.insert(FileFlags::IS_DIR);
    }
    if meta.is_symlink() {
        flags.insert(FileFlags::IS_SYMLINK);
    }
    if is_hidden {
        flags.insert(FileFlags::IS_HIDDEN);
    }
    if (mode & 0o111) != 0 {
        flags.insert(FileFlags::IS_EXECUTABLE);
    }
    if (mode & 0o200) == 0 {
        flags.insert(FileFlags::IS_READONLY);
    }

    flags
}

/// Ultra-fast parallel filesystem crawler
pub struct Crawler {
    config: CrawlerConfig,
}

impl Crawler {
    /// Creates a crawler with the given configuration
    pub fn new(config: CrawlerConfig) -> Self {
        Self { config }
    }

    /// Crawls configured roots directly into the provided `FileArena`
    pub fn crawl(&self, arena: &mut FileArena) -> Result<CrawlerStats> {
        self.crawl_with_progress(arena, |_| {})
    }

    /// Crawls configured roots directly into `FileArena` emitting progress batches
    pub fn crawl_with_progress<F>(
        &self,
        arena: &mut FileArena,
        mut progress_cb: F,
    ) -> Result<CrawlerStats>
    where
        F: FnMut(CrawlerProgress),
    {
        let start_time = Instant::now();
        let mut total_files = 0u64;
        let mut total_dirs = 0u64;
        let mut errors_encountered = 0u64;
        let mut cycle_detector = CycleDetector::new();

        // Pre-resolve network mounts once so we avoid querying /proc/mounts per file
        let mut network_mounts = self.config.network_shares.clone();
        if self.config.mark_network_shares {
            for (mount, _) in crate::network_fs::get_active_network_mounts() {
                if !network_mounts.contains(&mount) {
                    network_mounts.push(mount);
                }
            }
        }

        // Deduplicate roots: if a root is a subfolder of another root in the list,
        // it is already traversed by the parent root's walk, avoiding duplicate arena records.
        let mut effective_roots = Vec::new();
        for root in &self.config.root_paths {
            if self.config.is_excluded_path(root) {
                continue;
            }
            let clean_root = PathBuf::from(root.to_string_lossy().trim_end_matches('/'));
            let is_covered = self.config.root_paths.iter().any(|other| {
                if other == root || self.config.is_excluded_path(other) {
                    return false;
                }
                let clean_other = PathBuf::from(other.to_string_lossy().trim_end_matches('/'));
                clean_root.starts_with(&clean_other)
            });
            if !is_covered && !effective_roots.contains(&clean_root) {
                effective_roots.push(clean_root);
            }
        }

        for root in &effective_roots {
            let mut walk = jwalk::WalkDir::new(root)
                .skip_hidden(self.config.skip_hidden)
                .follow_links(self.config.follow_symlinks)
                .max_depth(self.config.max_depth);

            let config_clone = self.config.clone();
            walk = walk.process_read_dir(move |_depth, dir_path, _state, children| {
                if config_clone.is_excluded_path(dir_path) {
                    children.clear();
                    return;
                }

                children.retain(|res| match res {
                    Ok(entry) => {
                        let path = entry.path();
                        !config_clone.is_excluded_path(&path)
                    }
                    Err(_) => true,
                });
            });

            let mut batch_count = 0u64;

            for entry_result in walk {
                match entry_result {
                    Ok(entry) => {
                        let path = entry.path();

                        if entry.read_children_error.is_some() {
                            errors_encountered += 1;
                            tracing::debug!(
                                "Error reading children for {:?}: {:?}",
                                path,
                                entry.read_children_error
                            );
                        }

                        // Symlink cycle check when following directory symlinks
                        if self.config.follow_symlinks && entry.file_type().is_symlink() {
                            if let Ok(meta) = path.metadata() {
                                if meta.is_dir() && cycle_detector.check_or_record(&meta) {
                                    continue;
                                }
                            }
                        }

                        let metadata = match entry.metadata() {
                            Ok(m) => m,
                            Err(e) => {
                                errors_encountered += 1;
                                tracing::debug!("Skipping inaccessible path {:?}: {}", path, e);
                                continue;
                            }
                        };

                        let is_hidden = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .is_some_and(|s| s.starts_with('.') && s != "." && s != "..");

                        let mut flags = extract_flags(&metadata, is_hidden);
                        if self.config.mark_network_shares
                            && network_mounts.iter().any(|ns| path.starts_with(ns))
                        {
                            flags |= FileFlags::IS_NETWORK;
                        }

                        let size_bytes = if metadata.is_dir() { 0 } else { metadata.len() };
                        let mtime_secs = metadata
                            .modified()
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map_or(0, |d| d.as_secs());

                        if flags.is_dir() {
                            total_dirs += 1;
                        } else {
                            total_files += 1;
                        }

                        if let Err(e) = arena.add_file(&path, flags, size_bytes, mtime_secs) {
                            tracing::warn!("Failed to add file {:?} to arena: {}", path, e);
                            errors_encountered += 1;
                        }

                        batch_count += 1;
                        if batch_count >= 10_000 {
                            batch_count = 0;
                            progress_cb(CrawlerProgress {
                                files_indexed: total_files,
                                dirs_scanned: total_dirs,
                                current_directory: Some(path),
                            });
                        }
                    }
                    Err(e) => {
                        errors_encountered += 1;
                        tracing::debug!("Filesystem walk error: {}", e);
                    }
                }
            }
        }

        let elapsed = start_time.elapsed();
        let secs = elapsed.as_secs_f64();
        let throughput = if secs > 0.0 {
            total_files as f64 / secs
        } else {
            total_files as f64
        };

        progress_cb(CrawlerProgress {
            files_indexed: total_files,
            dirs_scanned: total_dirs,
            current_directory: None,
        });

        Ok(CrawlerStats {
            files_indexed: total_files,
            dirs_scanned: total_dirs,
            errors_encountered,
            elapsed,
            throughput_files_per_sec: throughput,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crawler_config_builder() {
        let config = CrawlerConfig::new()
            .add_root("/home/user")
            .add_exclusion("/home/user/cache")
            .add_excluded_name(".cache")
            .follow_symlinks(true)
            .max_depth(5)
            .skip_hidden(true)
            .skip_virtual_filesystems(true);

        assert_eq!(config.root_paths, vec![PathBuf::from("/home/user")]);
        assert_eq!(
            config.excluded_paths,
            vec![PathBuf::from("/home/user/cache")]
        );
        assert!(config.excluded_names.contains(&".cache".to_string()));
        assert!(config.follow_symlinks);
        assert_eq!(config.max_depth, 5);
        assert!(config.skip_hidden);
        assert!(config.skip_virtual_filesystems);
    }

    #[test]
    fn test_virtual_filesystem_exclusion() {
        let config = CrawlerConfig::new().skip_virtual_filesystems(true);
        assert!(config.is_excluded_path(Path::new("/proc/1/cmdline")));
        assert!(config.is_excluded_path(Path::new("/sys/class/net")));
        assert!(config.is_excluded_path(Path::new("/dev/sda")));
        assert!(config.is_excluded_path(Path::new("/run/user/1000")));
        assert!(config.is_excluded_path(Path::new("/var/lib/docker/overlay2")));

        // Normal paths are not excluded
        assert!(!config.is_excluded_path(Path::new("/home/user/sysadmin.txt")));
        assert!(!config.is_excluded_path(Path::new("/usr/bin/find-me")));
    }

    #[test]
    fn test_user_name_exclusions() {
        let config = CrawlerConfig::new();
        assert!(config.is_excluded_path(Path::new("/home/user/repo/.git")));
        assert!(config.is_excluded_path(Path::new("/home/user/app/node_modules")));
        assert!(config.is_excluded_path(Path::new("/home/user/rust/target")));
        // Ancestor folder matching
        assert!(
            config.is_excluded_path(Path::new("/home/user/app/node_modules/lodash/package.json"))
        );
        assert!(config.is_excluded_path(Path::new("/home/user/repo/.git/objects/pack/pack.idx")));
    }

    #[test]
    fn test_extension_and_glob_exclusions() {
        let config = CrawlerConfig::new()
            .add_excluded_extension("pyc")
            .add_excluded_extension(".tmp")
            .add_excluded_glob("*~")
            .add_excluded_glob("*.bak");

        assert!(config.is_excluded_path(Path::new("/home/user/src/main.pyc")));
        assert!(config.is_excluded_path(Path::new("/home/user/src/main.PYC")));
        assert!(config.is_excluded_path(Path::new("/home/user/data/temp.tmp")));
        assert!(config.is_excluded_path(Path::new("/home/user/doc/draft.txt~")));
        assert!(config.is_excluded_path(Path::new("/home/user/doc/file.bak")));

        // Non-excluded extensions
        assert!(!config.is_excluded_path(Path::new("/home/user/src/main.py")));
        assert!(!config.is_excluded_path(Path::new("/home/user/doc/draft.txt")));
    }

    #[test]
    fn test_glob_matches() {
        assert!(glob_matches("*.pyc", "app.pyc"));
        assert!(glob_matches("*.pyc", "app.cpython-310.pyc"));
        assert!(!glob_matches("*.pyc", "app.py"));
        assert!(glob_matches("*~", "backup.txt~"));
        assert!(glob_matches("test-*", "test-runner"));
        assert!(!glob_matches("test-*", "production-runner"));
        assert!(glob_matches("*cache*", "mycachefolder"));
    }
}
