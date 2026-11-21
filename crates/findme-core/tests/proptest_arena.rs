//! Property-based fuzz testing for StringPool, DirectoryTable, and FileArena

use findme_core::{FileArena, FileFlags, StringPool};
use proptest::prelude::*;
use std::path::PathBuf;

proptest! {
    // ------------------------------------------------------------------------
    // Property 1: Any valid UTF-8 string appended to StringPool must borrow back
    // exactly identical to the input string.
    // ------------------------------------------------------------------------
    #[test]
    fn prop_string_pool_roundtrip(s in "\\PC{0, 255}") {
        let mut pool = StringPool::new();
        let (offset, len) = pool.append(&s).expect("valid UTF-8 must append");
        let borrowed = pool.get(offset, len).expect("must retrieve slice");
        prop_assert_eq!(borrowed, s.as_str());
    }

    // ------------------------------------------------------------------------
    // Property 2: Any arbitrary valid directory path + filename combination
    // added to FileArena must reconstruct back to the exact original POSIX path.
    // ------------------------------------------------------------------------
    #[test]
    fn prop_arena_path_reconstruction(
        segments in prop::collection::vec("[a-zA-Z0-9_\\-\\.]+", 1..6),
        filename in "[a-zA-Z0-9_\\-\\.]+\\.[a-z]{1,4}",
        size in any::<u64>(),
        mtime in any::<u64>(),
    ) {
        let mut path = PathBuf::from("/");
        for seg in &segments {
            path.push(seg);
        }
        path.push(&filename);

        let mut arena = FileArena::new();
        let idx = arena.add_file(&path, FileFlags::NONE, size, mtime)
            .expect("adding file to arena must succeed");

        let reconstructed = arena.reconstruct_path_by_index(idx as usize)
            .expect("path reconstruction must succeed");

        prop_assert_eq!(reconstructed, path);
    }

    // ------------------------------------------------------------------------
    // Property 3: FileFlags bitwise operations must strictly obey boolean algebra
    // invariants across arbitrary u16 bit patterns.
    // ------------------------------------------------------------------------
    #[test]
    fn prop_file_flags_algebra(bits_a in any::<u16>(), bits_b in any::<u16>()) {
        let flag_a = FileFlags(bits_a);
        let flag_b = FileFlags(bits_b);

        // Commutativity of BitOr: A | B == B | A
        prop_assert_eq!(flag_a | flag_b, flag_b | flag_a);

        // Commutativity of BitAnd: A & B == B & A
        prop_assert_eq!(flag_a & flag_b, flag_b & flag_a);

        // Idempotency: A | A == A, A & A == A
        prop_assert_eq!(flag_a | flag_a, flag_a);
        prop_assert_eq!(flag_a & flag_a, flag_a);

        // Absorption: A | (A & B) == A
        prop_assert_eq!(flag_a | (flag_a & flag_b), flag_a);

        // Involution: !(!A) == A
        prop_assert_eq!(!(!flag_a), flag_a);
    }
}
