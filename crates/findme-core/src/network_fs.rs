//! Network filesystem indexing, mount detection, and periodic polling
//!
//! Provides support for indexing remote network shares (NFS, SMB/CIFS, SSHFS, FUSE)
//! with automatic `/proc/mounts` detection, `FileFlags::IS_NETWORK` tagging, and
//! periodic polling synchronization to detect changes where inotify cannot listen.

use crate::arena::FileArena;
use crate::error::Result;
use crate::record::FileFlags;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{spawn, JoinHandle};
use std::time::{Duration, Instant};

/// Known network filesystem types supported by Find-Me
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NetworkFsType {
    /// Network File System (NFS v3/v4)
    Nfs,
    /// Server Message Block / Common Internet File System (SMB/CIFS)
    SmbCifs,
    /// Secure Shell Filesystem (SSHFS / FUSE)
    Sshfs,
    /// Remote FUSE-based mounts (WebDAV, Rclone, AFP)
    FuseRemote,
    /// Generic or other remote network mounts
    GenericNetwork,
}

impl NetworkFsType {
    /// Returns the human-readable display name of the network filesystem
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Nfs => "NFS",
            Self::SmbCifs => "SMB/CIFS",
            Self::Sshfs => "SSHFS",
            Self::FuseRemote => "Remote FUSE",
            Self::GenericNetwork => "Network Share",
        }
    }

    /// Identifies a network filesystem type from a POSIX mount filesystem name
    pub fn from_fs_name(name: &str) -> Option<Self> {
        let lower = name.to_ascii_lowercase();
        match lower.as_str() {
            "nfs" | "nfs4" | "nfsd" => Some(Self::Nfs),
            "cifs" | "smbfs" | "smb3" => Some(Self::SmbCifs),
            "fuse.sshfs" | "sshfs" => Some(Self::Sshfs),
            "fuse.rclone" | "davfs" | "davfs2" | "afpfs" => Some(Self::FuseRemote),
            _ if lower.starts_with("nfs") => Some(Self::Nfs),
            _ if lower.starts_with("cifs") || lower.starts_with("smb") => Some(Self::SmbCifs),
            _ => None,
        }
    }
}

/// Configuration for indexing a remote network share
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkShareConfig {
    /// Mount point path of the network share
    pub path: PathBuf,
    /// Detected or configured network filesystem type
    pub fs_type: NetworkFsType,
    /// Background periodic polling interval to detect remote server changes
    pub poll_interval: Duration,
    /// Network read timeout to prevent slow or offline shares from blocking
    pub timeout: Duration,
    /// Whether indexing and polling is enabled for this share
    pub enabled: bool,
}

impl NetworkShareConfig {
    /// Creates a new network share configuration with sensible defaults
    pub fn new<P: Into<PathBuf>>(path: P, fs_type: NetworkFsType) -> Self {
        Self {
            path: path.into(),
            fs_type,
            poll_interval: Duration::from_secs(60),
            timeout: Duration::from_secs(10),
            enabled: true,
        }
    }

    /// Sets custom periodic polling interval
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// Sets custom network read timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// Parses POSIX `/proc/mounts` or `/proc/self/mountinfo` text into network mountpoints
pub fn parse_mountinfo_network_shares(mounts_content: &str) -> Vec<(PathBuf, NetworkFsType)> {
    let mut shares = Vec::new();

    for line in mounts_content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 3 {
            // Standard /proc/mounts format: <spec> <file> <vfstype> <mntops> <freq> <passno>
            let mount_point_raw = parts[1];
            let fs_type_raw = parts[2];

            if let Some(fs_type) = NetworkFsType::from_fs_name(fs_type_raw) {
                // Decode octal escape sequences (e.g., \040 -> space)
                let decoded_path = decode_octal_escapes(mount_point_raw);
                shares.push((PathBuf::from(decoded_path), fs_type));
            }
        }
    }

    // Sort by path length descending so longer mount prefixes match first
    shares.sort_by_key(|a| std::cmp::Reverse(a.0.as_os_str().len()));
    shares
}

/// Decodes octal escape sequences commonly found in `/proc/mounts` (e.g. `\040` for space)
fn decode_octal_escapes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            let mut octal = String::new();
            for _ in 0..3 {
                if let Some(&next) = chars.peek() {
                    if ('0'..='7').contains(&next) {
                        octal.push(chars.next().unwrap_or('0'));
                    } else {
                        break;
                    }
                }
            }
            if octal.len() == 3 {
                if let Ok(byte) = u8::from_str_radix(&octal, 8) {
                    result.push(byte as char);
                    continue;
                }
            }
            result.push('\\');
            result.push_str(&octal);
        } else {
            result.push(c);
        }
    }

    result
}

/// Reads current system mount points and returns detected network shares
pub fn get_active_network_mounts() -> Vec<(PathBuf, NetworkFsType)> {
    let paths_to_try = ["/proc/mounts", "/proc/self/mounts", "/etc/mtab"];
    for p in &paths_to_try {
        if let Ok(content) = fs::read_to_string(p) {
            let shares = parse_mountinfo_network_shares(&content);
            if !shares.is_empty() {
                return shares;
            }
        }
    }
    Vec::new()
}

/// Detects whether a specific path is mounted on a network filesystem
pub fn detect_network_filesystem(path: &Path) -> Option<NetworkFsType> {
    let active_shares = get_active_network_mounts();
    for (mount_root, fs_type) in active_shares {
        if path.starts_with(&mount_root) {
            return Some(fs_type);
        }
    }
    None
}

/// Returns true if a path resides within a known network filesystem mount
pub fn is_network_path(path: &Path) -> bool {
    detect_network_filesystem(path).is_some()
}

/// Results summary from a network share polling cycle
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PollStats {
    /// Number of new files added during this poll
    pub files_added: usize,
    /// Number of files whose size or mtime was updated
    pub files_updated: usize,
    /// Number of files removed because they no longer exist on the share
    pub files_removed: usize,
    /// Total duration elapsed during the polling cycle
    pub elapsed_ms: u64,
}

/// Periodic polling monitor for network shares where inotify does not emit remote events
pub struct NetworkPoller {
    shares: Arc<parking_lot::RwLock<Vec<NetworkShareConfig>>>,
    is_running: Arc<AtomicBool>,
    worker_handle: Option<JoinHandle<()>>,
}

impl NetworkPoller {
    /// Creates a new network share poller
    pub fn new() -> Self {
        Self {
            shares: Arc::new(parking_lot::RwLock::new(Vec::new())),
            is_running: Arc::new(AtomicBool::new(false)),
            worker_handle: None,
        }
    }

    /// Adds a network share to be monitored by the poller
    pub fn add_share(&self, share: NetworkShareConfig) {
        self.shares.write().push(share);
    }

    /// Removes a network share by path
    pub fn remove_share(&self, path: &Path) {
        self.shares.write().retain(|s| s.path != path);
    }

    /// Returns a list of configured network shares
    pub fn shares(&self) -> Vec<NetworkShareConfig> {
        self.shares.read().clone()
    }

    /// Executes a single synchronization poll across all enabled network shares
    pub fn poll_once(&self, arena: &parking_lot::RwLock<FileArena>) -> Result<PollStats> {
        let start = Instant::now();
        let shares = self.shares.read().clone();
        let mut total_stats = PollStats::default();

        for share in shares {
            if !share.enabled || !share.path.exists() {
                continue;
            }

            let share_stats = Self::sync_share(&share, arena)?;
            total_stats.files_added += share_stats.files_added;
            total_stats.files_updated += share_stats.files_updated;
            total_stats.files_removed += share_stats.files_removed;
        }

        total_stats.elapsed_ms = start.elapsed().as_millis() as u64;
        Ok(total_stats)
    }

    /// Synchronizes a single network share with `FileArena`
    fn sync_share(
        share: &NetworkShareConfig,
        arena_lock: &parking_lot::RwLock<FileArena>,
    ) -> Result<PollStats> {
        let start = Instant::now();
        let mut stats = PollStats::default();

        // 1. Collect current items from disk
        let mut disk_entries: HashMap<PathBuf, (u64, u64, bool)> = HashMap::new();
        let mut stack = vec![share.path.clone()];

        while let Some(current_dir) = stack.pop() {
            let read_dir = match fs::read_dir(&current_dir) {
                Ok(rd) => rd,
                Err(e) => {
                    tracing::debug!("Network share read error for {:?}: {}", current_dir, e);
                    continue;
                }
            };

            for entry_res in read_dir {
                let entry = match entry_res {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                let path = entry.path();
                let meta = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                let is_dir = meta.is_dir();
                let size = if is_dir { 0 } else { meta.len() };
                let mtime = meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map_or(0, |d| d.as_secs());

                if is_dir {
                    stack.push(path.clone());
                }
                disk_entries.insert(path, (size, mtime, is_dir));
            }
        }

        // 2. Read current arena records under this share
        let mut arena = arena_lock.write();
        let mut existing_paths: HashSet<PathBuf> = HashSet::new();

        for i in 0..arena.len() {
            if let Some(record) = arena.get(i) {
                if !record.flags.is_deleted() {
                    if let Some(path) = arena.reconstruct_path(record) {
                        if path == share.path {
                            continue;
                        }
                        if path.starts_with(&share.path) {
                            existing_paths.insert(path.clone());

                            // Check if file still exists on disk
                            if let Some(&(new_size, new_mtime, _is_dir)) = disk_entries.get(&path) {
                                if record.size_bytes != new_size || record.mtime_secs != new_mtime {
                                    let mut new_flags = record.flags;
                                    new_flags |= FileFlags::IS_NETWORK;
                                    arena.update_file_metadata(
                                        &path, new_size, new_mtime, new_flags,
                                    );
                                    stats.files_updated += 1;
                                }
                            } else {
                                // File was deleted on remote share
                                arena.remove_file(&path);
                                stats.files_removed += 1;
                            }
                        }
                    }
                }
            }
        }

        // 3. Add new files from disk not yet in arena
        for (disk_path, (size, mtime, is_dir)) in disk_entries {
            if !existing_paths.contains(&disk_path) {
                let file_name = disk_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                let is_hidden = file_name.starts_with('.') && file_name != "." && file_name != "..";

                let mut flags = FileFlags::IS_NETWORK;
                if is_dir {
                    flags |= FileFlags::IS_DIR;
                }
                if is_hidden {
                    flags |= FileFlags::IS_HIDDEN;
                }

                if arena.add_file(&disk_path, flags, size, mtime).is_ok() {
                    stats.files_added += 1;
                }
            }
        }

        stats.elapsed_ms = start.elapsed().as_millis() as u64;
        Ok(stats)
    }

    /// Spawns a background thread that periodically polls network shares
    pub fn start(&mut self, arena: Arc<parking_lot::RwLock<FileArena>>, interval: Duration) {
        if self.is_running.swap(true, Ordering::SeqCst) {
            return;
        }

        let is_running = Arc::clone(&self.is_running);
        let shares_arc = Arc::clone(&self.shares);

        let handle = spawn(move || {
            while is_running.load(Ordering::SeqCst) {
                std::thread::sleep(interval);
                if !is_running.load(Ordering::SeqCst) {
                    break;
                }

                let shares = shares_arc.read().clone();
                for share in shares {
                    if share.enabled && share.path.exists() {
                        let _ = Self::sync_share(&share, &arena);
                    }
                }
            }
        });

        self.worker_handle = Some(handle);
    }

    /// Stops the periodic polling worker thread
    pub fn stop(&mut self) {
        self.is_running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Default for NetworkPoller {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for NetworkPoller {
    fn drop(&mut self) {
        self.stop();
    }
}
