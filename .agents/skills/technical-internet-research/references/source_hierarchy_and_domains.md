# Source Hierarchy & Domain Targeting Reference

Detailed breakdown of top technical resources and how to query them effectively.

---

## 1. Stack Overflow Search Strategies

Stack Overflow contains millions of verified answers to edge cases.
- **Query Structure**:
  `site:stackoverflow.com/questions [rust] <exact_problem>`
- **Examples**:
  - `site:stackoverflow.com/questions [rust] [iced] change theme dynamically`
  - `site:stackoverflow.com/questions [rust] [inotify] handle queue overflow safely`
  - `site:stackoverflow.com/questions [rust] compare epoch seconds date range`
- **Verification Rule**: Always verify that the answer is marked as accepted or has high upvotes, and check the answer date (prefer answers from 2022+ for modern Rust idioms).

---

## 2. Reddit (`/r/rust`, `/r/linuxdev`) Search Strategies

Reddit often contains deep architecture discussions, real-world benchmarks, and post-mortems of performance bottlenecks.
- **Query Structure**:
  `site:reddit.com/r/rust "<topic>" OR "<library>"`
- **Examples**:
  - `site:reddit.com/r/rust "nucleo-matcher" vs "fuzzy-matcher" benchmark`
  - `site:reddit.com/r/rust "iced" virtual list 100000 items`
  - `site:reddit.com/r/rust fastest directory traversal "jwalk" vs "ignore"`
- **Signal Extraction**: Look for comments from crate maintainers (e.g. `hecrj` for Iced, `BurntSushi` for regex/walkdir).

---

## 3. GitHub Discussions, Issues & PRs

The most accurate and up-to-date documentation is often in merged PR descriptions and issue threads:
- **Query Structure**:
  `site:github.com/<owner>/<repo>/issues "<keyword>"`
- **Examples**:
  - `site:github.com/iced-rs/iced/issues "table" virtualized`
  - `site:github.com/notify-rs/notify/issues "IN_Q_OVERFLOW" recovery`
  - `site:github.com/helix-editor/helix/issues "nucleo" score`

---

## 4. Linux Kernel Man Pages & POSIX Standards

For systems programming, nothing supersedes the Linux man pages:
- **Query Structure**:
  `site:man7.org/linux/man-pages "<syscall>"`
- **Examples**:
  - `site:man7.org/linux/man-pages "getdents64"`
  - `site:man7.org/linux/man-pages "inotify_add_watch"`
  - `site:man7.org/linux/man-pages "stat"`
