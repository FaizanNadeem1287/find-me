# Project Current State & Active Task Snapshot

*Last Updated*: 2026-11-21 23:51 PKT

---

## 1. Active Task & Branch Status

Attribute | Current Value | Notes / References
:--- | :--- | :---
**Active Phase** | **Phase 2: Core Engine - Query Parser, Nucleo-Matcher & Filtering** | Documented in `PHASES.md`
**Active Task ID** | **[TASK-2.1] Query: Everything Search Syntax Tokenizer & AST Parser** (`6j4626PQcQCQCh6x`) | Moved to `In Progress`
**Active Git Branch** | `feature/p2-query-parser` | Branched from `main`; will be retained permanently
**Todoist Card** | `[TASK-2.1] Query: Everything Search Syntax Tokenizer & AST Parser` | Section: **In Progress** (`6j45x632fhrVMmvQ`)
**Todoist Project** | [Everything](https://app.todoist.com/app/project/everything-6j45wf6xPVCM2RJx) (`id: 6j45wf6xPVCM2RJx`) | 5 active sections, 23 total tasks

---

## 2. Active Implementation Objectives in Task 2.1

- [ ] Implement AST nodes in `crates/findme-core/src/query/ast.rs` (`QueryNode`, `SearchPredicate`, `QueryAst`).
- [ ] Implement unit and date conversions in `crates/findme-core/src/query/units.rs` (size binary units, ISO/relative dates).
- [ ] Implement zero-allocation lexer in `crates/findme-core/src/query/lexer.rs`.
- [ ] Implement fault-tolerant Pratt / recursive-descent parser in `crates/findme-core/src/query/parser.rs`.
- [ ] Export query API in `crates/findme-core/src/query/mod.rs` and `crates/findme-core/src/lib.rs`.
- [ ] Implement 50+ query variations unit test suite.
- [ ] Implement proptest property-based fuzz tests verifying 0 panics on arbitrary string queries.
- [ ] Benchmark query parsing latency confirming $<50\,\mu\text{s}$.
- [ ] Run pre-merge quality gate `verify_task.sh` (formatting, clippy -D warnings, tests).
- [ ] Merge into `main` with `--no-ff`, retain branch permanently, and post Todoist audit comment.

---

## 3. Immediate Next Steps

- Implement `crates/findme-core/src/query/` submodules.
