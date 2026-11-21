# Task Lifecycle Procedures

Step-by-step checklists to follow when managing tasks during autonomous development.

---

## Checklist: Starting a Task
1. [ ] Check current `In Progress` section in Todoist. Confirm no other task is active.
2. [ ] Identify next task from `To Do` section.
3. [ ] Checkout `main` and pull latest commits.
4. [ ] Create new feature branch: `git checkout -b feature/<task-id>-<name>`.
5. [ ] Call `update-tasks` to move the task to `In Progress` (`sectionId: "6j45x632fhrVMmvQ"`).

---

## Checklist: Pausing a Task
1. [ ] Identify the blocking issue (e.g. system dependencies, missing user direction).
2. [ ] Commit all work in progress on the feature branch.
3. [ ] Update task description with blocker reason.
4. [ ] Call `update-tasks` to move the task to `Paused` (`sectionId: "6j45x634PcW9V5wQ"`).
5. [ ] Report blocker to the user.

---

## Checklist: Completing a Task
1. [ ] Execute verification gate: `bash .agents/skills/find-me-qa-verification/scripts/verify_task.sh`.
2. [ ] Confirm 0 errors and 0 warnings across formatting, clippy, unit tests, and doc tests.
3. [ ] Perform architect review (no `unwrap()`, zero workarounds, non-blocking UI).
4. [ ] Checkout `main`: `git checkout main`.
5. [ ] Merge with non-fast-forward: `git merge --no-ff feature/<task-id>-<name> -m "feat(<task-id>): <description>"`.
6. [ ] **Do NOT delete the feature branch!** Keep it intact.
7. [ ] Call `update-tasks` to move the task to `Completed` (`sectionId: "6j45x63Fqh349Gxx"`).
