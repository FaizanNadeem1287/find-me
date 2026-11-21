//! Integration and edge-case test suite for binary index persistence and cold startup
//!
//! Validates header parsing, CRC32 integrity checks, corruption fallback, atomic writes,
//! and cold startup performance on 1,000,000 records.

use findme_core::{FileArena, FileFlags, IndexStorage, HEADER_SIZE, SCHEMA_VERSION, STORAGE_MAGIC};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tempfile::tempdir;

#[test]
fn test_storage_roundtrip_large() {
    let dir = tempdir().expect("tempdir");
    let cache_path = dir.path().join("index.bin");
    let mut arena = FileArena::new();

    // Populate with 10,000 files across 500 nested directories
    for i in 0..10_000 {
        let dir_num = i % 500;
        let p = format!("/home/user/project_{dir_num}/sub_{dir_num}/source_file_{i}.rs");
        let flags = if i % 2 == 0 {
            FileFlags::IS_EXECUTABLE
        } else {
            FileFlags::IS_READONLY
        };
        arena
            .add_file(
                Path::new(&p),
                flags,
                (i * 100) as u64,
                1700000000 + i as u64,
            )
            .expect("add_file");
    }

    assert_eq!(arena.len(), 10_000);

    IndexStorage::save_to_file(&arena, &cache_path).expect("save_to_file");
    assert!(cache_path.exists());

    let header = IndexStorage::verify_file(&cache_path).expect("verify_file");
    assert_eq!(header.magic, *STORAGE_MAGIC);
    assert_eq!(header.version, SCHEMA_VERSION);
    assert_eq!(header.record_count, 10_000);

    let loaded = IndexStorage::load_from_file(&cache_path).expect("load_from_file");
    assert_eq!(loaded.len(), 10_000);

    // Verify sample records
    for i in (0..10_000).step_by(250) {
        let dir_num = i % 500;
        let expected_path =
            format!("/home/user/project_{dir_num}/sub_{dir_num}/source_file_{i}.rs");
        let actual_path = loaded.reconstruct_path_by_index(i).expect("path");
        assert_eq!(actual_path, PathBuf::from(&expected_path));

        let rec = loaded.get(i).expect("record");
        assert_eq!(rec.size_bytes, (i * 100) as u64);
        assert_eq!(rec.mtime_secs, 1700000000 + i as u64);

        let found_idx = loaded
            .find_record_by_path(Path::new(&expected_path))
            .expect("find_record_by_path");
        assert_eq!(found_idx, i);
    }
}

#[test]
fn test_storage_missing_file_handling() {
    let dir = tempdir().expect("tempdir");
    let missing_path = dir.path().join("does_not_exist.bin");

    let load_res = IndexStorage::load_from_file(&missing_path);
    assert!(load_res.is_err());

    let fallback_res = IndexStorage::load_or_fallback(&missing_path);
    assert!(fallback_res.is_none());
}

#[test]
fn test_storage_empty_file_zero_bytes() {
    let dir = tempdir().expect("tempdir");
    let empty_path = dir.path().join("empty.bin");
    fs::write(&empty_path, b"").expect("create empty file");

    let load_res = IndexStorage::load_from_file(&empty_path);
    assert!(load_res.is_err());

    let verify_res = IndexStorage::verify_file(&empty_path);
    assert!(verify_res.is_err());

    let fallback_res = IndexStorage::load_or_fallback(&empty_path);
    assert!(fallback_res.is_none());
}

#[test]
fn test_storage_truncated_header() {
    let dir = tempdir().expect("tempdir");
    let trunc_path = dir.path().join("trunc.bin");
    // Write 16 bytes (< HEADER_SIZE of 32 bytes)
    fs::write(&trunc_path, b"FINDME\x01\0TRUNCATED").expect("write trunc");

    let load_res = IndexStorage::load_from_file(&trunc_path);
    assert!(load_res.is_err());

    let fallback_res = IndexStorage::load_or_fallback(&trunc_path);
    assert!(fallback_res.is_none());
}

#[test]
fn test_storage_invalid_magic_bytes() {
    let dir = tempdir().expect("tempdir");
    let cache_path = dir.path().join("index.bin");
    let mut arena = FileArena::new();
    arena
        .add_file(Path::new("/tmp/test.txt"), FileFlags::NONE, 10, 100)
        .expect("add");
    IndexStorage::save_to_file(&arena, &cache_path).expect("save");

    // Corrupt magic bytes
    let mut data = fs::read(&cache_path).expect("read");
    data[0..8].copy_from_slice(b"BADMAGIC");
    fs::write(&cache_path, &data).expect("write corrupt");

    let load_res = IndexStorage::load_from_file(&cache_path);
    assert!(load_res.is_err());
    let err_str = load_res.err().map(|e| e.to_string()).unwrap_or_default();
    assert!(err_str.contains("Invalid index magic bytes"));

    let fallback_res = IndexStorage::load_or_fallback(&cache_path);
    assert!(fallback_res.is_none());
}

#[test]
fn test_storage_schema_version_mismatch() {
    let dir = tempdir().expect("tempdir");
    let cache_path = dir.path().join("index.bin");
    let mut arena = FileArena::new();
    arena
        .add_file(Path::new("/tmp/test.txt"), FileFlags::NONE, 10, 100)
        .expect("add");
    IndexStorage::save_to_file(&arena, &cache_path).expect("save");

    // Overwrite schema version to 999
    let mut data = fs::read(&cache_path).expect("read");
    let bad_ver: u32 = 999;
    data[8..12].copy_from_slice(&bad_ver.to_le_bytes());
    fs::write(&cache_path, &data).expect("write corrupt");

    let load_res = IndexStorage::load_from_file(&cache_path);
    assert!(load_res.is_err());
    let err_str = load_res.err().map(|e| e.to_string()).unwrap_or_default();
    assert!(err_str.contains("Unsupported index schema version 999"));

    let fallback_res = IndexStorage::load_or_fallback(&cache_path);
    assert!(fallback_res.is_none());
}

#[test]
fn test_storage_crc32_checksum_mismatch() {
    let dir = tempdir().expect("tempdir");
    let cache_path = dir.path().join("index.bin");
    let mut arena = FileArena::new();
    arena
        .add_file(Path::new("/tmp/test.txt"), FileFlags::NONE, 10, 100)
        .expect("add");
    IndexStorage::save_to_file(&arena, &cache_path).expect("save");

    // Mutate a byte in the payload
    let mut data = fs::read(&cache_path).expect("read");
    let payload_byte_idx = HEADER_SIZE + 5;
    data[payload_byte_idx] ^= 0xFF;
    fs::write(&cache_path, &data).expect("write corrupt");

    let load_res = IndexStorage::load_from_file(&cache_path);
    assert!(load_res.is_err());
    let err_str = load_res.err().map(|e| e.to_string()).unwrap_or_default();
    assert!(err_str.contains("CRC32 checksum mismatch"));

    let fallback_res = IndexStorage::load_or_fallback(&cache_path);
    assert!(fallback_res.is_none());
}

#[test]
fn test_storage_truncated_payload() {
    let dir = tempdir().expect("tempdir");
    let cache_path = dir.path().join("index.bin");
    let mut arena = FileArena::new();
    arena
        .add_file(Path::new("/tmp/test.txt"), FileFlags::NONE, 10, 100)
        .expect("add");
    IndexStorage::save_to_file(&arena, &cache_path).expect("save");

    // Truncate payload in half
    let data = fs::read(&cache_path).expect("read");
    let truncated_len = HEADER_SIZE + (data.len() - HEADER_SIZE) / 2;
    fs::write(&cache_path, &data[..truncated_len]).expect("write truncated");

    let load_res = IndexStorage::load_from_file(&cache_path);
    assert!(load_res.is_err());

    let fallback_res = IndexStorage::load_or_fallback(&cache_path);
    assert!(fallback_res.is_none());
}

#[test]
fn test_storage_atomic_overwrite_and_clean_temp() {
    let dir = tempdir().expect("tempdir");
    let cache_path = dir.path().join("index.bin");

    let mut arena1 = FileArena::new();
    arena1
        .add_file(Path::new("/tmp/initial.txt"), FileFlags::NONE, 50, 1)
        .expect("add 1");
    IndexStorage::save_to_file(&arena1, &cache_path).expect("save 1");

    let loaded1 = IndexStorage::load_from_file(&cache_path).expect("load 1");
    assert_eq!(loaded1.len(), 1);

    // Overwrite with second arena
    let mut arena2 = FileArena::new();
    arena2
        .add_file(Path::new("/tmp/new1.txt"), FileFlags::NONE, 100, 2)
        .expect("add new 1");
    arena2
        .add_file(Path::new("/tmp/new2.txt"), FileFlags::NONE, 200, 3)
        .expect("add new 2");
    IndexStorage::save_to_file(&arena2, &cache_path).expect("save 2");

    let loaded2 = IndexStorage::load_from_file(&cache_path).expect("load 2");
    assert_eq!(loaded2.len(), 2);

    // Verify no temporary files remain in directory
    for entry in fs::read_dir(dir.path()).expect("read_dir") {
        let entry = entry.expect("entry");
        let fname = entry.file_name().to_string_lossy().into_owned();
        assert!(
            !fname.contains(".tmp."),
            "Temp file was not cleaned up: {fname}"
        );
    }
}

#[test]
fn test_storage_xdg_path_resolution() {
    let cache_dir = IndexStorage::default_cache_dir();
    let cache_path = IndexStorage::default_cache_path();

    assert!(cache_dir.to_string_lossy().contains("find-me"));
    assert_eq!(cache_path, cache_dir.join("index.bin"));
}

#[test]
fn test_storage_one_million_records_cold_startup_benchmark() {
    let dir = tempdir().expect("tempdir");
    let cache_path = dir.path().join("index_1m.bin");

    println!("Generating 1,000,000 synthetic records for persistence benchmark...");
    let gen_start = Instant::now();
    let mut arena = FileArena::with_capacity(1_000_000, 25 * 1024 * 1024, 10_000);

    for i in 0..1_000_000 {
        let dir_id = (i % 10_000) as u32;
        let file_path = format!("/usr/share/app_{dir_id}/module_{i}.dat");
        arena
            .add_file(
                Path::new(&file_path),
                FileFlags::NONE,
                (i * 10) as u64,
                1700000000,
            )
            .expect("add_file");
    }
    println!("Generation completed in {:.2?}", gen_start.elapsed());
    assert_eq!(arena.len(), 1_000_000);

    println!("Persisting 1,000,000 records to disk...");
    let save_start = Instant::now();
    IndexStorage::save_to_file(&arena, &cache_path).expect("save_to_file 1M");
    println!("Save completed in {:.2?}", save_start.elapsed());

    let file_size_mb = fs::metadata(&cache_path).expect("meta").len() as f64 / (1024.0 * 1024.0);
    println!("Persisted index size on disk: {file_size_mb:.2} MB");

    // Cold load benchmark
    println!("Benchmarking cold load from disk...");
    let load_start = Instant::now();
    let loaded = IndexStorage::load_from_file(&cache_path).expect("load_from_file 1M");
    let load_duration = load_start.elapsed();

    println!("Cold startup 1,000,000 records load time: {load_duration:?}");
    assert_eq!(loaded.len(), 1_000_000);

    // Verify lookup integrity
    let sample_idx = loaded
        .find_record_by_path(Path::new("/usr/share/app_0/module_0.dat"))
        .expect("find record 0");
    assert_eq!(sample_idx, 0);

    // In release mode, enforce <50ms budget; in debug mode, verify reasonable completion
    if !cfg!(debug_assertions) {
        assert!(
            load_duration.as_millis() < 50,
            "Cold load of 1,000,000 records took {:?}, exceeding 50ms performance budget!",
            load_duration
        );
    } else {
        println!(
            "[NOTE] Debug mode active: load time was {:?} (release budget is <50ms)",
            load_duration
        );
    }
}
