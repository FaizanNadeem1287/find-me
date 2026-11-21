# Symlink & Cycle Prevention Guide

This guide details how Find-Me prevents infinite recursion and stack overflows when traversing filesystem trees containing circular symlinks.

---

## 1. The Circular Symlink Hazard

A circular symlink occurs when a directory contains a symbolic link pointing back to an ancestor directory:
```
/home/user/docs/loop -> /home/user/docs
```
A naive recursive crawler traversing this without cycle detection will run indefinitely, allocating millions of path strings until memory is exhausted or the stack overflows.

---

## 2. Inode Set Cycle Detection Algorithm

Find-Me maintains a set of visited directory identifiers:

```rust
use std::collections::HashSet;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

pub struct CycleDetector {
    visited_inodes: HashSet<(u64, u64)>, // (dev, ino)
}

impl CycleDetector {
    pub fn new() -> Self {
        Self {
            visited_inodes: HashSet::with_capacity(16384),
        }
    }

    /// Returns true if directory was already visited (cycle detected)
    pub fn visit_or_detect_cycle(&mut self, path: &Path) -> bool {
        // Resolve symlinks to target canonical metadata
        let Ok(meta) = path.metadata() else {
            return false;
        };

        if !meta.is_dir() {
            return false;
        }

        let key = (meta.dev(), meta.ino());
        if !self.visited_inodes.insert(key) {
            // Already present: Cycle detected!
            return true;
        }

        false
    }
}
```

---

## 3. Virtual Filesystems to Skip Automatically

Always bypass these kernel virtual mounts during root scans:
- `/proc` (Linux process and kernel state)
- `/sys` (Kernel device model sysfs)
- `/dev` (Device nodes)
- `/run` (Runtime daemons and volatile sockets)
- `/sys/fs/cgroup` (Control groups)
- `/var/lib/docker` / `/var/lib/containerd` (Container overlay layers)
