# Zero-Workaround Quality Checklist for Autonomous Coding

Autonomous agents must strictly adhere to these non-negotiable architectural standards during 90% agent-driven coding:

---

## 1. Zero-Workaround Invariants

Invariant | Violation (Workaround) | Production Solution (Proper Fix)
:--- | :--- | :--- |
**No Elevated Root** | Calling `sudo` or requiring `CAP_SYS_RAWIO` to read raw `/dev/` block devices | Multithreaded userspace directory traversal (`jwalk`) across mount points
**No Polling Loops** | `loop { thread::sleep(1s); check_files(); }` | Linux kernel `inotify` event streams via `notify` crate
**No UI Thread Blocking** | Running directory crawling or heavy regex loops directly inside `update()` | Delegating work to Tokio background tasks (`spawn_blocking`) or Rayon thread pools
**No Brittle Shelling** | Shelling out to `sh -c "find / -name ..."` or `grep` | Pure native Rust POSIX APIs via `std::fs`, `jwalk`, and `nix`
**No Web Runtimes** | Embedding Chromium, WebViews, Node.js, or Electron | Pure Rust **Iced** desktop GUI compiled to native machine code
**No Panic Shortcuts** | `.unwrap()` or `.expect()` in non-test code | Explicit error mapping with `thiserror` and graceful UI reporting
**No Heap Churn** | Allocating `String` and `PathBuf` for every file | Interned string pool and flat 32-byte `FileRecord` structures
**No Branch Erasure** | Deleting feature branches after merging | Retaining all feature branches permanently for complete audit history

---

## 2. Pre-Merge Verification Self-Audit

Before an agent proposes merging a feature branch into `main`:
1. [ ] Was this task developed on an isolated feature branch?
2. [ ] Does `bash .agents/skills/find-me-qa-verification/scripts/verify_task.sh` pass cleanly with zero errors and zero warnings?
3. [ ] Are all unit tests, integration tests, or headless UI tests passing?
4. [ ] Are there zero `.unwrap()` or `.expect()` calls in production paths?
5. [ ] Is the branch ready for a non-fast-forward merge (`git merge --no-ff`)?
