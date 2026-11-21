# Headless Desktop UI Testing Guide (Iced)

This guide documents how to thoroughly test Find-Me's pure Rust desktop user interface without needing a graphical display, GPU, or X11/Wayland session.

---

## 1. Why Headless UI Testing Works in Iced

In traditional GUI frameworks (like Electron, Qt, or GTK), testing UI often requires starting heavy virtual display servers (like `Xvfb`) or brittle Selenium/WebDriver scripts that break on font or pixel variations.

Because **Iced** is built on the strict Elm Architecture (Model-View-Update), the UI is fundamentally a deterministic state machine:
1. **`State`**: The pure data describing the screen (current search query, filtered results list, active screen, theme).
2. **`Message`**: Explicit events dispatched by user actions (e.g. `SearchInputChanged("test")`, `SortColumn(ColumnId::Size)`, `SelectRow(3)`).
3. **`update()`**: A pure transition function: `fn update(&mut self, message: Message) -> Task<Message>`.
4. **`view()`**: A declarative description of what widgets should be rendered given the current state.

Because of this separation, **UI business logic, keyboard navigation, filtering, and screen routing can be verified 100% headlessly in standard Rust tests.**

---

## 2. Using `iced_test` Simulator

The official `iced_test` crate provides a `Simulator` that evaluates widget trees and simulates user interactions headlessly:

```toml
# In findme-ui/Cargo.toml
[dev-dependencies]
iced_test = "0.14"
```

### Pattern 1: Testing State Machine Transitions
Directly test how user messages update the application state:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use findme_core::SearchResultItem;

    #[test]
    fn test_search_input_updates_state() {
        let mut app = FindMeApp::default();
        assert_eq!(app.search_query, "");

        // Dispatch message simulating user typing
        let _ = app.update(Message::SearchInputChanged("project_alpha".to_string()));

        assert_eq!(app.search_query, "project_alpha");
        assert!(app.is_searching);
    }

    #[test]
    fn test_escape_clears_search() {
        let mut app = FindMeApp::default();
        app.search_query = "something".to_string();

        let _ = app.update(Message::KeyPressed(iced::keyboard::Key::Named(
            iced::keyboard::key::Named::Escape,
        )));

        assert_eq!(app.search_query, "");
    }
}
```

### Pattern 2: Simulating Widget Clicks & Text Entry with `Simulator`
Test that buttons, input fields, and navigation links trigger the correct messages:

```rust
#[cfg(test)]
mod tests {
    use iced_test::simulator;
    use super::*;

    #[test]
    fn test_sidebar_navigation_switch() {
        let mut app = FindMeApp::default();
        assert_eq!(app.current_screen, Screen::Search);

        // Wrap the view in a headless Simulator
        let mut ui = simulator(app.view());

        // Click the 'Index Management' navigation item in sidebar
        let _ = ui.click("Index Management");

        // Process generated messages
        for msg in ui.into_messages() {
            let _ = app.update(msg);
        }

        // Verify screen state updated
        assert_eq!(app.current_screen, Screen::IndexManagement);
    }
}
```

### Pattern 3: Testing Results Sorting & Selection
Test that clicking column headers or pressing arrow keys correctly sorts and selects rows:

```rust
#[test]
fn test_column_sort_by_size_descending() {
    let mut app = FindMeApp::default();
    app.results = vec![
        mock_result("file_small.txt", 1024),
        mock_result("file_huge.iso", 104857600),
        mock_result("file_medium.bin", 524288),
    ];

    // Click 'Size' column header
    let _ = app.update(Message::SortResults(ColumnId::Size));

    assert_eq!(app.results[0].name, "file_huge.iso");
    assert_eq!(app.results[1].name, "file_medium.bin");
    assert_eq!(app.results[2].name, "file_small.txt");
}
```

---

## 3. Visual Regression Testing with Software Renderer

To verify that layout geometries and styling remain intact without a GPU:
- Use `iced_tiny_skia` (software renderer).
- Snapshot the widget layout into an in-memory pixmap.
- Assert that dimensions, padding, and text alignments conform to baseline specs.
