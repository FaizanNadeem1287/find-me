# AST & Predicate Evaluator Design

Describes how the parsed query is represented as an Abstract Syntax Tree (AST) and evaluated against in-memory `FileRecord` structs.

---

## 1. Abstract Syntax Tree (AST) Definition

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum QueryToken {
    /// Plain substring or fuzzy search term
    Text(String),
    /// Extension filter: .rs, .toml
    Extension(Vec<String>),
    /// Size comparison: min and/or max bytes
    SizeRange { min: Option<u64>, max: Option<u64> },
    /// Modification date range in epoch seconds
    DateRange { min_secs: u64, max_secs: u64 },
    /// Path prefix restriction
    PathPrefix(String),
    /// Regex matcher compiled safely
    Regex(regex::Regex),
    /// Attribute mask requirements
    Attributes { required: u16, forbidden: u16 },
    /// Inverted negation
    Not(Box<QueryToken>),
}

pub struct QueryAst {
    pub tokens: Vec<QueryToken>,
}
```

---

## 2. High-Performance Evaluation Pipeline

To evaluate 1,000,000 files in under 5 milliseconds:
1. **Pass 1: Fast Integer Predicates (SIMD friendly)**:
   - Filter `size_bytes` against `SizeRange` (simple $u64$ comparisons).
   - Filter `mtime_secs` against `DateRange` (simple $u64$ comparisons).
   - Filter `flags` using bitwise AND (`record.flags & required == required`).
   - *This eliminates 95%+ of candidates without touching any strings!*
2. **Pass 2: Extension & Path Matching**:
   - Resolve extension and directory ID only for remaining candidates.
3. **Pass 3: Name Matching**:
   - Evaluate exact substring via `memchr` or fuzzy match via `nucleo-matcher`.
