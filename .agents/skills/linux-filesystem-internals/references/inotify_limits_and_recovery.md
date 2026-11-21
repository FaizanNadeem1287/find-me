# Inotify Limits & Queue Overflow Recovery

This reference details the Linux kernel's inotify architecture, watch descriptor management, and overflow recovery strategies.

---

## 1. Inotify Kernel Configuration

The Linux kernel regulates inotify via sysctl parameters:

File Path | Default Value | Meaning
:--- | :--- | :---
`/proc/sys/fs/inotify/max_user_watches` | `524,288` (or `65,536`) | Maximum directories a user can watch simultaneously
`/proc/sys/fs/inotify/max_queued_events` | `16,384` | Event queue buffer size before `IN_Q_OVERFLOW` fires
`/proc/sys/fs/inotify/max_user_instances` | `128` | Maximum active inotify handles per user

---

## 2. Handling `IN_Q_OVERFLOW`

When an application performs mass file operations (e.g., `git checkout` changing 100,000 files or `rm -rf node_modules`), events fill the kernel buffer faster than the user-space process can read them.

The kernel emits `IN_Q_OVERFLOW` and **drops subsequent events**.

### Recovery Strategy:
1. **Detect**: Listen for the overflow flag in the event stream.
2. **Debounce**: Wait 500ms for the burst of filesystem activity to subside.
3. **Incremental Rescan**: Schedule a targeted parallel directory re-crawl (`jwalk`) on the watched root directory.
4. **Reconcile**: Atomically update the in-memory arena with the freshly crawled tree.

This guarantees that Find-Me **never** gets out of sync with disk, even during heavy compilation or package installations!
