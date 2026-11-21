---
name: linux-filesystem-internals
description: >-
  Systems engineering reference for low-level Linux POSIX filesystem operations in Find-Me.
  Covers direct stat syscalls, handling permissions (EACCES), symlink loop detection via inode tracking,
  inotify kernel limits (/proc/sys/fs/inotify), and incremental overflow recovery.
---

# Linux Filesystem Internals & Systems Programming Skill

This skill provides systems-level knowledge, POSIX interfaces, and kernel event handling techniques required to build a rock-solid, crash-proof file indexing engine on Linux.

---

## Core Systems Engineering Principles

1. **Permission Isolation (`EACCES` / `EPERM`)**:
   - Desktop search utilities must traverse user directories and root volumes without stopping or crashing on unreadable paths (e.g. `/root`, `/lost+found`, private directories).
   - Any permission denied error must be gracefully ignored, continuing traversal on sibling nodes.
2. **Symlink Loop Prevention**:
   - Recursive directory crawling can trigger infinite loops if circular symlinks exist (e.g. `dir/link -> dir`).
   - The crawler must maintain an active traversal set of `(device_id, inode)` pairs using `nix::sys::stat::stat` or `std::os::unix::fs::MetadataExt`.
3. **Virtual Filesystem Exclusion**:
   - Linux mounts virtual kernel filesystems that generate dynamic, infinite, or blocking I/O:
     - `/proc`, `/sys`, `/dev`, `/run`, `/tmp`, `/sys/fs/cgroup`.
   - The crawler must automatically exclude these mount points from default indexing.
4. **Kernel Inotify Management**:
   - Linux limits user inotify watches via `/proc/sys/fs/inotify/max_user_watches` (typically 524,288).
   - Handle `IN_Q_OVERFLOW` safely by initiating an incremental scan of the affected tree.

---

## Detailed Reference Guides

- [POSIX Stat & Metadata Guide](./references/posix_stat_and_metadata.md): Extracting sizes, timestamps, modes, device IDs, and inodes with zero allocations.
- [Symlink & Loop Prevention Guide](./references/symlink_and_loop_prevention.md): Cycle detection algorithms using inode sets.
- [Inotify Limits & Overflow Recovery](./references/inotify_limits_and_recovery.md): Managing watch limits and recovering from queue overflows.
