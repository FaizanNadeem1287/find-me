//! Exhaustive integration test suite for FileRecord, StringPool, DirectoryTable, and FileArena edge cases

use findme_core::{
    DirectoryTable, EngineError, FileArena, FileFlags, FileRecord, SearchResultItem, ROOT_DIR_ID,
};
use std::path::{Path, PathBuf};

// ============================================================================
// 1. FileRecord & Memory Layout Edge Cases
// ============================================================================

#[test]
fn test_filerecord_exact_size_alignment_and_offsets() {
    assert_eq!(std::mem::size_of::<FileRecord>(), 32);
    assert_eq!(std::mem::align_of::<FileRecord>(), 8);

    // Verify exact field offsets to guarantee no compiler padding gaps
    let dummy = FileRecord::new(0, 0, 0, FileFlags::NONE, 0, 0);
    let base_ptr = &dummy as *const FileRecord as usize;

    let dir_id_ptr = &dummy.dir_id as *const u32 as usize;
    let name_offset_ptr = &dummy.name_offset as *const u32 as usize;
    let name_len_ptr = &dummy.name_len as *const u16 as usize;
    let flags_ptr = &dummy.flags as *const FileFlags as usize;
    let reserved_ptr = &dummy._reserved as *const u32 as usize;
    let size_bytes_ptr = &dummy.size_bytes as *const u64 as usize;
    let mtime_secs_ptr = &dummy.mtime_secs as *const u64 as usize;

    assert_eq!(dir_id_ptr - base_ptr, 0, "dir_id offset must be 0");
    assert_eq!(
        name_offset_ptr - base_ptr,
        4,
        "name_offset offset must be 4"
    );
    assert_eq!(name_len_ptr - base_ptr, 8, "name_len offset must be 8");
    assert_eq!(flags_ptr - base_ptr, 10, "flags offset must be 10");
    assert_eq!(reserved_ptr - base_ptr, 12, "_reserved offset must be 12");
    assert_eq!(
        size_bytes_ptr - base_ptr,
        16,
        "size_bytes offset must be 16"
    );
    assert_eq!(
        mtime_secs_ptr - base_ptr,
        24,
        "mtime_secs offset must be 24"
    );
}

#[test]
fn test_filerecord_extreme_sizes_and_timestamps() {
    let sizes = [
        0u64,
        1u64,
        512u64,
        1023u64,
        1024u64,
        1024 * 1024,
        1024 * 1024 * 1024,
        u32::MAX as u64,
        (u32::MAX as u64) + 1,
        10u64 * 1024 * 1024 * 1024 * 1024, // 10 TB
        u64::MAX,
    ];

    let timestamps = [
        0u64,
        1u64,
        86400u64,
        1700000000u64,
        i32::MAX as u64, // Year 2038 boundary
        (i32::MAX as u64) + 1,
        32503680000u64, // Year 3000
        u64::MAX,
    ];

    for &size in &sizes {
        for &ts in &timestamps {
            let record =
                FileRecord::with_reserved(42, 100, 20, FileFlags::IS_READONLY, 999, size, ts);
            assert_eq!(record.dir_id, 42);
            assert_eq!(record.name_offset, 100);
            assert_eq!(record.name_len, 20);
            assert_eq!(record.size_bytes, size);
            assert_eq!(record.mtime_secs, ts);
            assert_eq!(record._reserved, 999);
            assert!(record.is_readonly());
            assert!(!record.is_dir());
        }
    }
}

#[test]
fn test_search_result_item_size_and_mtime_formatting() {
    let dir_item = SearchResultItem {
        id: 1,
        name: "test_dir".into(),
        path: "/path/test_dir".into(),
        size_bytes: 4096,
        mtime_secs: 0,
        is_dir: true,
    };
    assert_eq!(
        dir_item.formatted_size(),
        "",
        "Directories must have empty formatted size"
    );

    let cases = [
        (0u64, "0 B"),
        (512u64, "512 B"),
        (1023u64, "1023 B"),
        (1024u64, "1.0 KB"),
        (1536u64, "1.5 KB"),
        (1024 * 1024, "1.00 MB"),
        (50 * 1024 * 1024, "50.00 MB"),
        (1024 * 1024 * 1024, "1.00 GB"),
        (3 * 1024 * 1024 * 1024 + 512 * 1024 * 1024, "3.50 GB"),
    ];

    for (bytes, expected) in cases {
        let item = SearchResultItem {
            id: 2,
            name: "file.bin".into(),
            path: "/path/file.bin".into(),
            size_bytes: bytes,
            mtime_secs: 0,
            is_dir: false,
        };
        assert_eq!(item.formatted_size(), expected);
    }

    // Timestamp formatting checks
    let epoch_zero = SearchResultItem {
        id: 3,
        name: "epoch.txt".into(),
        path: "/epoch.txt".into(),
        size_bytes: 10,
        mtime_secs: 0,
        is_dir: false,
    };
    assert_eq!(epoch_zero.formatted_mtime(), "1970-01-01 00:00:00");

    let y2k = SearchResultItem {
        id: 4,
        name: "y2k.txt".into(),
        path: "/y2k.txt".into(),
        size_bytes: 10,
        mtime_secs: 946684800, // 2000-01-01 00:00:00 UTC
        is_dir: false,
    };
    assert_eq!(y2k.formatted_mtime(), "2000-01-01 00:00:00");
}

#[test]
fn test_file_flags_exhaustive_operations() {
    let mut flags = FileFlags::NONE;
    assert_eq!(flags, FileFlags::NONE);

    // Bitwise OR
    flags = flags | FileFlags::IS_DIR | FileFlags::IS_SYMLINK;
    assert!(flags.is_dir());
    assert!(flags.is_symlink());
    assert!(!flags.is_hidden());
    assert!(!flags.is_executable());
    assert!(!flags.is_readonly());

    // Bitwise AND
    let dir_only = flags & FileFlags::IS_DIR;
    assert!(dir_only.is_dir());
    assert!(!dir_only.is_symlink());

    // Bitwise NOT
    let not_dir = !FileFlags::IS_DIR;
    assert!((flags & not_dir).is_symlink());
    assert!(!(flags & not_dir).is_dir());

    // BitOrAssign & BitAndAssign
    let mut f2 = FileFlags::IS_EXECUTABLE;
    f2 |= FileFlags::IS_READONLY;
    assert!(f2.is_executable());
    assert!(f2.is_readonly());

    f2 &= FileFlags::IS_READONLY;
    assert!(!f2.is_executable());
    assert!(f2.is_readonly());

    // Remove
    f2.remove(FileFlags::IS_READONLY);
    assert_eq!(f2, FileFlags::NONE);
}

// ============================================================================
// 2. StringPool Edge Cases
// ============================================================================

#[test]
fn test_string_pool_empty_and_single_char() {
    let mut pool = findme_core::StringPool::new();
    let (off0, len0) = pool.append("").expect("empty string append failed");
    assert_eq!(len0, 0);
    assert_eq!(pool.get(off0, len0), Some(""));

    let (off1, len1) = pool.append("x").expect("single char append failed");
    assert_eq!(len1, 1);
    assert_eq!(pool.get(off1, len1), Some("x"));
}

#[test]
fn test_string_pool_posix_max_filename() {
    let mut pool = findme_core::StringPool::new();
    // Linux POSIX NAME_MAX is 255 bytes
    let max_posix = "a".repeat(255);
    let (off, len) = pool.append(&max_posix).expect("255 bytes append failed");
    assert_eq!(len, 255);
    assert_eq!(pool.get(off, len), Some(max_posix.as_str()));
}

#[test]
fn test_string_pool_boundary_u16_max() {
    let mut pool = findme_core::StringPool::new();
    let max_len_str = "b".repeat(u16::MAX as usize);
    let (off, len) = pool.append(&max_len_str).expect("u16::MAX append failed");
    assert_eq!(len, u16::MAX);
    assert_eq!(pool.get(off, len), Some(max_len_str.as_str()));

    // Exceeding u16::MAX must safely return Err(EngineError::NameTooLong)
    let too_large_str = "c".repeat((u16::MAX as usize) + 1);
    let res = pool.append(&too_large_str);
    assert!(matches!(res, Err(EngineError::NameTooLong(65536))));
}

#[test]
fn test_string_pool_invalid_utf8_slice_rejection() {
    let mut pool = findme_core::StringPool::new();
    // 🦀 is 4 bytes in UTF-8: [0xF0, 0x9F, 0xA6, 0x80]
    let (off, len) = pool.append("🦀").expect("append failed");
    assert_eq!(len, 4);

    // Slicing half the crab emoji (2 bytes) must safely return None (not panic, not corrupt)
    assert_eq!(pool.get(off, 2), None);
    assert_eq!(pool.get(off + 1, 2), None);
    // Exact slice returns valid crab
    assert_eq!(pool.get(off, 4), Some("🦀"));
}

#[test]
fn test_string_pool_complex_unicode_and_special_characters() {
    let mut pool = findme_core::StringPool::new();
    let special_filenames = [
        "   leading_and_trailing_spaces.txt   ",
        "tab\tseparated\tname.tsv",
        "newline\nin\nfilename.log",
        "quotes_\"double\"_'single'.rs",
        "shell_chars_`echo $HOME`_&_|_<>.sh",
        "dots_...and...more...dots.tar.gz.bak",
        "ملف_عربي_مع_مسافات.doc",
        "קובץ_עברית.pdf",
        "ελληνικό_αρχείο.odt",
        "数学_∑_∫_∂x_π_∞.latex",
        "👨‍👩‍👧‍👦_family_zwj_sequence.png",
        "mixed_🦀_English_中文_Deutsch_über_größen.txt",
    ];

    let mut handles = Vec::new();
    for &name in &special_filenames {
        let handle = pool.append(name).expect("append failed");
        handles.push(handle);
    }

    for (idx, &(off, len)) in handles.iter().enumerate() {
        assert_eq!(pool.get(off, len), Some(special_filenames[idx]));
    }
}

// ============================================================================
// 3. DirectoryTable Edge Cases
// ============================================================================

#[test]
fn test_directory_root_and_redundant_slashes() {
    let mut table = DirectoryTable::new();
    assert_eq!(table.get_or_insert(Path::new("/")), ROOT_DIR_ID);
    assert_eq!(table.get_path(ROOT_DIR_ID), Some(PathBuf::from("/")));

    // Empty path resolves to root
    assert_eq!(table.get_or_insert(Path::new("")), ROOT_DIR_ID);

    // Repeated calls return identical ID
    let id1 = table.get_or_insert(Path::new("/usr/local/bin"));
    let id2 = table.get_or_insert(Path::new("/usr/local/bin"));
    assert_eq!(id1, id2);
    assert_eq!(table.get_path(id1), Some(PathBuf::from("/usr/local/bin")));
}

#[test]
fn test_directory_deeply_nested_hierarchy() {
    let mut table = DirectoryTable::new();
    let mut deep_path = PathBuf::from("/");
    for i in 0..128 {
        deep_path.push(format!("level_{:03}", i));
    }

    let id = table.get_or_insert(&deep_path);
    let reconstructed = table.get_path(id);
    assert_eq!(reconstructed, Some(deep_path));
}

#[test]
fn test_directory_cycle_prevention_guard() {
    let mut table = DirectoryTable::new();
    let id1 = table.get_or_insert(Path::new("/dir_a"));
    let id2 = table.get_or_insert(Path::new("/dir_a/dir_b"));

    // Artificially inject a cycle into DirectoryTable to test cycle guard:
    // id1 points to id2 and id2 points to id1
    // (Simulating corrupted cache or filesystem loop)
    // We access entries through serialization/deserialization or helper
    let mut entries_json = serde_json::to_value(&table).expect("serialize table");
    // Mutate parent_id of id1 to point to id2 (cycle!)
    entries_json["entries"][id1 as usize]["parent_id"] = serde_json::json!(id2);

    let corrupt_table: DirectoryTable =
        serde_json::from_value(entries_json).expect("deserialize corrupted table");

    // Must return None safely via MAX_DEPTH guard without hanging or stack overflow!
    assert_eq!(corrupt_table.get_path(id1), None);
    assert_eq!(corrupt_table.get_path(id2), None);
}

#[test]
fn test_directory_siblings_and_isolation() {
    let mut table = DirectoryTable::new();
    let bin_id = table.get_or_insert(Path::new("/usr/bin"));
    let lib_id = table.get_or_insert(Path::new("/usr/lib"));
    let share_id = table.get_or_insert(Path::new("/usr/share"));

    assert_ne!(bin_id, lib_id);
    assert_ne!(lib_id, share_id);

    let usr_id = table.find_id(Path::new("/usr")).expect("/usr must exist");
    assert_eq!(table.get_entry(bin_id).unwrap().parent_id, usr_id);
    assert_eq!(table.get_entry(lib_id).unwrap().parent_id, usr_id);
    assert_eq!(table.get_entry(share_id).unwrap().parent_id, usr_id);

    assert_eq!(table.get_path(bin_id), Some(PathBuf::from("/usr/bin")));
    assert_eq!(table.get_path(lib_id), Some(PathBuf::from("/usr/lib")));
    assert_eq!(table.get_path(share_id), Some(PathBuf::from("/usr/share")));
}

// ============================================================================
// 4. FileArena & Full Path Reconstruction Edge Cases
// ============================================================================

#[test]
fn test_arena_files_directly_in_root() {
    let mut arena = FileArena::new();
    let vmlinuz_idx = arena
        .add_file(
            Path::new("/vmlinuz"),
            FileFlags::IS_SYMLINK,
            1024,
            1763740000,
        )
        .expect("add /vmlinuz failed");
    let initrd_idx = arena
        .add_file(Path::new("/initrd.img"), FileFlags::NONE, 2048, 1763740001)
        .expect("add /initrd.img failed");

    assert_eq!(
        arena.reconstruct_path_by_index(vmlinuz_idx as usize),
        Some(PathBuf::from("/vmlinuz"))
    );
    assert_eq!(
        arena.reconstruct_path_by_index(initrd_idx as usize),
        Some(PathBuf::from("/initrd.img"))
    );
}

#[test]
fn test_arena_hidden_files_and_dotted_extensions() {
    let mut arena = FileArena::new();
    let hidden_idx = arena
        .add_file(
            Path::new("/home/user/.bashrc"),
            FileFlags::IS_HIDDEN,
            3450,
            1763741000,
        )
        .expect("add .bashrc failed");

    let multi_dot_idx = arena
        .add_file(
            Path::new("/home/user/archive.tar.gz.bak"),
            FileFlags::NONE,
            99999,
            1763742000,
        )
        .expect("add multi-dot failed");

    let no_ext_idx = arena
        .add_file(
            Path::new("/home/user/Makefile"),
            FileFlags::IS_READONLY,
            500,
            1763743000,
        )
        .expect("add no-ext failed");

    assert_eq!(
        arena.reconstruct_path_by_index(hidden_idx as usize),
        Some(PathBuf::from("/home/user/.bashrc"))
    );
    assert_eq!(
        arena.reconstruct_path_by_index(multi_dot_idx as usize),
        Some(PathBuf::from("/home/user/archive.tar.gz.bak"))
    );
    assert_eq!(
        arena.reconstruct_path_by_index(no_ext_idx as usize),
        Some(PathBuf::from("/home/user/Makefile"))
    );
}

#[test]
fn test_arena_out_of_bounds_and_corrupt_records() {
    let mut arena = FileArena::new();
    let _ = arena.add_file(Path::new("/home/user/valid.txt"), FileFlags::NONE, 100, 100);

    // Out-of-bounds indices return None safely
    assert_eq!(arena.get(10), None);
    assert_eq!(arena.get(usize::MAX), None);
    assert_eq!(arena.reconstruct_path_by_index(10), None);
    assert_eq!(arena.reconstruct_path_by_index(usize::MAX), None);

    // Corrupted records referencing non-existent directory IDs or offsets
    let corrupt_dir_record = FileRecord::new(999999, 0, 5, FileFlags::NONE, 10, 10);
    assert_eq!(arena.reconstruct_path(&corrupt_dir_record), None);

    let corrupt_pool_record = FileRecord::new(ROOT_DIR_ID, 999999, 10, FileFlags::NONE, 10, 10);
    assert_eq!(arena.reconstruct_path(&corrupt_pool_record), None);
}

#[test]
fn test_arena_clear_and_reuse() {
    let mut arena = FileArena::new();
    let _ = arena.add_file(Path::new("/tmp/test.txt"), FileFlags::NONE, 10, 10);
    assert_eq!(arena.len(), 1);

    arena.clear();
    assert_eq!(arena.len(), 0);
    assert!(arena.is_empty());
    assert_eq!(arena.pool().len(), 0);

    // Reusing after clear
    let idx = arena
        .add_file(Path::new("/var/log/syslog"), FileFlags::NONE, 200, 200)
        .expect("reuse failed");
    assert_eq!(idx, 0);
    assert_eq!(
        arena.reconstruct_path_by_index(0),
        Some(PathBuf::from("/var/log/syslog"))
    );
}
