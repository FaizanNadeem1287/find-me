# Testing Pyramid & QA Strategy for Find-Me

This document provides a clear, comprehensive breakdown of every type of testing used in Find-Me, explaining what each test type does, why it is critical, and how it protects against bugs, crashes, and performance regressions.

---

## 1. The Multi-Tiered Testing Strategy

Because Find-Me is a high-speed systems tool combining low-level Linux filesystem operations, concurrent background threading, and an interactive desktop UI, relying on a single test approach is insufficient. We employ six distinct test disciplines:

```
+--------------------------------------------------------------------------------+
| 1. Unit Testing: Micro-level correctness of algorithms and data structures     |
+--------------------------------------------------------------------------------+
| 2. Property-Based / Fuzz Testing: Random input generation to catch panics      |
+--------------------------------------------------------------------------------+
| 3. Integration Testing: Realistic filesystem fixtures, jwalk, and inotify      |
+--------------------------------------------------------------------------------+
| 4. Headless UI Testing: Deterministic Elm-architecture simulation via iced_test |
+--------------------------------------------------------------------------------+
| 5. Performance & Regression Testing: Benchmarking throughput and latency       |
+--------------------------------------------------------------------------------+
| 6. Smoke & System Testing: Debian package validation and live desktop checks   |
+--------------------------------------------------------------------------------+
```

---

## 2. Detailed Breakdown of Test Disciplines

### 2.1 Unit Testing (`cargo test --lib`)
- **What it does**: Tests isolated functions, structs, and algorithms in isolation without depending on disk or network.
- **Where applied**:
  - `FileRecord` packing/unpacking and bitflag manipulation.
  - String pool interning and offset resolution.
  - Query tokenizer & AST parsing (e.g., verifying `size:>10mb` converts to `10_485_760` bytes).
  - Date filtering calculations (e.g., verifying `dm:today` matches epoch seconds between midnight and now).
- **Goal**: 100% test coverage over core domain logic.

### 2.2 Property-Based & Fuzz Testing (`proptest`)
- **What it does**: Instead of testing handpicked examples (like `"test.txt"`), property testing generates thousands of random, weird, and edge-case inputs (empty strings, Unicode grapheme clusters, null bytes, extreme file sizes, deeply nested paths, malformed regexes) to prove properties always hold true.
- **Example Invariant**:
  - *"For any random arbitrary string query $Q$, the query parser MUST either return a valid parsed AST or a descriptive SyntaxError, and MUST NEVER panic or crash."*
- **Goal**: Eliminate crashes caused by unexpected user input or corrupt filesystem metadata.

### 2.3 Integration Testing (`tests/*.rs` with `tempfile`)
- **What it does**: Tests how different components interact together in realistic operating conditions.
- **Where applied**:
  - Creating temporary directory trees with nested folders, symlinks, hidden files, empty files, and permission constraints using `tempfile::tempdir()`.
  - Running `jwalk` parallel traversal and verifying that the resulting in-memory index exactly matches every file created on disk.
  - Modifying, renaming, and deleting files inside the temporary directory and verifying that the `notify` inotify watcher updates the in-memory arena within 200ms.
- **Goal**: Guarantee the engine accurately reflects the real Linux filesystem.

### 2.4 Headless Desktop UI Testing (`iced_test`)
- **What it does**: Tests UI user interactions, keyboard shortcuts, view rendering, and state updates **without launching a physical window or requiring an X11/Wayland display server**.
- **How it works**:
  - Iced uses the Elm architecture: `State`, `Message`, and `update(&mut self, message)`.
  - The `iced_test::Simulator` wraps the view tree, clicks buttons, types text into input boxes, and fires keyboard events headlessly.
  - Generated messages are fed into `update()`, and assertions verify state and view changes.
- **Goal**: UI logic and navigation are 100% tested in automated CI pipelines without physical screens.

### 2.5 Performance & Benchmark Testing (`criterion`)
- **What it does**: Measures execution time, throughput, and memory consumption with statistical rigor.
- **Key Metrics Enforced**:
  - **Filesystem Crawling Throughput**: >200,000 files/sec on SSD.
  - **Query Matching Latency**: <5 milliseconds for 1,000,000 indexed records.
  - **Memory Efficiency**: <45 MB RAM per 1,000,000 files in the in-memory arena.
  - **Cold Startup Latency**: <50 ms to load cached index from disk via `rkyv`/`memmap2`.
- **Goal**: Prevent performance regressions from slipping into production.

### 2.6 Smoke & System Testing
- **What it does**: Validates the built application as an end-user will experience it.
- **Where applied**:
  - Building the `.deb` package via `cargo deb`.
  - Validating the desktop entry file via `desktop-file-validate assets/find-me.desktop`.
  - Verifying application launch and exit status codes.
- **Goal**: Ensure the distribution artifact installs cleanly on Debian/Ubuntu systems.

---

## 3. Eliminating Flaky Tests

A **flaky test** is a test that sometimes passes and sometimes fails without code changes. Flaky tests destroy developer confidence and usually stem from bad timing assumptions:
1. **Never use arbitrary `std::thread::sleep`**:
   - Bad: `thread::sleep(Duration::from_millis(50)); assert!(watcher.has_updated());` (fails on slow/busy CI runners).
   - Good: Use Tokio channels with bounded timeouts:
     ```rust
     tokio::time::timeout(Duration::from_secs(2), rx.recv())
         .await
         .expect("Timed out waiting for file watcher update")
         .expect("Channel closed");
     ```
2. **Isolate Filesystem Tests**:
   - Each integration test must operate inside its own unique `tempfile::tempdir()`. Never share paths between concurrent test runners.
