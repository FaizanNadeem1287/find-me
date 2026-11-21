# Query Formulation & Operators Guide

Mastery of search engine operators allows finding exact technical solutions in seconds without wading through blog spam.

---

## 1. Core Search Operators

Operator | Syntax | Effect | Example
:--- | :--- | :--- | :---
**Exact Match** | `"..."` | Forces exact character sequence | `"iced::widget::table"`
**Domain Scope** | `site:...` | Restricts results to specific domain | `site:docs.rs/iced`
**Disjunction** | `OR` / `\|` | Matches either term | `"jwalk" OR "rayon" traversal`
**Exclusion** | `-term` | Excludes irrelevant or noisy results | `"rust" GUI -electron -tauri`
**File Type** | `filetype:pdf` | Finds whitepapers or RFC specs | `filetype:pdf "NTFS" "MFT"`

---

## 2. Advanced Search Query Formulas

### Scenario A: Debugging an Obscure Compiler Error
```text
"error[E0382]" "borrow of moved value" "iced" site:stackoverflow.com OR site:reddit.com/r/rust
```

### Scenario B: Finding Real-World Usage of an API
```text
site:github.com "iced_test::simulator" "Simulator"
```

### Scenario C: Verifying Linux System Call Behavior
```text
site:man7.org "inotify" "IN_Q_OVERFLOW" "buffer"
```

### Scenario D: Comparing Crate Performance
```text
site:reddit.com/r/rust "rkyv" vs "bincode" benchmark speed
```
