# Conventional Commit Standards for Find-Me

This guide details the commit message format and atomic commit rules used across the Find-Me codebase.

---

## 1. Commit Structure

Every commit message must follow the Conventional Commits specification:

```
<type>(<scope>): <short description in present tense>

[optional body explaining WHY the change was made]

[optional footer referencing Todoist task or issue]
```

### Types:
- **`feat`**: A new feature, struct, screen, or capability (e.g. `feat(p1): implement 32-byte FileRecord packing`).
- **`fix`**: A bug fix or error recovery mechanism (e.g. `fix(watcher): handle IN_Q_OVERFLOW gracefully`).
- **`test`**: Adding unit tests, property tests, or benchmarks (e.g. `test(query): add proptest fuzzing for size filters`).
- **`perf`**: A code change that improves performance without altering functionality (e.g. `perf(matcher): use SIMD memchr for exact substring search`).
- **`docs`**: Documentation, skill updates, or roadmap updates (e.g. `docs(git): document branch retention rules`).
- **`refactor`**: Code restructuring without adding features or fixing bugs (e.g. `refactor(ui): extract sidebar into reusable component`).
- **`chore`**: Tooling updates, Cargo dependencies, or formatting (e.g. `chore: update iced to 0.14`).

---

## 2. Commit Hygiene Rules

1. **Atomic Commits**: Each commit should represent a single logical unit of work. Do not mix unrelated refactors with new features.
2. **Imperative Mood**: Use "add", "implement", "fix", not "added", "implementing", "fixed".
3. **No Dead or Debug Code**: Never commit commented-out code, temporary `dbg!()` statements, or local scratch files.
4. **Zero Compilation Breakage**: Every commit on a feature branch must compile cleanly. Never commit broken code that prevents `cargo check` from running.
