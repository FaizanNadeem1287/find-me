# Troubleshooting & Frequently Asked Questions (FAQ)

This guide covers common diagnostic procedures, kernel configuration, and troubleshooting steps for Find-Me.

---

## 1. Frequently Asked Questions (FAQ)

### Q: How is Find-Me different from `fzf`, `locate`, or `plocate`?
* **`locate` / `plocate`**: Relies on a cron-based database (`updatedb`) that is stale until updated. Does not offer real-time inotify monitoring, interactive GUI tables, or semantic content searching.
* **`fzf`**: A fuzzy finder for the terminal, but requires streaming directory listings (e.g. from `find` or `fd`) on every search, which can take several seconds for large drives.
* **Find-Me**: Maintains an in-memory 32-byte arena with sub-millisecond query evaluation, real-time Linux inotify sync, virtualized 60 FPS graphical UI, and local offline semantic AI.

### Q: Does Find-Me send any data to external servers or cloud APIs?
**No.** Find-Me is 100% offline-first. All vector embeddings, HNSW graphs, content extraction, and searches run locally on your CPU/memory. Zero network telemetry, zero cloud calls.

### Q: How much RAM does Find-Me consume?
Find-Me uses a compact 32-byte record layout with string pooling.
* ~50,000 files: **~4 MB** RAM
* ~200,000 files: **~15.8 MB** RAM
* ~1,000,000 files: **~75 MB** RAM

---

## 2. Linux Inotify Watches Limit

### Symptom
When indexing large directory trees (e.g. hundreds of thousands of folders), you may see an inotify watch warning or system notice.

### Diagnosis
Check your current system inotify watch limit:
```bash
cat /proc/sys/fs/inotify/max_user_watches
```
Default is often 8,192 or 65,536 on some distributions.

### Solution
Increase the limit to 524,288 watches:
```bash
echo "fs.inotify.max_user_watches=524288" | sudo tee -a /etc/sysctl.d/99-inotify.conf
sudo sysctl -p /etc/sysctl.d/99-inotify.conf
```

---

## 3. Downloads / Monitored Folder Not Appearing

### Diagnosis & Fix
1. Navigate to **Index Management** (`Ctrl + 3`).
2. Verify that your desired folder root (e.g. `/home/user/Downloads`) is listed in **Monitored Directory Roots**.
3. Check the **Exclusion Rules (.gitignore Style)** section to confirm the folder is not excluded by a rule (e.g. `*.tmp`, `node_modules/`, or a custom rule).
4. Click **"Rebuild Index"** or run:
   ```bash
   find-me --rebuild
   ```

---

## 4. Rebuilding Semantic Vector Index

### Symptom
You added new documents, PDFs, or audio recordings and want them immediately indexed for semantic meaning.

### Procedure
1. Navigate to **Index Management** (`Ctrl + 3`).
2. In the **"Local Semantic Vector Search & Content Indexing"** card, click **"Index Content Now"**.
3. The card displays live progress:
   ```text
   ⚡ Indexing: 14 / 37 files (38%) • report.pdf
   ```
4. When finished, a completion banner appears at the top:
   ```text
   ⓘ Semantic vector indexing complete: 37 files indexed (124 vector chunks)
   ```

---

## 5. Cleaning Disk Cache & Memory Reset

If you ever need to reset Find-Me's cache completely:
```bash
# Clear binary index and semantic embeddings
rm -rf ~/.cache/find-me

# Re-launch Find-Me to perform a fresh scan
find-me
```
