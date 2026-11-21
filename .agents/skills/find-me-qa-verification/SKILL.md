---
name: find-me-qa-verification
description: >-
  Comprehensive guide and procedural runbook for testing, debugging, and verifying Find-Me tasks.
  Use whenever completing or reviewing a task in Find-Me to verify stability, execute unit/integration
  tests, run headless Iced UI tests, perform property-based fuzzing, validate performance benchmarks,
  and enforce the pre-merge quality gate before merging into main.
---

# Find-Me QA, Testing & Task Verification Skill

This skill provides step-by-step instructions, standard testing methodologies, and automated verification procedures to ensure that every feature, bug fix, and optimization in **Find-Me** is 100% stable, fully tested, and regression-free.

---

## The Testing Pyramid for Find-Me

Find-Me employs a comprehensive multi-layered testing strategy designed specifically for high-performance desktop applications:

```
                  / \
                 /   \       Manual Smoke & Release Packaging (.deb)
                /     \      (Installed binary, desktop entry, Wayland/X11)
               /       \
              /---------\    Performance & Benchmarking (criterion)
             /           \   (Throughput >200k files/s, query latency <5ms)
            /-------------\  Headless UI Testing (iced_test Simulator)
           /               \ (Deterministic message/state loops, no window)
          /-----------------\ Integration Testing (tempfile, notify, jwalk)
         /                   \ (Real directory trees, live inotify events)
        /---------------------\ Property-Based Fuzzing (proptest)
       /                       \ (Fuzzing query parser with millions of inputs)
      /-------------------------\ Unit Testing (cargo test --lib)
     /                           \ (Data structures, compact arena, tokenizers)
```

---

## Procedural Workflow: Verifying a Task Before Merge

Whenever you finish working on a task in a feature branch (e.g. `feature/p1-memory-arena`), execute this procedure before asking to merge into `main`:

### Step 1: Run the Automated Verification Script
Execute the pre-merge verification script from the repository root:
```bash
bash .agents/skills/find-me-qa-verification/scripts/verify_task.sh
```
This runs:
1. `cargo fmt -- --check`: Verifies idiomatic Rust formatting.
2. `cargo clippy --all-targets --all-features -- -D warnings`: Enforces strict zero-warning linter rules.
3. `cargo test --workspace`: Executes all unit, integration, and doc tests across all crates.
4. `cargo test --workspace -- --ignored`: Executes extended integration tests.

### Step 2: Run Task-Specific Tests
Depending on the component modified:
- **Engine / Crawler Changes**: Run integration tests with temporary filesystem fixtures (`tests/integration_crawler.rs`).
- **Query Parser Changes**: Run property-based tests (`cargo test --test proptest_query`).
- **UI Component Changes**: Run headless UI tests using `iced_test` (`cargo test -p findme-ui`).
- **Performance-Sensitive Code**: Run micro-benchmarks (`cargo bench --no-run` to compile, `cargo bench` to evaluate).

### Step 3: Run the Task Completion Checklist
Verify every item in the [Task Completion Checklist](./references/task_completion_checklist.md):
- [ ] No `unwrap()` or `expect()` in reachable production paths.
- [ ] No unhandled errors (all mapped to `thiserror` enums).
- [ ] All new functions/structs have doc comments and corresponding unit tests.
- [ ] No hardcoded `/proc` or `sleep()` polling loops.
- [ ] All changes committed to the feature branch.
- [ ] Feature branch ready for non-fast-forward merge (`git merge --no-ff`) into `main`.

---

## Detailed Reference Guides

- [Testing Pyramid & Strategy](./references/testing_pyramid_and_strategy.md): In-depth breakdown of each testing type and why it matters.
- [Headless Iced UI Testing Guide](./references/iced_ui_testing_guide.md): Testing Iced desktop UI without an X11/Wayland display server using `iced_test`.
- [Engine, Concurrency & Filesystem Testing](./references/engine_and_concurrency_testing.md): Testing `jwalk` parallel traversal and `inotify` file watchers without flaky delays.
- [Performance Benchmarking & Regression Detection](./references/performance_and_benchmarking.md): Latency thresholds (<5ms), crawl benchmarks, and memory profiling.
- [Task Completion Checklist](./references/task_completion_checklist.md): Mandatory pre-merge gate checklist.
