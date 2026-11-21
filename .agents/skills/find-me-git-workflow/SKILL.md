---
name: find-me-git-workflow
description: >-
  Operational playbook and strict standard for Git branch management, conventional commits,
  non-fast-forward merges, and permanent branch history retention for the Find-Me project.
  Use whenever creating branches, writing commits, reviewing diffs, and merging tasks into main.
---

# Find-Me Git Workflow & Branch Management Skill

This skill enforces strict version control standards, branch lifecycles, and commit hygiene for **Find-Me**. It guarantees that the Git history remains a clean, professional, and permanently auditable record of all completed engineering tasks.

---

## Core Git Directives (Non-Negotiable)

1. **`main` Branch Is Protected**:
   - `main` represents production-ready, passing code.
   - **NEVER write, edit, or commit implementation code directly on `main`**.
2. **Every Task Gets an Isolated Branch**:
   - Every task from [PHASES.md](../../../PHASES.md) must be developed in its own dedicated branch branched from `main`.
   - Branch naming format: `feature/<task-id>-<task-name>` (e.g. `feature/p1-memory-arena`).
3. **Non-Fast-Forward Merges Only (`--no-ff`)**:
   - When merging into `main`, **always** pass `--no-ff`.
   - Fast-forward merges erase branch topology; `--no-ff` creates an explicit merge commit that marks the exact boundaries of the task.
4. **Permanent Branch Retention (NEVER Delete Branches)**:
   - **DO NOT delete feature branches after merging** (`git branch -d` / `-D` is strictly forbidden).
   - All branches remain permanently in the Git repository so any task, commit, or experiment can be inspected or checked out at any time in the future.
5. **Conventional Commits**:
   - Commits must use standard prefixes: `feat:`, `fix:`, `test:`, `docs:`, `perf:`, `refactor:`, `chore:`.

---

## The Standard Branch Lifecycle

```
[ main (Production) ] ───────────────────────────────────────────┬────────► (Merged)
       │                                                         ▲
       │ git checkout -b feature/p1-crawler                      │ git merge --no-ff
       ▼                                                         │
[ feature/p1-crawler ] ──► [ feat: ... ] ──► [ test: ... ] ──────┘ (Branch RETAINED)
```

### 1. Starting a New Task
```bash
# 1. Switch to main and pull latest changes
git checkout main

# 2. Create the feature branch directly from main
git checkout -b feature/<task-id>-<task-name>

# 3. Verify clean start
git status
```

### 2. Staging & Committing Code
```bash
# Check modified and untracked files
git status

# Stage specific changes
git add <paths>

# Commit with Conventional Commits format
git commit -m "feat(<task-id>): <concise description of what changed>"
```

### 3. Pre-Merge Verification
Before merging into `main`, run the project verification gate:
```bash
bash .agents/skills/find-me-qa-verification/scripts/verify_task.sh
```
All checks (`cargo fmt`, `cargo clippy`, `cargo test`) must pass with 0 errors and 0 warnings.

### 4. Merging into `main` & Preserving the Branch
```bash
# 1. Switch back to main
git checkout main

# 2. Merge using non-fast-forward merge
git merge --no-ff feature/<task-id>-<task-name> -m "feat(<task-id>): complete <task description>"

# 3. DO NOT RUN: git branch -d feature/<task-id>-<task-name>
# The feature branch must remain intact permanently!
```

---

## Detailed References

- [Branch Naming Conventions](./references/branch_naming_conventions.md): Exact naming schemes and mappings to `PHASES.md` and Todoist.
- [Conventional Commit Standards](./references/commit_standards.md): Format rules, scope guidelines, and examples.
- [Merge & History Retention Policy](./references/merge_and_history_retention.md): Why `--no-ff` and permanent retention are enforced.
- [Git Command Cheatsheet](./references/git_commands_cheatsheet.md): Practical commands for inspection, diffing, and history graphs.
