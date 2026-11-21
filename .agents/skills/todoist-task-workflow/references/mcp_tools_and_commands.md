# Todoist MCP Tools & Command Reference

This guide provides tested JSON argument payloads for calling Todoist MCP tools via `call_mcp_tool`.

---

## 1. Finding Tasks (`find-tasks`)

Find all tasks in the Everything project:
```json
{
  "ServerName": "todoist",
  "ToolName": "find-tasks",
  "Arguments": {
    "projectId": "6j45wf6xPVCM2RJx",
    "limit": 50
  }
}
```

---

## 2. Moving a Task to Another Section (`update-tasks`)

Move a task from `To Do` to `In Progress`:
```json
{
  "ServerName": "todoist",
  "ToolName": "update-tasks",
  "Arguments": {
    "tasks": [
      {
        "id": "6j4626Qxrf4Q6vRx",
        "sectionId": "6j45x632fhrVMmvQ"
      }
    ]
  }
}
```

Move a task to `Completed`:
```json
{
  "ServerName": "todoist",
  "ToolName": "update-tasks",
  "Arguments": {
    "tasks": [
      {
        "id": "6j4626Mjv2q39JvQ",
        "sectionId": "6j45x63Fqh349Gxx"
      }
    ]
  }
}
```

---

## 3. Adding Implementation Summary Comment (`add-comments`)

Post an implementation audit trail comment upon task completion:
```json
{
  "ServerName": "todoist",
  "ToolName": "add-comments",
  "Arguments": {
    "comments": [
      {
        "taskId": "6j4626Mjv2q39JvQ",
        "content": "### Implementation Summary\n- **Branch**: `feature/p0-workspace-init`\n- **Merge Commit**: `bc88ada`\n- **Deliverables**: Multi-crate Cargo workspace, findme-core, findme-ui, findme-cli, and rustfmt.\n- **Verification**: `verify_task.sh` passed cleanly with 0 errors and 0 warnings."
      }
    ]
  }
}
```

---

## 4. Adding New Tasks (`add-tasks`)

Add a task to `Future Ideas`:
```json
{
  "ServerName": "todoist",
  "ToolName": "add-tasks",
  "Arguments": {
    "tasks": [
      {
        "projectId": "6j45wf6xPVCM2RJx",
        "sectionId": "6j45xVg6MVfrmpWQ",
        "content": "Windows Everything IPC Protocol Emulation",
        "description": "Expose UNIX domain socket emulating Everything SDK IPC for third-party tool integration.",
        "priority": "p4"
      }
    ]
  }
}
```

---

## 4. Marking a Task Complete (`complete-tasks`)

```json
{
  "ServerName": "todoist",
  "ToolName": "complete-tasks",
  "Arguments": {
    "taskIds": ["6j4626Mjv2q39JvQ"]
  }
}
```
