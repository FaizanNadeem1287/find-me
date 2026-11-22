//! Comprehensive integration and unit tests for Network Filesystem Indexing & Polling
//!
//! Validates:
//! 1. `/proc/mounts` parsing for NFS, SMB/CIFS, SSHFS, and FUSE remote mounts.
//! 2. `FileRecord` 32-byte layout and `FileFlags::IS_NETWORK` bitflag algebra.
//! 3. `attrib:n` / `attrib:!n` Everything search query evaluation.
//! 4. `Crawler` flagging files on network mounts.
//! 5. `NetworkPoller` periodic synchronization detecting remote creations, modifications, and deletions without inotify.
//! 6. Resilience against disconnected or unmounted shares.

use findme_core::{
    parse_attrib_filter, parse_mountinfo_network_shares, Crawler, CrawlerConfig, FileArena,
    FileFlags, FileRecord, NetworkFsType, NetworkPoller, NetworkShareConfig, QueryParser,
    SearchConfig, SearchEngine, SortColumn, SortDirection,
};
use parking_lot::RwLock;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use tempfile::tempdir;

#[test]
fn test_file_record_size_with_network_flag() {
    assert_eq!(
        std::mem::size_of::<FileRecord>(),
        32,
        "FileRecord must remain exactly 32 bytes with FileFlags::IS_NETWORK"
    );

    let flags = FileFlags::IS_NETWORK;
    assert!(flags.is_network());
    assert!(!flags.is_dir());
    assert!(!flags.is_hidden());

    let combined = FileFlags::IS_DIR | FileFlags::IS_NETWORK | FileFlags::IS_READONLY;
    assert!(combined.is_dir());
    assert!(combined.is_network());
    assert!(combined.is_readonly());
    assert!(!combined.is_symlink());

    let negated = !FileFlags::IS_NETWORK;
    assert!(!negated.is_network());
}

#[test]
fn test_mountinfo_parsing_nfs_smb_sshfs() {
    let mock_mounts = r#"
sysfs /sys sysfs rw,nosuid,nodev,noexec,relatime 0 0
proc /proc proc rw,nosuid,nodev,noexec,relatime 0 0
/dev/nvme0n1p2 / ext4 rw,relatime,errors=remount-ro 0 0
192.168.1.50:/volume1/storage /mnt/nas/media\040library nfs4 rw,relatime,vers=4.2 0 0
//192.168.1.100/work /mnt/windows_share cifs rw,relatime,vers=3.1.1 0 0
user@remote:/home/user /mnt/remote_box fuse.sshfs rw,nosuid,nodev,relatime 0 0
cloud:bucket /mnt/rclone_cloud fuse.rclone rw,nosuid,nodev,relatime 0 0
tmpfs /tmp tmpfs rw,nosuid,nodev 0 0
"#;

    let shares = parse_mountinfo_network_shares(mock_mounts);
    assert_eq!(shares.len(), 4, "Should parse 4 network shares");

    // 1. NFS share with escaped space
    let nfs_share = shares
        .iter()
        .find(|(p, _)| p == Path::new("/mnt/nas/media library"))
        .expect("NFS share with decoded space missing");
    assert_eq!(nfs_share.1, NetworkFsType::Nfs);

    // 2. SMB/CIFS share
    let smb_share = shares
        .iter()
        .find(|(p, _)| p == Path::new("/mnt/windows_share"))
        .expect("SMB share missing");
    assert_eq!(smb_share.1, NetworkFsType::SmbCifs);

    // 3. SSHFS share
    let sshfs_share = shares
        .iter()
        .find(|(p, _)| p == Path::new("/mnt/remote_box"))
        .expect("SSHFS share missing");
    assert_eq!(sshfs_share.1, NetworkFsType::Sshfs);

    // 4. Remote FUSE (Rclone)
    let fuse_share = shares
        .iter()
        .find(|(p, _)| p == Path::new("/mnt/rclone_cloud"))
        .expect("FUSE share missing");
    assert_eq!(fuse_share.1, NetworkFsType::FuseRemote);
}

#[test]
fn test_query_parser_network_attribute_filter() {
    let (req, forb) = parse_attrib_filter("n");
    assert!(req.is_network());
    assert!(!forb.is_network());

    let (req_neg, forb_neg) = parse_attrib_filter("!n");
    assert!(!req_neg.is_network());
    assert!(forb_neg.is_network());

    // Search Engine Integration with attrib:n
    let mut arena = FileArena::new();

    // Local file
    arena
        .add_file(
            Path::new("/mnt/local_report.pdf"),
            FileFlags::NONE,
            1024,
            100,
        )
        .unwrap();
    // Network file
    arena
        .add_file(
            Path::new("/mnt/remote_backup.tar.gz"),
            FileFlags::IS_NETWORK,
            2048,
            200,
        )
        .unwrap();

    // 1. Search with attrib:n (Network only)
    let ast_network = QueryParser::parse("attrib:n");
    let config = SearchConfig {
        max_results: 10,
        sort_column: SortColumn::Name,
        sort_direction: SortDirection::Ascending,
    };

    let results_network = SearchEngine::search(&arena, &ast_network, &config);
    assert_eq!(results_network.len(), 1);
    let name_network = arena.get_name(results_network[0].record_idx).unwrap();
    assert_eq!(name_network, "remote_backup.tar.gz");

    // 2. Search with attrib:!n (Local only)
    let ast_local = QueryParser::parse("attrib:!n");
    let results_local = SearchEngine::search(&arena, &ast_local, &config);
    assert_eq!(results_local.len(), 1);
    let name_local = arena.get_name(results_local[0].record_idx).unwrap();
    assert_eq!(name_local, "local_report.pdf");
}

#[test]
fn test_crawler_network_share_flagging() {
    let dir = tempdir().expect("tempdir failed");
    let share_path = dir.path().to_path_buf();

    let file1 = share_path.join("smb_file1.txt");
    let mut f1 = File::create(&file1).unwrap();
    f1.write_all(b"smb data 1").unwrap();
    f1.sync_all().unwrap();
    drop(f1);

    let mut arena = FileArena::new();
    let config =
        CrawlerConfig::with_roots(vec![share_path.clone()]).add_network_share(share_path.clone());
    let crawler = Crawler::new(config);

    let stats = crawler.crawl(&mut arena).expect("crawl failed");
    assert!(stats.files_indexed >= 1);

    // Verify indexed record has FileFlags::IS_NETWORK set
    let mut found_network_file = false;
    for i in 0..arena.len() {
        if let Some(record) = arena.get(i) {
            if let Some(name) = arena.pool().get(record.name_offset, record.name_len) {
                if name == "smb_file1.txt" {
                    assert!(
                        record.flags.is_network(),
                        "smb_file1.txt must have IS_NETWORK flag"
                    );
                    found_network_file = true;
                }
            }
        }
    }
    assert!(found_network_file, "smb_file1.txt must be indexed");
}

#[test]
fn test_network_poller_detects_remote_changes() {
    let dir = tempdir().expect("tempdir failed");
    let share_path = dir.path().to_path_buf();

    // 1. Initial files on share
    let file1 = share_path.join("document_alpha.doc");
    let file2 = share_path.join("document_beta.doc");

    let mut f1 = File::create(&file1).unwrap();
    f1.write_all(b"Version 1").unwrap();
    f1.sync_all().unwrap();
    drop(f1);

    let mut f2 = File::create(&file2).unwrap();
    f2.write_all(b"Will be deleted").unwrap();
    f2.sync_all().unwrap();
    drop(f2);

    let arena = Arc::new(RwLock::new(FileArena::new()));
    let config =
        CrawlerConfig::with_roots(vec![share_path.clone()]).add_network_share(share_path.clone());
    let crawler = Crawler::new(config);
    crawler.crawl(&mut arena.write()).expect("Initial crawl");

    let poller = NetworkPoller::new();
    poller.add_share(NetworkShareConfig::new(&share_path, NetworkFsType::SmbCifs));

    // 2. Simulate remote server changes without inotify:
    // a. Add remote file
    let file3 = share_path.join("remote_created.txt");
    let mut f3 = File::create(&file3).unwrap();
    f3.write_all(b"Created remotely").unwrap();
    f3.sync_all().unwrap();
    drop(f3);

    // b. Modify file1
    let mut f1_mod = File::create(&file1).unwrap();
    f1_mod.write_all(b"Version 2 updated content").unwrap();
    f1_mod.sync_all().unwrap();
    drop(f1_mod);

    // c. Delete file2
    fs::remove_file(&file2).expect("remove file2");

    // 3. Trigger periodic poll
    let stats = poller.poll_once(&arena).expect("poll_once failed");
    assert_eq!(stats.files_added, 1, "Should detect 1 added file");
    assert_eq!(stats.files_removed, 1, "Should detect 1 deleted file");
    assert!(stats.files_updated >= 1, "Should detect modified file");

    // 4. Verify arena state
    let read_arena = arena.read();
    let mut names = Vec::new();
    for i in 0..read_arena.len() {
        if let Some(record) = read_arena.get(i) {
            if !record.flags.is_deleted() {
                if let Some(name) = read_arena.pool().get(record.name_offset, record.name_len) {
                    names.push(name.to_string());
                }
                assert!(
                    record.flags.is_network(),
                    "All share files must have IS_NETWORK"
                );
            }
        }
    }

    assert!(names.contains(&"remote_created.txt".to_string()));
    assert!(names.contains(&"document_alpha.doc".to_string()));
    assert!(!names.contains(&"document_beta.doc".to_string()));
}

#[test]
fn test_network_poller_offline_resilience() {
    let poller = NetworkPoller::new();
    poller.add_share(NetworkShareConfig::new(
        "/mnt/nonexistent_mock_share_9999",
        NetworkFsType::Nfs,
    ));

    let arena = RwLock::new(FileArena::new());
    let stats = poller
        .poll_once(&arena)
        .expect("Polling offline share should not fail");
    assert_eq!(stats.files_added, 0);
    assert_eq!(stats.files_updated, 0);
    assert_eq!(stats.files_removed, 0);
}
