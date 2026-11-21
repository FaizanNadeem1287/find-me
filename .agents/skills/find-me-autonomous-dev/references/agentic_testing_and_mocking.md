# Agentic Testing & Mocking Guide for Autonomous Development

This guide outlines how an autonomous agent constructs test fixtures, mocks, and property fuzz tests without requiring manual human setup.

---

## 1. Test-Driven Development (TDD) for Agents

When implementing any task autonomously, follow this test-first strategy:
1. **Identify the Interface**: Define the function signatures, structs, and error types first.
2. **Write the Test Fixture**: Write a test verifying the expected behavior before writing the implementation.
3. **Run the Test**: Verify the test fails for the expected reason (compile error or assertion failure).
4. **Implement**: Write the minimal code to satisfy the test.
5. **Verify**: Ensure the test passes.

---

## 2. Generating Isolated Filesystem Fixtures

For crawler and filesystem tests, agents must generate realistic mock directories automatically:

```rust
use tempfile::TempDir;
use std::fs::{self, File};
use std::io::Write;

pub struct MockFilesystem {
    pub dir: TempDir,
}

impl MockFilesystem {
    pub fn create_standard_test_tree() -> std::io::Result<Self> {
        let dir = TempDir::new()?;
        let root = dir.path();

        // Standard structure
        fs::create_dir_all(root.join("projects/rust_app/src"))?;
        fs::create_dir_all(root.join("documents/finance"))?;
        fs::create_dir_all(root.join(".hidden_folder"))?;

        // Files with varying sizes
        File::create(root.join("projects/rust_app/Cargo.toml"))?
            .write_all(b"[package]\nname = \"test\"\n")?;
        File::create(root.join("projects/rust_app/src/main.rs"))?
            .write_all(b"fn main() {}\n")?;
        File::create(root.join("documents/finance/taxes_2026.pdf"))?
            .write_all(&vec![0u8; 1024 * 500])?; // 500 KB

        Ok(Self { dir })
    }
}
```

---

## 3. Property-Based Fuzzing with `proptest`

To guarantee that the query parser and matching algorithms never panic under autonomous execution, agents write property tests:

```rust
use proptest::prelude::*;
use findme_core::query::QueryParser;

proptest! {
    #[test]
    fn parser_never_panics_on_arbitrary_strings(s in "\\PC*") {
        // Must never panic on any valid Unicode string
        let _ = QueryParser::parse(&s);
    }

    #[test]
    fn parser_handles_arbitrary_size_operators(
        prefix in "(size:>|size:<|size:>=|size:<=)",
        num in 0u64..10_000_000,
        unit in "(b|kb|mb|gb|)"
    ) {
        let query = format!("{}{}{}", prefix, num, unit);
        let _ = QueryParser::parse(&query);
    }
}
```

---

## 4. Headless Mock Engine for UI Development

While building UI screens in Phase 4–6, agents use a deterministic `MockEngine` so UI development can proceed independently without waiting for the full filesystem crawler:

```rust
pub struct MockEngine {
    items: Vec<SearchResultItem>,
}

impl MockEngine {
    pub fn with_sample_dataset(count: usize) -> Self {
        let mut items = Vec::with_capacity(count);
        for i in 0..count {
            items.push(SearchResultItem {
                id: i as u64,
                name: format!("file_{:06}.rs", i),
                path: format!("/home/user/workspace/src/module_{}/file_{:06}.rs", i % 100, i),
                size_bytes: (i as u64 * 1024) % (100 * 1024 * 1024),
                mtime_secs: 1763740000 + (i as u64 * 60),
                is_dir: i % 10 == 0,
            });
        }
        Self { items }
    }
}
```
