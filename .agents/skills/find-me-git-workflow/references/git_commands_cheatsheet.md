# Git Command Cheatsheet for Find-Me

A quick reference of common Git operations used during autonomous development.

---

## 1. Branch Management

```bash
# List all local branches with their latest commit
git branch -v

# Create and switch to a new task branch from main
git checkout main
git checkout -b feature/<task-id>-<name>

# Inspect visual branch history graph
git log --graph --oneline --decorate --all -n 25
```

---

## 2. Staging & Inspection

```bash
# Check current branch and modified files
git status

# Inspect exact unstaged differences
git diff

# Inspect staged changes before committing
git diff --staged

# Stage all changes in a specific directory
git add crates/findme-core/
```

---

## 3. Merging & Preserving History

```bash
# Step 1: Switch to main
git checkout main

# Step 2: Merge feature branch with explicit merge commit
git merge --no-ff feature/<task-id>-<name> -m "feat(<task-id>): <concise description>"

# Step 3: Verify merge commit
git log -1 --stat

# Step 4: Verify working tree is clean
git status
```

---

## 4. Reverting or Emergency Recovery

```bash
# Revert an entire merged task from main without rewriting history
git revert -m 1 <merge-commit-hash>

# Discard unstaged local changes in a specific file
git checkout -- <file-path>

# View recent HEAD movements (useful if accidentally detached)
git reflog
```
