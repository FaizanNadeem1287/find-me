//! High-performance parallel filesystem crawler powered by jwalk and Rayon

use crate::arena::FileArena;
use crate::error::Result;
use crate::record::FileFlags;
use std::collections::HashSet;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Configuration options for the filesystem crawler
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrawlerConfig {
    /// Root directory paths to crawl
    pub root_paths: Vec<PathBuf>,
    /// Explicit directory paths to exclude
    pub excluded_paths: Vec<PathBuf>,
    /// Directory names to exclude everywhere (e.g. ".git", "node_modules", "target")
    pub excluded_names: Vec<String>,
    /// Whether to follow directory symlinks (default: false)
    pub follow_symlinks: bool,
    /// Maximum directory traversal depth (default: usize::MAX)
    pub max_depth: usize,
    /// Whether to skip hidden files starting with '.' (default: false)
    pub skip_hidden: bool,
    /// Automatically skip virtual Linux kernel filesystems (default: true)
    pub skip_virtual_filesystems: bool,
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
            ],
            follow_symlinks: false,
            max_depth: usize::MAX,
            skip_hidden: false,
            skip_virtual_filesystems: true,
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

        for excl in &self.excluded_paths {
            if path.starts_with(excl) {
                return true;
            }
        }

        if let Some(file_name) = path.file_name() {
            let name_str = file_name.to_string_lossy();
            for excl_name in &self.excluded_names {
                if name_str == *excl_name {
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

        for root in &self.config.root_paths {
            if self.config.is_excluded_path(root) {
                continue;
            }

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

                        let flags = extract_flags(&metadata, is_hidden);
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
    }
}
