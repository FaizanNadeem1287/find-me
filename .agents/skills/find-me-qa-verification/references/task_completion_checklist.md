# Task Completion & Pre-Merge Verification Checklist

Every developer and subagent working on Find-Me **MUST** complete and verify this checklist before a feature branch can be approved and merged into `main`.

---

## Pre-Merge Quality Gate Checklist

### 1. Code Standards & Linting
- [ ] **Formatting**: Code formatted with standard settings (`cargo fmt --all -- --check`).
- [ ] **Zero Warnings**: Clippy executed in strict mode (`cargo clippy --all-targets --all-features -- -D warnings`) with 0 warnings reported.
- [ ] **No Dead Code**: No unused imports, variables, or unreachable branches left in production files.
- [ ] **Proper Visibility**: Structs and functions are `pub(crate)` or private unless explicitly required by the public API.

### 2. Error Handling & Safety
- [ ] **No Unwraps**: Zero instances of `.unwrap()` or `.expect()` in non-test production code where input or filesystem state can trigger a panic.
- [ ] **Typed Errors**: All errors are represented using `thiserror` typed enums with clear, human-readable error messages.
- [ ] **No Workarounds**: Zero shell hacks, no hardcoded `/proc` reading, no unneeded `sudo`/root requirements.

### 3. Testing & Verification
- [ ] **Unit Tests**: Every new function, parser token, and data structure has matching unit tests in `src/`.
- [ ] **Integration Tests**: Any filesystem or async feature has integration tests in `tests/` using `tempfile`.
- [ ] **No Flaky Tests**: Zero arbitrary `thread::sleep()` delays. All async tests use Tokio channels and bounded timeouts.
- [ ] **All Tests Passing**: Full test suite passes cleanly: `cargo test --workspace --all-targets`.
- [ ] **Doc Tests Passing**: Code examples in documentation pass: `cargo test --workspace --doc`.

### 4. Headless UI Verification (For UI Tasks)
- [ ] **Elm State Integrity**: New messages correctly update state and emit expected commands.
- [ ] **Headless Test Coverage**: `iced_test` headless simulator tests verify widget clicks, keyboard shortcuts, and state transitions.
- [ ] **Theme Support**: Both Light and Dark themes verified.
- [ ] **60 FPS Scrolling**: Virtualized rendering (`VirtualList`/`table`) used for lists with $>100$ items.

### 5. Git & Branching Audit Trail
- [ ] **Isolated Branch**: All commits are isolated within a dedicated `feature/<task-id>-<task-name>` branch.
- [ ] **Descriptive Commits**: Commit messages follow Conventional Commits (`feat:`, `fix:`, `test:`, `docs:`, `refactor:`).
- [ ] **Non-Fast-Forward Merge**: Merged into `main` using `git merge --no-ff`.
- [ ] **Permanent Retention**: **Feature branch is NOT deleted** after merging.
