# Command-Line Interface (CLI) & Headless Automation

Find-Me features a high-performance command-line interface (`findme-cli`) for headless shell scripting, terminal querying, JSON data pipelines, batch exports, and GUI integration.

---

## 1. Basic CLI Syntax

```bash
find-me [OPTIONS] [QUERY]
```

If run without arguments, Find-Me launches the graphical user interface (GUI).

---

## 2. Headless Search Queries

Execute fast file searches directly in your terminal without opening a graphical window using `--headless`:

```bash
# Search for Rust source files
find-me --headless "ext:rs fn main"

# Constrain result count
find-me --headless "realtor *.pdf" --limit 10

# Search inside a specific directory path
find-me --headless "path:Downloads size:>50MB"
```

---

## 3. Machine-Readable JSON Output (`--json`)

Pipe structured search results into tools like `jq`, shell scripts, or backend services:

```bash
find-me --headless --json "ext:py" --limit 5 | jq .
```

### JSON Output Structure
```json
{
  "query": "ext:py",
  "total_matches": 124,
  "duration_ms": 1,
  "results": [
    {
      "id": 1042,
      "name": "trainer.py",
      "path": "/home/user/Projects/ai/trainer.py",
      "size_bytes": 14250,
      "mtime_secs": 1700000000,
      "is_dir": false
    }
  ]
}
```

---

## 4. Multi-Format Batch Exporters

Export search results directly to files in various standard formats:

| Format Flag | Output Type | Description |
| :--- | :--- | :--- |
| `--export-csv <file>` | **CSV** | Comma-Separated Values containing `Name`, `Path`, `Size`, `Modified`. |
| `--export-txt <file>` | **Plain Text** | Clean newline-delimited list of absolute paths. |
| `--export-efu <file>` | **EFU File List** | Everything File List (`Filename,Size,Date Modified,Date Created,Attributes`). |
| `--export-json <file>`| **JSON** | Full structured JSON array of matching records. |

### Example
```bash
# Export all documents larger than 10MB to CSV
find-me --headless "ext:pdf;docx size:>10MB" --export-csv large_docs.csv
```

---

## 5. GUI Integration & Query Pre-filling

Launch the graphical window with an active query already filled in:

```bash
find-me --gui --query "Day12 *.pdf"
```

This is ideal for binding custom desktop shortcuts (e.g. `Super + Space` or terminal aliases) to launch Find-Me with pre-selected filters.

---

## 6. Index Inspection & Management Flags

Inspect memory usage and control the persistent index from the terminal:

```bash
# Print active cache metrics and arena statistics
find-me --stats

# Force a background index rebuild
find-me --rebuild

# Clear the persistent binary index
find-me --clear
```
