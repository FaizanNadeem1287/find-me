# Branch Naming Conventions & Taxonomy

This reference defines the standardized branch naming convention for Find-Me, ensuring 1:1 traceability between Git branches, [PHASES.md](../../../PHASES.md) tasks, and Todoist cards.

---

## 1. Branch Naming Structure

Every branch name follows this pattern:
```
<type>/<task-id>-<short-description>
```

### Prefix Types:
Prefix | Purpose | Example
:--- | :--- | :---
`feature/` | New phase tasks, UI components, or engine features | `feature/p1-memory-arena`
`bugfix/` | Correcting a bug, memory leak, or regression | `bugfix/fix-inotify-overflow`
`perf/` | Pure performance optimization or benchmark tuning | `perf/simd-string-matcher`
`docs/` | Architecture documentation, skills, and roadmaps | `docs/update-threat-model`

---

## 2. Complete Phase-to-Branch Mapping Matrix

Phase | Task ID | Branch Name | Matched Todoist Task
:--- | :--- | :--- | :---
**Phase 0** | Task 0.1 | `feature/p0-workspace-init` | Task 0.1: Initialize Cargo Workspace Structure
**Phase 1** | Task 1.1 | `feature/p1-memory-arena` | Task 1.1: Compact In-Memory Arena & String Pool
**Phase 1** | Task 1.2 | `feature/p1-parallel-crawler` | Task 1.2: Parallel Directory Crawler (jwalk / Rayon)
**Phase 2** | Task 2.1 | `feature/p2-query-parser` | Task 2.1: Search Query Tokenizer & Parser
**Phase 2** | Task 2.2 | `feature/p2-search-matcher` | Task 2.2: High-Performance Search Matching Engine
**Phase 3** | Task 3.1 | `feature/p3-fs-watcher` | Task 3.1: Real-Time Filesystem Monitoring (notify / inotify)
**Phase 3** | Task 3.2 | `feature/p3-index-persistence` | Task 3.2: Persistent Index Storage & Instant Startup
**Phase 4** | Task 4.1 | `feature/p4-ui-shell` | Task 4.1: Iced Application Shell & Navigation
**Phase 4** | Task 4.2 | `feature/p4-ui-theming` | Task 4.2: Cohesive Dark & Light Theming
**Phase 5** | Task 5.1 | `feature/p5-search-input` | Task 5.1: Main Search Input & Status Indicators
**Phase 5** | Task 5.2 | `feature/p5-virtual-table` | Task 5.2: Virtualized Tabular Results Component
**Phase 6** | Task 6.1 | `feature/p6-advanced-screen` | Task 6.1: Advanced Search Screen
**Phase 6** | Task 6.2 | `feature/p6-index-mgmt` | Task 6.2: Index Management Screen
**Phase 6** | Task 6.3 | `feature/p6-remaining-screens` | Task 6.3: Search History, Settings & About Screens
**Phase 7** | Task 7.1 | `feature/p7-engine-integration` | Task 7.1: Asynchronous UI <-> Engine Bridge
**Phase 8** | Task 8.1 | `feature/p8-desktop-integration` | Task 8.1: File Context Menu & Desktop Integration
**Phase 8** | Task 8.2 | `feature/p8-keyboard-nav` | Task 8.2: Comprehensive Keyboard Navigation
**Phase 9** | Task 9.1 | `feature/p9-debian-packaging` | Task 9.1: Debian Packaging with cargo-deb
**Phase 9** | Task 9.2 | `feature/p9-qa-benchmarks` | Task 9.2: Performance Benchmarking & QA Verification
