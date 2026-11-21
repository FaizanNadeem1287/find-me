# Non-Fast-Forward Merge & Permanent History Retention Policy

This document explains the technical justification for enforcing `--no-ff` merges and the permanent preservation of all feature branches.

---

## 1. Why Fast-Forward Merges Are Forbidden

When Git executes a default fast-forward merge (`git merge feature/...`), it simply slides the `main` pointer forward to the tip of the feature branch:

```
Fast-Forward Merge (Problematic):
A --- B --- C --- D --- E (main)
      \               /
       (Branch disappears into a flat linear history)
```

**Disadvantages of Fast-Forward**:
- Destroys task boundaries: All commits from the feature branch blur into `main`.
- Impossible to revert the entire task cleanly with a single `git revert -m 1 <merge_commit>`.
- Loses the historical context of which task a commit belonged to.

---

## 2. The Non-Fast-Forward Merge (`--no-ff`) Standard

Find-Me strictly mandates `--no-ff` for all merges into `main`:

```bash
git checkout main
git merge --no-ff feature/<task-id>-<task-name> -m "feat(<task-id>): <task description>"
```

```
Non-Fast-Forward Merge (Enforced):
A --- B ------------------------- M (Merge Commit on main)
       \                         /
        C --- D --- E -----------  (feature branch preserved)
```

**Benefits**:
- **Explicit Merge Commit**: The merge commit `M` documents the exact time the task was integrated and validated.
- **Clean Topology**: Running `git log --graph --oneline` clearly visualizes each task branch from its inception to completion.
- **Instant Rollback**: If a regression is discovered later, the entire task can be rolled back safely by reverting the single merge commit `M`.

---

## 3. Permanent Branch Retention Rule

### Rule: NEVER Run `git branch -d` or `git branch -D`

In many teams, branches are deleted after merging to "clean up". In Find-Me, **we explicitly forbid branch deletion**:
1. **Permanent Audit Trail**: Each branch retains the exact branch name (e.g. `feature/p0-workspace-init`, `feature/p1-memory-arena`).
2. **Direct Branch Checkout**: Any developer, QA tester, or user can checkout `feature/p1-parallel-crawler` directly to review its isolated commits or run historical benchmarks.
3. **Traceability to Todoist**: Every task card in Todoist references its permanent Git branch name, providing an unbreakable link between the task board and version control.
