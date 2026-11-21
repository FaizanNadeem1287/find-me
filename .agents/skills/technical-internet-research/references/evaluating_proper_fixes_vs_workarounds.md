# Evaluating Proper Fixes vs. Workarounds

A rigorous checklist for vetting internet code snippets before adding them to Find-Me.

---

## 1. Red Flags (Identify & Reject Workarounds)

If a forum or Stack Overflow answer proposes any of the following, **DO NOT COPY IT**:
- ❌ **`unsafe { ... }` without an airtight SAFETY comment and justification**: Often used to bypass the borrow checker inappropriately.
- ❌ **`.unwrap()` or `.expect("should not fail")`**: Hides crash scenarios under unusual input.
- ❌ **`thread::sleep(Duration::from_millis(...))`**: An arbitrary hack used to "wait" for async tasks or file updates; causes flaky tests.
- ❌ **Shelling out to `/bin/bash` or `sh -c`**: Spawning subshells instead of using native Rust APIs is slow and fragile.
- ❌ **Hardcoded `/tmp` paths**: Vulnerable to symlink attacks; always use `tempfile` or XDG directories.
- ❌ **Demanding Root/Sudo**: Asking users to run desktop apps with elevated privileges violates security standards.

---

## 2. Green Flags (Accept Proper Solutions)

A researched solution is high quality and acceptable if it demonstrates:
-  **Deterministic Lifetime Scoping**: Uses `{ ... }` blocks or data restructuring to drop locks/borrows cleanly.
-  **Strong Error Propagation**: Returns `Result<T, EngineError>` with contextual error descriptions.
-  **Non-Blocking Concurrency**: Uses Tokio channels (`mpsc`, `watch`) and `tokio::task::spawn_blocking` for CPU/IO heavy tasks.
-  **Resource Isolation**: Uses XDG specifications (`ProjectDirs`), `tempfile::tempdir()`, and POSIX error checks.
-  **Documented Edge-Case Handling**: Explicitly accounts for permissions (`EACCES`), unmounted devices, and malformed user strings.
