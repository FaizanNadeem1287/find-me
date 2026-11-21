# Rust Ecosystem Research Cheatsheet

Curated search blueprints for the core technology domains of Find-Me.

---

## 1. Iced GUI Framework

Target | Recommended Search Blueprint
:--- | :---
**Virtual Table / List** | `site:github.com/iced-rs/iced/issues "table" OR "VirtualList"`
**Dynamic Theme Switch** | `site:docs.rs/iced "Theme" "palette" custom theme`
**Keyboard Shortcuts** | `site:docs.rs/iced "keyboard::Event" "Key::Named"`
**Async Subscriptions** | `site:docs.rs/iced "subscription::run" tokio channel`

---

## 2. Filesystem & Systems Programming

Target | Recommended Search Blueprint
:--- | :---
**Parallel Traversal** | `site:docs.rs/jwalk "WalkDir" rayon parallel`
**Inotify Event Streams** | `site:docs.rs/notify "recommended_watcher" debounced`
**POSIX File Inodes** | `site:docs.rs/nix "stat" "ino" "dev"`
**DBus File Manager** | `site:specifications.freedesktop.org "org.freedesktop.FileManager1"`

---

## 3. Query Parsing & String Matching

Target | Recommended Search Blueprint
:--- | :---
**Nucleo Matcher** | `site:docs.rs/nucleo-matcher "Pattern" "match_list"`
**Regex Safety** | `site:docs.rs/regex "RegexBuilder" size_limit`
**SIMD Substring** | `site:docs.rs/memchr "memmem" Find`
