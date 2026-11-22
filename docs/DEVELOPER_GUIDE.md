# Developer & Contributor Guide

This document outlines development environment setup, coding guidelines, automated testing, quality gates, and git branch workflows for contributors to Find-Me.

---

## 1. Toolchain & Prerequisites

* **Rust Toolchain**: Stable 1.80+ (managed via `rustup`).
* **C Compiler & Linker**: `gcc` or `clang`, `pkg-config`.
* **System Libraries**:
  ```bash
  # Debian / Ubuntu
  sudo apt install -y build-essential pkg-config libx11-dev libxcursor-dev libxrandr-dev libxi-dev libxkbcommon-dev libfontconfig1-dev
  ```

---

## 2. Core Engineering Directives

All contributors and AI agents must adhere to the project's strict engineering directives:

1. **100% Pure Rust Stack**: Zero web technologies (no Electron, no Tauri, no WebViews, no JS/CSS runtimes). GUI is strictly pure Rust [Iced](https://github.com/iced-rs/iced).
2. **Zero Workarounds Policy**: Only production-grade solutions designed for thousands of users.
3. **No Unwraps**: Zero `.unwrap()` or `.expect()` in non-test production code. All errors must be handled gracefully with `Result` or `Option`.
4. **Zero `unsafe`**: No `unsafe` shortcuts or pointer casts.
5. **Non-Blocking UI**: UI thread must never block. All I/O, crawling, and embedding run on Tokio/Rayon worker thread pools.
6. **Protected `main` Branch & PR Review Gate**: Never commit directly to `main`. Every task develops on a feature branch (`feature/<task-id>-<task-name>`), is pushed to GitHub, and submitted as a Pull Request against `main`. Changes are reviewed before merging into `main`.
7. **Permanent Branch Retention**: Feature branches are **never deleted** (`git branch -d` / `-D` is strictly forbidden). Merges to `main` must use non-fast-forward merges (`git merge --no-ff`) and push to `origin`.

---

## 3. Development Commands

### Check & Lint
```bash
# Verify formatting
cargo fmt --all -- --check

# Strict Clippy check (warnings treated as errors)
cargo clippy --all-targets --all-features -- -D warnings
```

### Run Tests
```bash
# Execute full workspace test suite
cargo test --workspace

# Run single crate tests
cargo test -p findme-semantic
cargo test -p findme-ui --test semantic_ui_tests
```

### Pre-Merge Automated Quality Gate
Before merging any feature branch into `main`, execute the automated quality gate:
```bash
bash .agents/skills/find-me-qa-verification/scripts/verify_task.sh
```
This runs:
1. `cargo fmt --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. Full workspace unit and integration test suite
4. Headless UI test suites (`iced_test`)
5. Criterion latency budget check (< 5 ms for 100k records)

---

## 4. Git Branch, Pull Request & Review Workflow

```bash
# 1. Start feature branch from main
git checkout main
git pull origin main
git checkout -b feature/<task-id>-<task-name>

# 2. Develop with tests and verify quality gate
cargo test -p <affected-crate>
cargo clippy --all-targets --all-features -- -D warnings
bash .agents/skills/find-me-qa-verification/scripts/verify_task.sh

# 3. Commit with Conventional Commits
git commit -m "feat(<task-id>): <clear actionable description>"

# 4. Push feature branch to GitHub remote
git push -u origin feature/<task-id>-<task-name>

# 5. Create Pull Request against main
gh pr create --base main --head feature/<task-id>-<task-name> --title "feat(<task-id>): <description>" --body "<deliverables>"

# 6. STOP & WAIT FOR USER PR REVIEW
# Do not merge directly! Wait for user review and explicit approval.

# 7. Merge into main ONLY after user approval using non-fast-forward merge
git checkout main
git pull origin main
git merge --no-ff feature/<task-id>-<task-name> -m "merge: feat(<task-id>): <description>"
git push origin main

# 8. PERMANENT BRANCH RETENTION: Do NOT delete the feature branch!
```
