---
name: query-syntax-and-parser
description: >-
  Complete specification, grammar, tokenization, and AST evaluation reference for the Find-Me
  search query engine. Covers Everything-style query syntax (ext:, size:, dm:, path:, regex:,
  boolean operators, and wildcards), date math, size unit conversion, and safe predicate evaluation.
---

# Search Query Syntax & Parser Engineering Skill

This skill defines the complete grammar, tokenization logic, AST structure, and evaluation engine for search queries in **Find-Me**, fully matching the power-user capabilities of Windows *Everything*.

---

## 1. Supported Query Syntax Overview

Category | Syntax Pattern | Example | Meaning
:--- | :--- | :--- | :---
**Exact Substring** | `term` | `main.rs` | Case-insensitive substring match
**Fuzzy Search** | `term` | `cfg_jsn` | Smith-Waterman dual matrix fuzzy scoring (`nucleo`)
**File Extension** | `ext:<ext1;ext2>` | `ext:rs;toml` | Matches files with `.rs` or `.toml` extensions
**Size Comparison** | `size:<op><val><unit>` | `size:>10mb` | Matches files strictly larger than 10 Megabytes
**Size Range** | `size:<min>..<max>` | `size:100kb..5mb` | Matches files between 100 KB and 5 MB
**Date Modified** | `dm:<keyword>` | `dm:today` | Matches files modified since midnight today
**Date Range** | `dm:<date1>..<date2>` | `dm:2026-11-01..2026-11-21` | Files modified within specified calendar range
**Path Scoping** | `path:<prefix>` | `path:/home/user/code` | Files residing inside `/home/user/code`
**Attribute Filter** | `attrib:<flags>` | `attrib:d` | `d` (directory), `h` (hidden), `x` (executable)
**Exclusion** | `!<term>` | `!node_modules` | Excludes any file or path containing `node_modules`
**Regular Expression** | `regex:<pattern>` | `regex:^test_.*\.json$` | Matches regex against file name

---

## 2. Detailed Reference Guides

- [Complete EBNF Query Grammar](./references/query_grammar_ebnf.md): Formal grammatical specification for tokenizer and parser.
- [Size & Date Parsers](./references/size_and_date_parsers.md): Unit conversions (B, KB, MB, GB) and human date math (`today`, `yesterday`, `lastweek`).
- [AST & Predicate Evaluator](./references/ast_and_predicate_evaluator.md): Compiling parsed query tokens into high-speed closure predicates.
