# Search Syntax & Query Language Reference

Find-Me supports an intuitive query grammar inspired by Voidtools *Everything*, enhanced with modern semantic prefixes, boolean operators, wildcards, and date math.

---

## 1. Search Modes

Find-Me features 4 distinct search modes, toggleable via UI pills or operator prefixes:

| Mode | Trigger Prefix | Description |
| :--- | :--- | :--- |
| **`[Everything]`** | (Default) | Fast filename, extension, path, and metadata search. |
| **`[Content]`** | `content:<query>` or `text:<query>` | Full-text lexical search inside document text using BM25. |
| **`[Semantic]`** | `semantic:<query>` or `sem:<query>` | Neural vector search by concept, meaning, and intent. |
| **`[Hybrid]`** | `hybrid:<query>` | Reciprocal Rank Fusion blending BM25 keywords + Dense Vectors. |

---

## 2. Basic Search & Wildcards

* **Exact Match**: Words separated by spaces act as an implicit **AND**:
  ```text
  realtor proposal pdf
  ```
* **Wildcards**:
  - `*`: Matches zero or more characters (`*.rs`, `report_*_2026.docx`).
  - `?`: Matches exactly one character (`file_??.txt`).
* **Exact Phrases**: Enclose multi-word terms in double quotes:
  ```text
  "AI Powered Realtor Application"
  ```
* **Boolean OR**: Separate terms with a pipe symbol `|`:
  ```text
  report | invoice | quotation
  ```
* **Boolean NOT**: Invert a match by prefixing with an exclamation mark `!`:
  ```text
  *.log !error !debug
  ```

---

## 3. Extension & Path Filters

| Operator | Syntax | Example | Description |
| :--- | :--- | :--- | :--- |
| `ext:` | `ext:<extension>` | `ext:pdf;docx` | Matches files with specific extensions (semicolon acts as OR). |
| `path:` | `path:<substring>` | `path:Downloads/Projects` | Matches only files residing under paths containing the substring. |
| `parent:` | `parent:<dir>` | `parent:src` | Matches files whose immediate parent directory matches `dir`. |
| `root:` | `root:<path>` | `root:/home/user` | Constrains query to a specific filesystem mount or root. |

---

## 4. Size Filters & Comparators

Find-Me supports exact sizes, comparisons (`>`, `<`, `>=`, `<=`), ranges (`..`), and unit modifiers (`B`, `KB`, `MB`, `GB`, `TB`):

```text
size:>50MB
size:<10KB
size:10MB..100MB
size:>=1GB
```

### Predefined Size Aliases
* `size:empty` (0 bytes)
* `size:tiny` (0 KB – 16 KB)
* `size:small` (16 KB – 1 MB)
* `size:medium` (1 MB – 100 MB)
* `size:large` (100 MB – 1 GB)
* `size:huge` (> 1 GB)

---

## 5. Date & Time Filters (`dm:`, `dc:`, `da:`)

Filter files by **Date Modified** (`dm:`), **Date Created** (`dc:`), or **Date Accessed** (`da:`):

### Relative Date Aliases
* `dm:today` (Modified since midnight today)
* `dm:yesterday` (Modified yesterday)
* `dm:thisweek` (Modified within current calendar week)
* `dm:lastweek` (Modified within past 7 days)
* `dm:thismonth` (Modified within current month)
* `dm:lastmonth` (Modified within past 30 days)
* `dm:thisyear` (Modified within current year)
* `dm:lastyear` (Modified within past 365 days)

### ISO Date Formats & Ranges
* Specific date: `dm:2026-11-22`
* Date range: `dm:2026-01-01..2026-06-30`
* Comparisons: `dm:>2026-10-01`, `dm:<=2026-11-15`

---

## 6. POSIX File Attributes (`attrib:`)

Filter files by POSIX flags and file types:
* `attrib:d` or `folder:`: Match directories only.
* `attrib:f` or `file:`: Match regular files only.
* `attrib:h`: Match hidden files (files beginning with `.`).
* `attrib:s`: Match symlinks.
* `attrib:r`: Match read-only files.

---

## 7. Regular Expressions (`regex:`)

Find-Me supports full Rust regex syntax by prefixing with `regex:`:
```text
regex:^IMG_[0-9]{4}\.(jpg|png)$
regex:test_.*_v[0-9]+\.rs
```

---

## 8. Combining Multiple Filters

All operators and filters compose together cleanly:
```text
ext:pdf "realtor" size:>50KB dm:lastmonth path:Downloads
```
*(Finds PDF documents containing "realtor", larger than 50 KB, modified in the last month, inside the Downloads folder).*
