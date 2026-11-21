---
name: todoist-task-workflow
description: >-
  Operational runbook and reference for managing Find-Me project tasks, board states,
  and lifecycle transitions in Todoist using the Todoist MCP server. Use whenever starting,
  pausing, updating, or completing tasks in the Everything Todoist project.
---

# Todoist Task Management & Workflow Skill

This skill provides step-by-step procedures, tool schemas, project/section IDs, and lifecycle rules for managing tasks in the **[Everything](https://app.todoist.com/app/project/everything-6j45wf6xPVCM2RJx)** Todoist project.

---

## 1. Quick Reference: Project & Section Identifiers

All tasks for Find-Me belong to the **Everything** project:

Entity | Name | ID | Description / Purpose
:--- | :--- | :--- | :---
**Project** | `Everything` | `6j45wf6xPVCM2RJx` | Primary project board for Find-Me development
**Section 1** | `To Do` | `6j45x63GH3R3vWCQ` | Backlog of planned tasks from `PHASES.md` ready for execution
**Section 2** | `In Progress` | `6j45x632fhrVMmvQ` | Active task currently under development in a feature branch
**Section 3** | `Paused` | `6j45x634PcW9V5wQ` | Tasks temporarily blocked on dependencies or awaiting user input
**Section 4** | `Completed` | `6j45x63Fqh349Gxx` | Tasks verified via `verify_task.sh` and merged into `main`
**Section 5** | `Future Ideas` | `6j45xVg6MVfrmpWQ` | Long-term roadmap items, extra packaging formats, and post-v1 ideas

---

## 2. Standard Task Lifecycle Procedures

### Procedure 1: Starting a Task (To Do -> In Progress)
When starting work on a new phase or task:
1. Create and switch to the dedicated feature branch:
   ```bash
   git checkout main
   git checkout -b feature/<task-id>-<name>
   ```
2. Locate the task in Todoist using `find-tasks` (filtered by `projectId: "6j45wf6xPVCM2RJx"`).
3. Move the task to **In Progress** using `update-tasks`:
   ```json
   {
     "tasks": [
       {
         "id": "<task_id>",
         "sectionId": "6j45x632fhrVMmvQ"
       }
     ]
   }
   ```
4. Verify exactly **one** task is in `In Progress` at any time.

---

### Procedure 2: Pausing a Blocked Task (In Progress -> Paused)
If work is blocked by external factors (e.g. system packages, credentials, user direction):
1. Update the task description with the blocker details.
2. Move the task to **Paused** using `update-tasks`:
   ```json
   {
     "tasks": [
       {
         "id": "<task_id>",
         "sectionId": "6j45x634PcW9V5wQ",
         "description": "**Status: PAUSED**\nWaiting on: <blocker reason>"
       }
     ]
   }
   ```

---

### Procedure 3: Completing a Task (In Progress -> Completed)
When development is finished:
1. Run the automated quality gate:
   ```bash
   bash .agents/skills/find-me-qa-verification/scripts/verify_task.sh
   ```
2. Perform code review audit (no unwraps, zero workarounds, non-blocking UI).
3. Merge into `main` using non-fast-forward merge and **preserve the feature branch**:
   ```bash
   git checkout main
   git merge --no-ff feature/<task-id>-<name> -m "feat(<task-id>): <description>"
   ```
4. **Post Implementation Audit Comment**: Call `add-comments` on the Todoist task to record:
   - Git branch name and merge commit hash
   - Summary of features, structs, and interfaces implemented
   - Key files created/modified
   - QA verification status (`verify_task.sh` passed)
   ```json
   {
     "comments": [
       {
         "taskId": "<task_id>",
         "content": "### Implementation Summary\n- **Branch**: `feature/<task-id>-<name>`\n- **Merge Commit**: `<commit-hash>`\n- **Deliverables**: <summary of code>\n- **Verification**: `verify_task.sh` passed (fmt, clippy, unit/doc tests)."
       }
     ]
   }
   ```
5. Move the task to **Completed** in Todoist:
   ```json
   {
     "tasks": [
       {
         "id": "<task_id>",
         "sectionId": "6j45x63Fqh349Gxx"
       }
     ]
   }
   ```
5. Optionally mark completed using `complete-tasks` tool.

---

### Procedure 4: Adding Future Ideas
When the user or agent identifies a post-v1 feature or alternative packaging target:
1. Add to the **Future Ideas** section (`sectionId: "6j45xVg6MVfrmpWQ"`) using `add-tasks`.
2. Set priority to `p4` (lowest/backlog).

---

## Detailed References

- [Board Structure & IDs Reference](./references/board_structure_and_ids.md): Complete list of IDs, labels, and priorities.
- [MCP Tools & Command Reference](./references/mcp_tools_and_commands.md): Examples for all Todoist MCP tool calls.
- [Task Lifecycle Procedures](./references/task_lifecycle_procedures.md): Step-by-step checklists for transitions.
