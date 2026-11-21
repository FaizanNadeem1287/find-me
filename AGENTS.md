# Find-Me: AI Agent Instructions & Project Index

Welcome to **Find-Me**, an offline-first, high-performance Linux desktop file search application inspired by Windows *Everything*, written in 100% pure Rust with an Iced GUI.

This file serves as the **Master Project Index** (the Antigravity equivalent of `CLAUDE.md`). It is automatically loaded into the agent's context on every session.

---

## 1. Core Engineering Directives (Strict & Non-Negotiable)

1. **100% Pure Rust Stack**:
   - Zero web technologies (no Electron, no Tauri, no WebViews, no JS/HTML/CSS).
   - GUI framework: **Iced** (Elm architecture, virtualized rendering).
   - Core engine: Native Rust (`jwalk`, `nucleo-matcher`, `notify`).
2. **Zero Workarounds Policy**:
   - Only production-grade fixes designed for thousands of users.
   - Zero `unsafe` shortcuts, zero `.unwrap()` or `.expect()` in non-test production code.
   - Zero root/`sudo` privilege requirements; operates 100% within unprivileged user space.
   - Never block the UI thread; long-running operations execute on Tokio or Rayon thread pools.
3. **Protected `main` Branch**:
   - **NEVER write, edit, or commit implementation code directly on `main`**.
   - Every task is developed on a dedicated branch: `feature/<task-id>-<task-name>`.
4. **Permanent Branch Retention**:
   - **DO NOT delete feature branches after merging** (`git branch -d` / `-D` is strictly forbidden).
   - Always merge into `main` using non-fast-forward merges (`git merge --no-ff`).
5. **Live Todoist Synchronization**:
   - Synchronize all task states with the [Everything Todoist Project](https://app.todoist.com/app/project/everything-6j45wf6xPVCM2RJx) (`id: 6j45wf6xPVCM2RJx`).

---

## 2. Master Skills Index (9 Specialized Skills)

All skills reside in [.agents/skills/](./.agents/skills/) and are automatically recognized by Antigravity:

| Skill | Path | Description & Use Case |
| :--- | :--- | :--- |
| **`find-me-rust-dev`** | [.agents/skills/find-me-rust-dev/SKILL.md](./.agents/skills/find-me-rust-dev/SKILL.md) | Core Rust systems programming, memory models, Iced Elm architecture, and Debian packaging. |
| **`find-me-qa-verification`** | [.agents/skills/find-me-qa-verification/SKILL.md](./.agents/skills/find-me-qa-verification/SKILL.md) | 6-tier testing pyramid, headless UI testing (`iced_test`), Criterion benchmarks (<5ms), and `verify_task.sh`. |
| **`find-me-autonomous-dev`** | [.agents/skills/find-me-autonomous-dev/SKILL.md](./.agents/skills/find-me-autonomous-dev/SKILL.md) | 8-step autonomous execution cycle ("vibe coding"), self-healing compiler/borrow-checker guide. |
| **`todoist-task-workflow`** | [.agents/skills/todoist-task-workflow/SKILL.md](./.agents/skills/todoist-task-workflow/SKILL.md) | Todoist MCP runbook, project & section IDs, lifecycle transitions (To Do -> In Progress -> Completed). |
| **`find-me-git-workflow`** | [.agents/skills/find-me-git-workflow/SKILL.md](./.agents/skills/find-me-git-workflow/SKILL.md) | Branch lifecycle, Conventional Commits, non-fast-forward merge policy, and permanent branch retention. |
| **`linux-desktop-integration`** | [.agents/skills/linux-desktop-integration/SKILL.md](./.agents/skills/linux-desktop-integration/SKILL.md) | DBus `FileManager1` file highlighting, OS theme auto-detection, Wayland/X11 clipboard, and desktop entries. |
| **`query-syntax-and-parser`** | [.agents/skills/query-syntax-and-parser/SKILL.md](./.agents/skills/query-syntax-and-parser/SKILL.md) | EBNF query grammar, tokenization, size units, date math, and high-performance AST predicate evaluation. |
| **`linux-filesystem-internals`** | [.agents/skills/linux-filesystem-internals/SKILL.md](./.agents/skills/linux-filesystem-internals/SKILL.md) | POSIX metadata extraction, symlink cycle prevention, virtual filesystem exclusion, and inotify overflow recovery. |
| **`technical-internet-research`** | [.agents/skills/technical-internet-research/SKILL.md](./.agents/skills/technical-internet-research/SKILL.md) | Precision research methodology across Stack Overflow, Reddit, GitHub, Linux man pages, and Google. |

---

## 3. Specialized Subagents Roster (10 Agents)

| Subagent | Role & Primary Domain |
| :--- | :--- |
| **`findme-core-engine`** | Multithreaded crawler (`jwalk`), compact 32-byte in-memory arena, `nucleo-matcher`, and `inotify` watcher. |
| **`findme-ui-developer`** | Pure Rust **Iced** GUI, Elm architecture, virtualized tables (100k+ rows at 60 FPS), themes, and keyboard nav. |
| **`findme-qa-tester`** | Automated unit tests, property-based fuzzing (`proptest`), and headless UI test suites (`iced_test`). |
| **`findme-performance-profiler`** | Latency (<5ms) and throughput (>200k files/s) profiling, Criterion benchmarks, and memory audits. |
| **`findme-code-reviewer`** | Senior Architect auditing code for memory safety, zero workarounds, no unwraps, and non-blocking UI. |
| **`findme-packager-qa`** | Debian `.deb` packaging (`cargo-deb`), FreeDesktop standards, and system installation checks. |
| **`findme-desktop-integrator`** | FreeDesktop standards, DBus file highlighting (`FileManager1`), Wayland/X11 clipboard, and OS theme sync. |
| **`findme-query-architect`** | Query language lexing, parsing, boolean expression trees, and SIMD predicate evaluation. |
| **`findme-researcher`** | Deep technical research (Stack Overflow, Reddit `r/rust`, GitHub, Linux man pages) with zero-workaround vetting. |
| **`findme-todoist-manager`** | Todoist task and board lifecycle management, moving cards across sections, and logging progress. |

---

## 4. Persistent Context Management Protocol (`context/`)

Whenever an agent starts a session or finishes a task, it must maintain the [context/](./context/) directory:
- **[context/CURRENT_STATE.md](./context/CURRENT_STATE.md)**: Must always reflect the active task, active branch, blockers, and next steps.
- **[context/SESSION_LOG.md](./context/SESSION_LOG.md)**: Chronological session audit log. Must be updated with commits, deliverables, and handoff notes.
- **[context/ARCHITECTURE_CONTEXT.md](./context/ARCHITECTURE_CONTEXT.md)**: Core architectural constraints, crate boundaries, and performance budgets.

**At Start of Every Session**: Read `context/CURRENT_STATE.md` and the latest entry of `context/SESSION_LOG.md` to restore full state.
**At End of Every Task**: Update `context/CURRENT_STATE.md` and append to `context/SESSION_LOG.md`.

---

## 5. Quick Development Commands

```bash
# Start a task
git checkout main
git checkout -b feature/<task-id>-<task-name>

# Format code
cargo fmt --all

# Run strict clippy check
cargo clippy --all-targets --all-features -- -D warnings

# Execute test suite
cargo test --workspace

# Run automated pre-merge quality gate
bash .agents/skills/find-me-qa-verification/scripts/verify_task.sh

# Merge into main (NEVER delete the feature branch)
git checkout main
git merge --no-ff feature/<task-id>-<task-name> -m "feat(<task-id>): <description>"
```
