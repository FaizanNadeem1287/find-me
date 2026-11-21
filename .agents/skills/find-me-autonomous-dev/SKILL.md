---
name: find-me-autonomous-dev
description: >-
  Operational playbook and runbook for autonomous, agentic development ("vibe coding") of Find-Me.
  Guides the autonomous agent through end-to-end task execution: feature branching, test-driven
  implementation, self-healing Rust compiler diagnostics, automated headless UI validation,
  architectural review, and branch merging with permanent history retention.
---

# Autonomous Development & Agentic Workflow Playbook

This skill defines the autonomous execution loop for developing **Find-Me**. It empowers the AI agent to independently drive 90% of implementation, testing, debugging, and verification with engineering rigor, requiring only 10% high-level directional oversight and review from the user.

---

## The Autonomous Task Execution Cycle

For every task specified in [PHASES.md](../../../PHASES.md), the agent follows this 8-step autonomous cycle:

```
[ Step 1: Ingest Spec & Criteria ]
               |
               v
[ Step 2: Create Feature Branch ] ---------> (feature/<task-id>-<name> & Move to "In Progress" in Todoist)
               |
               v
[ Step 3: Write Test Fixture (TDD) ] -------> (Failing unit / headless test)
               |
               v
[ Step 4: Write Rust Implementation ] ------> (No unwraps, thiserror, compact structs)
               |
               v
[ Step 5: Self-Healing Compile Loop ] ------> (cargo check/clippy auto-fix loop)
               |
               v
[ Step 6: Run QA Verification Gate ] -------> (bash verify_task.sh)
               |
               v
[ Step 7: Architecture Review ] ------------> (Zero-workaround compliance)
               |
               v
[ Step 8: Merge, Retain & Complete ] -------> (git merge --no-ff, keep branch & Move to "Completed" in Todoist)
```

---

## 1. Step-by-Step Autonomous Workflow

### Step 1: Ingest Spec & Criteria
- Read the active task definition in [PHASES.md](../../../PHASES.md).
- Identify dependencies, input/output schemas, and acceptance criteria.

### Step 2: Create Feature Branch & Sync Todoist
```bash
git checkout main
git checkout -b feature/<task-id>-<task-name>
```
- In the Todoist project **[Everything](https://app.todoist.com/app/project/everything-6j45wf6xPVCM2RJx)**, move the corresponding task from `To Do` to `In Progress`.
- If blocked at any point by external factors or awaiting feedback, move the task to `Paused`.

### Step 3: Write Tests First (TDD)
- **Engine Tasks**: Write unit tests in `src/` or integration tests in `tests/` using `tempfile`.
- **Query Parser Tasks**: Write property tests in `tests/proptest_*.rs`.
- **UI Tasks**: Write headless UI tests using `iced_test::simulator`.

### Step 4: Pure Rust Implementation
- Implement the required structs, traits, and functions.
- Follow the zero-workaround policy:
  - No `unwrap()` or `expect()` in non-test code.
  - Strongly typed error handling via `thiserror`.
  - No blocking filesystem operations on async/UI threads (`spawn_blocking` or Rayon).
  - Compact memory footprint (32-byte records, string interning).

### Step 5: Self-Healing Compiler & Clippy Loop
When encountering compiler or linter errors, follow the [Self-Healing Rust Compiler Guide](./references/self_healing_rust_compiler.md):
1. Run `cargo check --message-format=json` or `cargo clippy`.
2. Inspect exact error codes (`E0382`, `E0502`, `E0597`, etc.).
3. Apply structured fixes without guessing (restructuring lifetimes, using `Arc`/`parking_lot`, or scoping borrows).
4. Run `cargo fmt --all` to format code automatically.

### Step 6: Automated Pre-Merge Quality Gate
Execute the project verification script:
```bash
bash .agents/skills/find-me-qa-verification/scripts/verify_task.sh
```
All 5 verification gates (formatting, clippy, unit/integration tests, doc tests, git status) must pass with zero errors and zero warnings.

### Step 7: Architectural Self-Audit
Verify compliance with the [Zero-Workaround Checklist](./references/zero_workaround_checklist.md):
- Does the code run 100% in user-space without root/sudo?
- Are long-running operations off the UI thread?
- Is memory consumption within budget (<45MB / 1M files)?

### Step 8: Merge into `main`, Retain Branch & Complete in Todoist
```bash
git checkout main
git merge --no-ff feature/<task-id>-<task-name> -m "feat(<task-id>): <task description>"
# CRITICAL: DO NOT DELETE THE FEATURE BRANCH!
```
- **Post Audit Comment in Todoist**: Call `add-comments` on the Todoist task with a detailed implementation summary (branch name, merge commit hash, deliverables, and verification pass status).
- **Move to Completed**: Move the task from `In Progress` into `Completed` (`sectionId: "6j45x63Fqh349Gxx"`).

---

## Detailed References

- [Self-Healing Rust Compiler Guide](./references/self_healing_rust_compiler.md): Algorithmic solutions for common rustc and borrow checker errors.
- [Agentic Testing & Mocking Guide](./references/agentic_testing_and_mocking.md): How autonomous agents generate robust test fixtures, property fuzz tests, and headless Iced UI mocks.
- [Zero-Workaround Checklist](./references/zero_workaround_checklist.md): Non-negotiable architectural constraints for production quality.
