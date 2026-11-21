# Board Structure & IDs Reference

This reference details the project, section, and attribute schema for the Todoist integration.

---

## 1. Project Details

- **Project Name**: `Everything`
- **Project ID**: `6j45wf6xPVCM2RJx`
- **URL**: `https://app.todoist.com/app/project/everything-6j45wf6xPVCM2RJx`
- **Color**: `charcoal`
- **View Style**: `board` / `list`

---

## 2. Section Details

| Section Name | Section ID | Order | Purpose |
| :--- | :--- | :---: | :--- |
| **To Do** | `6j45x63GH3R3vWCQ` | 1 | Backlog of tasks awaiting execution. |
| **In Progress** | `6j45x632fhrVMmvQ` | 2 | Currently active task being implemented. Only 1 active task at a time. |
| **Paused** | `6j45x634PcW9V5wQ` | 3 | Tasks blocked on external dependencies or user feedback. |
| **Completed** | `6j45x63Fqh349Gxx` | 4 | Tasks tested, reviewed, and merged into `main`. |
| **Future Ideas** | `6j45xVg6MVfrmpWQ` | 5 | Long-term roadmap, extra package formats, and enhancements. |

---

## 3. Priority Mapping

Priority | String in MCP | Usage in Find-Me
:--- | :--- | :---
**Priority 1 (Red)** | `"p1"` | Critical path / core architecture (Crawler, Matcher, UI Table, Debian packaging)
**Priority 2 (Orange)** | `"p2"` | Standard phase feature tasks (Memory arena, Inotify, Search bar, Keyboard nav)
**Priority 3 (Blue)** | `"p3"` | Secondary UI screens, theming, settings, and documentation
**Priority 4 (Default)** | `"p4"` | Future ideas, optional enhancements, and backlog items
