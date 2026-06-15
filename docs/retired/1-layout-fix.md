# Specification: Dynamic Ordered DockPanel Layout

This document specifies the technical design and changes required to update `DockPanel` to support true WPF-style ordered docking, eliminating the need to nest dockpanels in app code.

---

## 1. The Problem

The current implementation of `DockPanel::build` hardcodes the rendering hierarchy:
1. **Inner Layer (Horizontal):** Groups `left` + `fill` + `right` in a row.
2. **Outer Layer (Vertical):** Groups `top` + `[Inner Layer]` + `bottom` in a column.

Because this vertical container is always on the outside:
* `top` and `bottom` **always** occupy the full width of the layout.
* `left` and `right` are cut off at the top and bottom edges.

To construct a layout where `left` and `right` span the full height (such as the standard sidebar + content design), developers are currently forced to write nested dockpanel workarounds:
```rust
// Current nested workaround
dock_panel! {
    left: sidebar,
    fill: dock_panel! {
        top: header,
        fill: content,
        bottom: footer
    }
}
```

---

## 2. Proposed Changes

We will refactor `DockPanel` to store docked elements as an **ordered list** of tuples (`Vec<(DockSide, AnyElement)>`). During compilation, the builder will loop through the list in reverse order, dynamically wrapping the center layout from the inside out.

This allows the **call order** in the builder chain (or macro parameters) to dictate layout precedence.

### A. SDK Layout Primitives

#### `[MODIFY]` [layout.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/layout.rs)

1. Introduce a `DockSide` enum.
2. Update the `DockPanel` struct to store docked elements in a vector.
3. Update the builder methods (`.top()`, `.bottom()`, `.left()`, `.right()`) to push to this vector.
4. Update `build()` to nest containers dynamically.

```rust
pub enum DockSide {
    Top,
    Bottom,
    Left,
    Right,
}

pub struct DockPanel<Fill = MissingFill> {
    docked: Vec<(DockSide, AnyElement)>,
    fill: Fill,
}

impl<Fill> DockPanel<Fill> {
    pub fn top(mut self, element: impl IntoElement) -> Self {
        self.docked.push((DockSide::Top, element.into_any_element()));
        self
    }

    pub fn bottom(mut self, element: impl IntoElement) -> Self {
        self.docked.push((DockSide::Bottom, element.into_any_element()));
        self
    }

    pub fn left(mut self, element: impl IntoElement) -> Self {
        self.docked.push((DockSide::Left, element.into_any_element()));
        self
    }

    pub fn right(mut self, element: impl IntoElement) -> Self {
        self.docked.push((DockSide::Right, element.into_any_element()));
        self
    }

    pub fn fill(self, element: impl IntoElement) -> DockPanel<Filled> {
        DockPanel {
            docked: self.docked,
            fill: Filled { element: element.into_any_element() },
        }
    }
}

impl DockPanel<Filled> {
    pub fn build(self) -> Div {
        // Start with the innermost fill content
        let mut root = remainder(self.fill.element);

        // Nest each boundary from the inside out (reversing the addition order)
        for (side, element) in self.docked.into_iter().rev() {
            root = match side {
                DockSide::Top => {
                    div()
                        .size_full()
                        .min_w(px(0.0)).min_h(px(0.0))
                        .flex().flex_col()
                        .child(element)
                        .child(remainder(root.into_any_element()))
                }
                DockSide::Bottom => {
                    div()
                        .size_full()
                        .min_w(px(0.0)).min_h(px(0.0))
                        .flex().flex_col()
                        .child(remainder(root.into_any_element()))
                        .child(element)
                }
                DockSide::Left => {
                    div()
                        .size_full()
                        .min_w(px(0.0)).min_h(px(0.0))
                        .flex()
                        .child(element)
                        .child(remainder(root.into_any_element()))
                }
                DockSide::Right => {
                    div()
                        .size_full()
                        .min_w(px(0.0)).min_h(px(0.0))
                        .flex()
                        .child(remainder(root.into_any_element()))
                        .child(element)
                }
            };
        }

        root
    }
}
```

---

## 3. Example Gallery Refactoring

Once the SDK is updated, we can replace the nested workarounds in our gallery documentation panes with a single, clean `dock_panel!` macro call.

#### `[MODIFY]` [pane.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/dock_panel/pane.rs)

Clean up `DockPanelPane::new()` by declaring the full 5-region dock panel in a single call. Since `left` is declared first, it will span the full height of the outer boundary:

```rust
// Cleaned up, non-nested layout
dock_panel! {
    left: div()
        .w(px(52.0))
        .bg(chrome.panel_background)
        .child("Left"),
    top: div()
        .h(px(44.0))
        .bg(chrome.panel_background)
        .child("Top"),
    right: div()
        .w(px(68.0))
        .bg(chrome.panel_background)
        .child("Right"),
    bottom: div()
        .h(px(44.0))
        .bg(chrome.panel_background)
        .child("Bottom"),
    fill: div()
        .size_full()
        .bg(gpui::red())
        .child("Center")
}
```

---

## 4. Verification Plan

### Automated Tests
* Run unit tests inside `crates/sdk/src/layout.rs` to verify that order-based compilation functions correctly under type checking.
  ```bash
  cargo test -p gpui-luma --lib layout::tests
  ```

### Manual Verification
* Deploy `apps/gallery` and open the **DockPanel** gallery page.
* Confirm that the left/right bars span the full height of the panel edges, matching the visual WPF layout design.
