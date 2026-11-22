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

1. **`main` Branch Is Protected & Mandatory PR Review Gate**:
   - `main` represents production-ready, passing code.
   - **NEVER write, edit, or commit implementation code directly on `main`**.
   - **NEVER merge into `main` directly without user PR review and approval**.
   - Every completed task must be pushed to GitHub (`git push -u origin feature/<task-id>-<task-name>`) and submitted as a Pull Request against `main`.
   - **STOP & WAIT** for the user to review the PR before merging.
2. **Every Task Gets an Isolated Branch**:
   - Every task from [PHASES.md](../../../PHASES.md) must be developed in its own dedicated branch branched from `main`.
   - Branch naming format: `feature/<task-id>-<task-name>` (e.g. `feature/p1-memory-arena`).
3. **Pull Request Workflow & Review Process**:
   - Create PR using GitHub CLI (`gh pr create`) or via GitHub Web.
   - Summarize changes, deliverables, and test coverage in PR body.
   - The user reviews the PR diff on GitHub. Address any requested changes on the feature branch.
4. **Non-Fast-Forward Merges Only (`--no-ff`) Upon Approval**:
   - When merging approved PRs into `main`, **always** pass `--no-ff`.
   - Fast-forward merges erase branch topology; `--no-ff` creates an explicit merge commit that marks the exact boundaries of the task.
   - Push updated `main` to `origin` immediately after merging.
5. **Permanent Branch Retention (NEVER Delete Branches)**:
   - **DO NOT delete feature branches after merging** (`git branch -d` / `-D` is strictly forbidden).
   - All branches remain permanently in the Git repository and on GitHub remote (`origin`) so any task, commit, or experiment can be inspected or checked out at any time in the future.
6. **Conventional Commits**:
   - Commits must use standard prefixes: `feat:`, `fix:`, `test:`, `docs:`, `perf:`, `refactor:`, `chore:`.

---

## The Standard Branch & PR Lifecycle

```
[ main (Production) ] ─────────────────────────────────────────────────────────────┬────────► (Merged)
       │                                                                           ▲
       │ git checkout -b feature/p1-crawler                                        │ git merge --no-ff
       ▼                                                                           │ (Only after user PR review)
[ feature/p1-crawler ] ──► [ code & test ] ──► [ git push origin ] ──► [ GitHub PR ]
```

### 1. Starting a New Task
```bash
# 1. Switch to main and pull latest changes
git checkout main
git pull origin main

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
Before opening a PR, run the automated verification gate:
```bash
bash .agents/skills/find-me-qa-verification/scripts/verify_task.sh
```
All checks (`cargo fmt`, `cargo clippy`, `cargo test`) must pass with 0 errors and 0 warnings.

### 4. Pushing Feature Branch to GitHub
```bash
git push -u origin feature/<task-id>-<task-name>
```

### 5. Creating Pull Request against `main`
```bash
# Via GitHub CLI:
gh pr create --base main --head feature/<task-id>-<task-name> \
  --title "feat(<task-id>): <concise title>" \
  --body "### Summary
- Implementation details...
- Automated tests passing via verify_task.sh"

# Or navigate to GitHub compare URL:
# https://github.com/FaizanNadeem1287/find-me/compare/main...feature/<task-id>-<task-name>?expand=1
```

### 6. Mandatory PR Review Gate (User Review & Feedback)
- **Notify the user** that the feature branch has been pushed and the PR is open for review.
- **STOP & WAIT** for the user's review and explicit instructions/approval.
- If changes or fixes are requested during review, commit them on the feature branch and push to `origin`.

### 7. Merging into `main` (ONLY After Approval) & Permanent Retention
```bash
# 1. Switch back to main
git checkout main
git pull origin main

# 2. Merge using non-fast-forward merge
git merge --no-ff feature/<task-id>-<task-name> -m "merge: feat(<task-id>): complete <task description>"

# 3. Push merged main to GitHub
git push origin main

# 4. PERMANENT BRANCH RETENTION: DO NOT DELETE THE FEATURE BRANCH!
# (git branch -d is strictly forbidden)
```

---

## Detailed References

- [Branch Naming Conventions](./references/branch_naming_conventions.md): Exact naming schemes and mappings to `PHASES.md` and Todoist.
- [Conventional Commit Standards](./references/commit_standards.md): Format rules, scope guidelines, and examples.
- [Merge & History Retention Policy](./references/merge_and_history_retention.md): Why `--no-ff` and permanent retention are enforced.
- [Git Command Cheatsheet](./references/git_commands_cheatsheet.md): Practical commands for inspection, diffing, and history graphs.
