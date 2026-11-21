//! Comprehensive integration and edge-case test suite for the parallel filesystem crawler

use findme_core::{Crawler, CrawlerConfig, FileArena};
use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs::{symlink, PermissionsExt};
use tempfile::tempdir;

#[test]
fn test_crawler_basic_traversal() {
    let tmp = tempdir().expect("tempdir creation failed");
    let base = tmp.path();

    // Create realistic folder structure
    let dir_a = base.join("dir_a");
    let dir_b = base.join("dir_b");
    let dir_sub = dir_a.join("sub_dir");
    fs::create_dir_all(&dir_sub).expect("create dir_sub");
    fs::create_dir_all(&dir_b).expect("create dir_b");

    // Create files
    File::create(dir_a.join("file1.txt"))
        .unwrap()
        .write_all(b"hello")
        .unwrap();
    File::create(dir_sub.join("file2.rs"))
        .unwrap()
        .write_all(b"fn main() {}")
        .unwrap();
    File::create(dir_b.join("file3.pdf"))
        .unwrap()
        .write_all(b"%PDF-1.4")
        .unwrap();

    let mut arena = FileArena::new();
    let config = CrawlerConfig::new().add_root(base);
    let crawler = Crawler::new(config);

    let stats = crawler.crawl(&mut arena).expect("crawl failed");

    assert_eq!(stats.files_indexed, 3);
    assert!(stats.dirs_scanned >= 3);
    assert_eq!(stats.errors_encountered, 0);
    assert_eq!(arena.len(), 7);

    // Verify all files can be reconstructed
    let mut names = Vec::new();
    for i in 0..arena.len() {
        let record = arena.get(i).unwrap();
        if !record.is_dir() {
            let path = arena
                .reconstruct_path_by_index(i)
                .expect("reconstruct path");
            names.push(path.file_name().unwrap().to_str().unwrap().to_string());
        }
    }
    names.sort();
    assert_eq!(names, vec!["file1.txt", "file2.rs", "file3.pdf"]);
}

#[test]
fn test_crawler_permission_denied_eacces_isolation() {
    let tmp = tempdir().expect("tempdir creation failed");
    let base = tmp.path();

    let readable_dir = base.join("readable");
    let locked_dir = base.join("locked_eacces");
    fs::create_dir_all(&readable_dir).expect("create readable");
    fs::create_dir_all(&locked_dir).expect("create locked");

    File::create(readable_dir.join("ok.txt"))
        .unwrap()
        .write_all(b"ok")
        .unwrap();
    File::create(locked_dir.join("secret.txt"))
        .unwrap()
        .write_all(b"secret")
        .unwrap();

    // Remove all permissions from locked_dir (chmod 000)
    let mut perms = fs::metadata(&locked_dir).unwrap().permissions();
    perms.set_mode(0o000);
    fs::set_permissions(&locked_dir, perms.clone()).unwrap();

    let mut arena = FileArena::new();
    let config = CrawlerConfig::new().add_root(base);
    let crawler = Crawler::new(config);

    // Traversal must complete without panicking or returning Err
    let stats = crawler
        .crawl(&mut arena)
        .expect("crawl must tolerate EACCES");

    // Restore permissions so tempdir can clean up cleanly
    perms.set_mode(0o755);
    let _ = fs::set_permissions(&locked_dir, perms);

    // ok.txt was indexed, secret.txt was skipped
    assert!(stats.files_indexed >= 1);
    assert!(stats.errors_encountered >= 1);
    assert!(!arena.is_empty());
}

#[test]
fn test_crawler_circular_symlink_prevention() {
    let tmp = tempdir().expect("tempdir creation failed");
    let base = tmp.path();

    let dir_cycle = base.join("dir_cycle");
    fs::create_dir_all(&dir_cycle).expect("create dir_cycle");
    File::create(dir_cycle.join("inside.txt"))
        .unwrap()
        .write_all(b"inside")
        .unwrap();

    // Create a circular link: dir_cycle/link_to_self -> dir_cycle
    let loop_link = dir_cycle.join("link_to_self");
    symlink(&dir_cycle, &loop_link).expect("create circular symlink");

    let mut arena = FileArena::new();
    let config = CrawlerConfig::new().add_root(base).follow_symlinks(true); // Must terminate without infinite loop
    let crawler = Crawler::new(config);

    let stats = crawler
        .crawl(&mut arena)
        .expect("crawler must prevent symlink cycles");

    assert!(stats.files_indexed >= 1);
}

#[test]
fn test_crawler_exclusions_pruning() {
    let tmp = tempdir().expect("tempdir creation failed");
    let base = tmp.path();

    let normal_dir = base.join("src");
    let git_dir = base.join(".git");
    let node_modules = base.join("node_modules");
    let custom_excl = base.join("build_artifacts");

    fs::create_dir_all(&normal_dir).unwrap();
    fs::create_dir_all(&git_dir).unwrap();
    fs::create_dir_all(&node_modules).unwrap();
    fs::create_dir_all(&custom_excl).unwrap();

    File::create(normal_dir.join("main.rs"))
        .unwrap()
        .write_all(b"fn main() {}")
        .unwrap();
    File::create(git_dir.join("HEAD"))
        .unwrap()
        .write_all(b"ref: refs/heads/main")
        .unwrap();
    File::create(node_modules.join("pkg.json"))
        .unwrap()
        .write_all(b"{}")
        .unwrap();
    File::create(custom_excl.join("output.o"))
        .unwrap()
        .write_all(b"\0\0")
        .unwrap();

    let mut arena = FileArena::new();
    let config = CrawlerConfig::new()
        .add_root(base)
        .add_exclusion(&custom_excl);
    let crawler = Crawler::new(config);

    let stats = crawler.crawl(&mut arena).expect("crawl failed");

    // Only main.rs should be indexed; .git, node_modules, and custom_excl are pruned
    assert_eq!(stats.files_indexed, 1);
    assert_eq!(arena.len(), 3);

    let file_paths: Vec<_> = (0..arena.len())
        .filter_map(|i| {
            let record = arena.get(i)?;
            if !record.is_dir() {
                arena.reconstruct_path_by_index(i)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(file_paths.len(), 1);
    assert_eq!(file_paths[0].file_name().unwrap(), "main.rs");
}

#[test]
fn test_crawler_max_depth_pruning() {
    let tmp = tempdir().expect("tempdir creation failed");
    let base = tmp.path();

    let level1 = base.join("l1");
    let level2 = level1.join("l2");
    let level3 = level2.join("l3");
    fs::create_dir_all(&level3).unwrap();

    File::create(level1.join("f1.txt"))
        .unwrap()
        .write_all(b"1")
        .unwrap();
    File::create(level2.join("f2.txt"))
        .unwrap()
        .write_all(b"2")
        .unwrap();
    File::create(level3.join("f3.txt"))
        .unwrap()
        .write_all(b"3")
        .unwrap();

    let mut arena = FileArena::new();
    let config = CrawlerConfig::new().add_root(base).max_depth(2);
    let crawler = Crawler::new(config);

    let stats = crawler.crawl(&mut arena).expect("crawl failed");

    // Depth 2 stops before level 3
    assert!(stats.files_indexed < 3);
}

#[test]
fn test_crawler_progress_callback() {
    let tmp = tempdir().expect("tempdir creation failed");
    let base = tmp.path();

    for i in 0..10 {
        File::create(base.join(format!("file_{}.txt", i)))
            .unwrap()
            .write_all(b"data")
            .unwrap();
    }

    let mut arena = FileArena::new();
    let config = CrawlerConfig::new().add_root(base);
    let crawler = Crawler::new(config);

    let mut received_progress = Vec::new();
    let stats = crawler
        .crawl_with_progress(&mut arena, |prog| {
            received_progress.push(prog);
        })
        .expect("crawl with progress failed");

    assert_eq!(stats.files_indexed, 10);
    assert!(
        !received_progress.is_empty(),
        "Progress callback must be invoked"
    );
}

#[test]
fn test_crawler_high_throughput_benchmark() {
    let tmp = tempdir().expect("tempdir creation failed");
    let base = tmp.path();

    // Create 50 directory trees with 100 files each = 5,000 files
    for d in 0..50 {
        let dir = base.join(format!("tree_{:02}", d));
        fs::create_dir_all(&dir).unwrap();
        for f in 0..100 {
            let file_path = dir.join(format!("file_{:03}.txt", f));
            File::create(file_path)
                .unwrap()
                .write_all(b"bench")
                .unwrap();
        }
    }

    let mut arena = FileArena::with_capacity(6000, 100_000, 100);
    let config = CrawlerConfig::new().add_root(base);
    let crawler = Crawler::new(config);

    let stats = crawler.crawl(&mut arena).expect("benchmark crawl failed");

    assert_eq!(stats.files_indexed, 5000);
    assert_eq!(stats.errors_encountered, 0);
    assert!(stats.dirs_scanned >= 50);
    assert!(stats.throughput_files_per_sec > 0.0);
}
