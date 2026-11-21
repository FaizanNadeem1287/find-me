# Find-Me Context Management System

This directory maintains persistent project context, operational states, architectural memory, and cross-session audit logs for **Find-Me**.

Whenever an AI agent starts a new session or finishes a task, it must consult and update these files to preserve 100% continuity.

---

## Directory Index

File | Purpose | Update Frequency
:--- | :--- | :---
[CURRENT_STATE.md](./CURRENT_STATE.md) | Single source of truth for active task, active branch, blockers, and next steps | Every task change or session transition
[ARCHITECTURE_CONTEXT.md](./ARCHITECTURE_CONTEXT.md) | Architectural standards, module responsibilities, crate boundaries, and design invariants | Whenever architecture, crates, or schemas evolve
[SESSION_LOG.md](./SESSION_LOG.md) | Chronological session journal logging every conversation's achievements, commits, and handoffs | At the conclusion of every turn/session

---

## Agent Protocol: Maintaining Context

1. **At Session Start**:
   - Read `context/CURRENT_STATE.md` to instantly restore knowledge of the active task, branch, and blockers.
   - Read the latest entry in `context/SESSION_LOG.md` to see recent actions.
2. **During Task Execution**:
   - Keep `CURRENT_STATE.md` aligned with the active Git branch and Todoist card status.
3. **At Task Completion / Session End**:
   - Update `context/CURRENT_STATE.md` to reflect the newly completed task and next queued task.
   - Append a detailed session summary to `context/SESSION_LOG.md` documenting commits, files created/modified, and verification status.
