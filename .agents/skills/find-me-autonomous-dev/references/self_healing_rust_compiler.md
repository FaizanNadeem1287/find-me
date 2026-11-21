# Self-Healing Rust Compiler & Diagnostics Guide

This reference provides deterministic, algorithmic resolution patterns for compiler, borrow-checker, and linter errors encountered during autonomous Rust development.

---

## 1. Borrow Checker Diagnostics

### Error E0382: Use of Moved Value
- **Cause**: A variable that does not implement `Copy` was moved into a closure, function, or loop, and then referenced again.
- **Resolution**:
  - If lightweight/reference counted: Wrap in `std::sync::Arc` or clone prior to closure entry:
    ```rust
    let shared_state = Arc::clone(&state);
    tokio::spawn(async move {
        shared_state.do_something();
    });
    ```
  - If inside a loop: Borrow by reference (`&item` or `&mut item`) rather than consuming by value.

### Error E0502: Cannot Borrow as Mutable because it is also Borrowed as Immutable
- **Cause**: An active immutable reference exists in scope when a mutable borrow is attempted.
- **Resolution**:
  - Scope the immutable borrow with curly braces `{ ... }` so it drops before the mutable borrow begins:
    ```rust
    // Fixed: Scoped read drop
    let count = {
        let guard = arena.read();
        guard.len()
    };
    arena.write().insert(new_record);
    ```
  - Avoid calling methods that mutate `self` while iterating over `&self.items`. Collect indices (`Vec<usize>`) first, then mutate by index.

### Error E0597: Value Does Not Live Long Enough
- **Cause**: A reference points to temporary stack data that is dropped when the scope exits.
- **Resolution**:
  - Store owned data (`String`, `PathBuf`, or integer IDs) rather than references in long-lived structs.
  - In Find-Me: Use `u32` pool offsets into string arenas rather than `&str` references with complex lifetime annotations.

---

## 2. Concurrency & Async Diagnostics (Tokio & Iced)

### Error: `*mut () cannot be shared between threads safely` (Trait `Send` / `Sync` Not Satisfied)
- **Cause**: A type that is not thread-safe (e.g. `Rc<T>`, `RefCell<T>`, or raw pointers) is passed across a thread boundary or held across a `.await` point.
- **Resolution**:
  - Replace `Rc<T>` with `std::sync::Arc<T>`.
  - Replace `RefCell<T>` with `parking_lot::Mutex<T>` or `parking_lot::RwLock<T>`.
  - Ensure any lock guards held across `.await` points use `tokio::sync::Mutex` instead of standard sync locks, or drop sync locks before `.await`.

### Error: Blocking I/O inside Tokio Async Task
- **Cause**: Running synchronous filesystem reads (`std::fs`, `jwalk`) directly inside an async Tokio worker thread starves the async reactor.
- **Resolution**:
  - Offload blocking operations using `tokio::task::spawn_blocking`:
    ```rust
    let results = tokio::task::spawn_blocking(move || {
        crawler.crawl_sync(&path)
    }).await.map_err(|e| EngineError::TaskJoin(e.to_string()))??;
    ```
  - For parallel CPU work: Use Rayon's thread pool (`rayon::spawn` or `rayon::iter`).

---

## 3. Iced UI Elm Diagnostics

### Error: Mismatched Message Type in `view()`
- **Cause**: A widget dispatches a message of type `ChildMessage`, but `view()` returns `Element<ParentMessage>`.
- **Resolution**:
  - Map child messages to parent messages using `.map(ParentMessage::Child)`:
    ```rust
    search_bar::view(&self.search_state).map(Message::SearchBar)
    ```

### Error: Cannot Return Non-Static Lifetimes in Iced Tasks
- **Cause**: An asynchronous `Task` returned by `update()` captures a reference with a non-`'static` lifetime.
- **Resolution**:
  - Clone or move owned data into the task closure: `Task::perform(fetch_data(query.clone()), Message::DataFetched)`.
