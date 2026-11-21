# Architecture Context & Engineering Decisions

This reference records the core technical decisions, module boundaries, performance budgets, and design invariants for Find-Me.

---

## 1. Core Architectural Invariants (Zero Workarounds)

1. **100% Pure Rust Stack**:
   - Backend & Crawler: Pure Rust (`jwalk`, `rayon`, `notify`, `nucleo-matcher`).
   - GUI: **Iced** framework (Elm architecture, virtualized rendering). Zero JavaScript, HTML, Electron, or Tauri.
2. **Zero Elevated Privileges**:
   - Runs 100% in unprivileged user space. No `sudo`, no `setuid`, no `CAP_SYS_RAWIO`.
3. **No UI Freezes**:
   - Traversal and heavy filtering run on Tokio/Rayon worker pools; the UI thread maintains 60 FPS under all workloads.
4. **Permanent Branch History**:
   - `main` is protected (no direct code commits).
   - Every task has an isolated feature branch (`feature/<task-id>-<task-name>`).
   - Merges are always non-fast-forward (`--no-ff`).
   - **Feature branches are NEVER deleted**.

---

## 2. Multi-Crate Workspace Layout

```
Find-Me/
├── Cargo.toml                  # Workspace root manifest
├── rustfmt.toml                # Standard formatting configuration
├── AGENTS.md                   # Master agent instruction index (CLAUDE.md equivalent)
├── PHASES.md                   # 10-phase task breakdown roadmap
├── FINDINGS.md                 # Technical research and benchmarks
├── context/                    # Persistent session and state context
├── .agents/skills/             # 9 specialized domain skills
└── crates/
    ├── findme-core/            # Core indexing and search engine
    │   ├── src/lib.rs
    │   ├── src/error.rs        # Strongly-typed thiserror enums
    │   ├── src/record.rs       # 32-byte FileRecord & SearchResultItem
    │   └── src/directory.rs    # Deduplicated DirectoryTable
    ├── findme-ui/              # Pure Rust Iced desktop GUI
    │   ├── src/lib.rs
    │   ├── src/app.rs          # FindMeApp Elm-architecture loop
    │   ├── src/state.rs        # Central AppState
    │   ├── src/message.rs      # Master Message enum
    │   └── src/theme.rs        # Dark & Light minimal desktop theme
    └── findme-cli/             # Headless harness and benchmark utility
        └── src/main.rs
```

---

## 3. Memory & Performance Budgets

Metric | Target Budget | Technical Strategy
:--- | :--- | :---
**Traversal Speed** | $> 200,000$ files / sec | Parallel directory traversal with `jwalk` (Rayon pool)
**Query Latency** | $< 5.0$ ms (1M files) | `nucleo-matcher` + SIMD byte scans + early exit
**Memory Footprint** | $< 45$ MB per 1M files | Contiguous 32-byte `FileRecord` + interned string pool
**Cold Start Time** | $< 50$ ms | Binary zero-copy persistence (`rkyv` / `memmap2`)
**UI Rendering** | Steady $60$ FPS | Virtualized table (`VirtualList`/table) rendering visible rows only

---

## 4. Todoist Task Board Integration

- **Project**: `Everything` (`id: 6j45wf6xPVCM2RJx`)
- **Sections**:
  1. `To Do` (`6j45x63GH3R3vWCQ`): Scheduled tasks from `PHASES.md`
  2. `In Progress` (`6j45x632fhrVMmvQ`): Active task branch
  3. `Paused` (`6j45x634PcW9V5wQ`): Blocked on user input or system dependency
  4. `Completed` (`6j45x63Fqh349Gxx`): Merged and audited tasks
  5. `Future Ideas` (`6j45xVg6MVfrmpWQ`): Post-v1 feature backlog
