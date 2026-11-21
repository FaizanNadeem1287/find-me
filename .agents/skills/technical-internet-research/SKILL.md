---
name: technical-internet-research
description: >-
  Operational methodology and search playbook for deep technical internet research in Find-Me.
  Guides the agent in formulating high-precision search queries, mining Stack Overflow, Reddit (r/rust),
  GitHub discussions/issues, Linux man pages, and official documentation to find production-grade,
  zero-workaround solutions.
---

# Technical Internet Research & Intelligence Skill

This skill defines the methodology, query formulation patterns, source verification hierarchies, and evaluation filters for conducting deep technical research across the internet. It ensures that solutions gathered from external resources are **proper, production-grade, and free of brittle workarounds**.

---

## 1. Information Hierarchy & Source Authority

When researching solutions, prioritize sources by engineering authority:

```
[ Tier 1: Official & Authoritative ]
  • docs.rs (Official crate documentation)
  • GitHub Issues / PRs / Discussions of dependencies (iced-rs, tokio, notify)
  • Linux Kernel Man Pages (man7.org, kernel.org)
  • Rust Reference & Clippy Documentation
               │
               ▼
[ Tier 2: Practitioner Communities (In-Depth Technical Discussion) ]
  • Stack Overflow (Accepted answers, high-reputation explanations)
  • Reddit (/r/rust, /r/linuxdev, /r/systems)
  • This Week in Rust & Community RFCs
               │
               ▼
[ Tier 3: Edge-Case & Contextual Resources ]
  • Technical Engineering Blogs (matklad, fasterthanlime, BurntSushi)
  • GitHub Code Search (real-world implementations in active open-source projects)
  • Quora / General Developer Forums
```

---

## 2. Query Formulation Principles

### Rule 1: Always Pin Versions
Ecosystems evolve rapidly. Outdated Iced or Tokio examples cause compile errors:
- Bad: `iced table virtual list` (Returns obsolete 2021 Iced 0.4 tutorials)
- Good: `"iced 0.14" OR "iced 0.13" "virtual" OR "table"`

### Rule 2: Quote Exact Compiler Error Codes
Search with the compiler's exact diagnostic token:
- Bad: `rust borrow error mutable and immutable`
- Good: `"E0502" "cannot borrow" "as mutable" "iced"`

### Rule 3: Target Specific High-Signal Domains
Use Google search domain operators to bypass spam and AI-generated aggregators:
- Stack Overflow: `site:stackoverflow.com/questions "rust" "notify" inotify overflow`
- Reddit: `site:reddit.com/r/rust "iced" virtual list performance`
- GitHub Issues: `site:github.com/iced-rs/iced/issues virtual scroll table`
- Linux Man Pages: `site:man7.org inotify IN_Q_OVERFLOW`

---

## 3. The Zero-Workaround Solution Filter

Before adopting any code pattern found on Stack Overflow or Reddit, run it through the **Zero-Workaround Filter**:

Evaluation Check | Reject (Workaround) | Accept (Proper Solution)
:--- | :--- | :--- |
**Safety** | Wrapping code in `unsafe { ... }` to silence borrow checker | Restructuring data ownership, using arenas, or scoping borrows
**Error Handling** | Calling `.unwrap()` or `.expect()` | Mapping errors to `thiserror` typed enums with `?` propagation
**UI Thread** | Calling blocking `std::fs` inside async loops | Using `tokio::task::spawn_blocking` or Rayon pools
**Permissions** | Demanding `sudo` or root privileges | Operating 100% within standard user POSIX permissions
**Kernel Polling** | `loop { sleep(100ms); check(); }` | Subscribing to kernel `inotify` event streams

---

## Detailed Reference Guides

- [Source Hierarchy & Domain Targeting Guide](./references/source_hierarchy_and_domains.md): Best practices for searching Stack Overflow, Reddit, and GitHub.
- [Query Formulation & Operators Guide](./references/query_formulation_operators.md): Precision Google search syntax, quotes, exclusions, and filters.
- [Evaluating Proper Fixes vs. Workarounds](./references/evaluating_proper_fixes_vs_workarounds.md): Framework for vetting external code before adoption.
- [Rust Ecosystem Research Cheatsheet](./references/rust_ecosystem_research_cheatsheet.md): Ready-to-use search templates for Iced, Tokio, POSIX, and systems programming.
