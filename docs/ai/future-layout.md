# Future Layout Concepts (WPF-Style Layout Panels)

This document preserves the designs and macro concepts for WPF-style layout containers mapped directly to GPUI's flex and grid layouts.

---

## 1. `stack_panel!` Macro

Arranges child elements into a single line, either horizontally or vertically.

### Macro Definition
```rust
macro_rules! stack_panel {
    (
        vertical $(gap=$gap:expr)? $(items=$align:ident)?;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = gpui::div().flex().flex_col();
        $( panel = panel.gap(gpui::px($gap as f32)); )?
        $( panel = stack_panel!(@align_items panel, $align); )?
        $( panel = panel.child($child); )*
        panel
    }};

    (
        horizontal $(gap=$gap:expr)? $(items=$align:ident)?;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = gpui::div().flex();
        $( panel = panel.gap(gpui::px($gap as f32)); )?
        $( panel = stack_panel!(@align_items panel, $align); )?
        $( panel = panel.child($child); )*
        panel
    }};

    (@align_items $panel:ident, center) => { $panel.items_center() };
    (@align_items $panel:ident, start) => { $panel.items_start() };
    (@align_items $panel:ident, end) => { $panel.items_end() };
}
```

### Example Usage
```rust
stack_panel! {
    vertical gap=12;
    card_title("Settings", "Manage preferences", title_color, body_color),
    self.username_input.clone(),
    stack_panel! {
        horizontal gap=8 items=center;
        self.submit_btn.clone(),
        self.cancel_btn.clone(),
    }
}
```

---

## 2. `wrap_panel!` Macro

Positions child elements sequentially from left to right or top to bottom. When elements reach the edge of the panel, they wrap automatically to the next line.

### Macro Definition
```rust
macro_rules! wrap_panel {
    (
        $(gap=$gap:expr)? $(items=$align:ident)?;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = gpui::div().flex().flex_wrap();
        $( panel = panel.gap(gpui::px($gap as f32)); )?
        $( panel = wrap_panel!(@align_items panel, $align); )?
        $( panel = panel.child($child); )*
        panel
    }};

    (@align_items $panel:ident, center) => { $panel.items_center() };
    (@align_items $panel:ident, start) => { $panel.items_start() };
    (@align_items $panel:ident, end) => { $panel.items_end() };
}
```

### Example Usage
```rust
// Useful for tags, filter lists, or variable-size icon lists
wrap_panel! {
    gap=6 items=center;
    tag_element("rust"),
    tag_element("gpui"),
    tag_element("declarative-ui"),
    tag_element("wpf"),
    tag_element("layout-panels"),
}
```

---

## 3. `grid_panel!` Macro

Arranges child elements in a flexible, multi-column and multi-row layout grid utilizing GPUI's built-in grid layout system.

### Macro Definition
```rust
macro_rules! grid_panel {
    (
        cols=$cols:expr $(rows=$rows:expr)? $(gap=$gap:expr)?;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = gpui::div().grid().grid_cols($cols);
        $( panel = panel.grid_rows($rows); )?
        $( panel = panel.gap(gpui::px($gap as f32)); )?
        $( panel = panel.child($child); )*
        panel
    }};
}
```

### Example Usage
```rust
// Renders elements in a 3-column grid structure
grid_panel! {
    cols=3 gap=16;
    grid_card("Card 1"),
    grid_card("Card 2"),
    grid_card("Card 3"),
    grid_card("Card 4"),
    grid_card("Card 5"),
    grid_card("Card 6"),
}
```

---

## 4. How to Integrate into the SDK for Reusability

To distribute these layout panels as a reusable part of the `gpui-luma` SDK, they must follow proper namespacing and export rules.

### 4.1 Crate-Safe Pathing
Inside `#[macro_export]` macros, any external dependencies (like `gpui`) must use absolute paths to compile correctly in client crates. The internal helper rule matches must also reference `$crate` to resolve properly:

```rust
#[macro_export]
macro_rules! stack_panel {
    (
        vertical $(gap=$gap:expr)? $(items=$align:ident)?;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_col();
        $( panel = panel.gap(::gpui::px($gap as f32)); )?
        $( panel = $crate::stack_panel!(@align_items panel, $align); )?
        $( panel = panel.child($child); )*
        panel
    }};
    
    // ... horizontal matcher ...

    (@align_items $panel:ident, center) => { $panel.items_center() };
    (@align_items $panel:ident, start) => { $panel.items_start() };
    (@align_items $panel:ident, end) => { $panel.items_end() };
}
```

### 4.2 Module Structure
1. Create `crates/sdk/src/layout.rs` containing the macro definitions.
2. In `crates/sdk/src/lib.rs`, register the module:
   ```rust
   pub mod layout;
   ```
3. Re-export the layout macros in the SDK's prelude module so they are automatically in scope for all downstream developers:
   ```rust
   pub use gpui_luma::layout::*;
   ```

---

## 5. `dock_panel!` Macro & Cross-Framework Mappings

WPF's `DockPanel` docks child elements to the Top, Bottom, Left, or Right sides of the remaining screen area, letting the last child expand to fill the rest.

### 5.1 Cross-Framework Equivalents

* **SwiftUI**:
  SwiftUI doesn't feature a direct `DockPanel` container. Instead, developers nest `HStack` and `VStack` combinations combined with `Spacer()` elements. Alternatively, SwiftUI's `.safeAreaInset(edge:...)` modifier achieves docking behavior by pinning secondary views to the edges of a central scrolling view.
* **Flutter**:
  Flutter developers typically nest `Column` and `Row` widgets or utilize a `CustomMultiChildLayout` delegate. However, Flutter's **`Scaffold`** widget acts as a pre-configured, specialized `DockPanel`, offering distinct slots like `appBar` (Top), `bottomNavigationBar` (Bottom), `drawer` (Left), and `body` (Fill).

---

### 5.2 Mapping to GPUI / Flexbox

In GPUI (and CSS Flexbox), docking is represented by nested horizontal/vertical flex boxes:
1. Outer vertical container (`flex().flex_col()`):
   * Top docked element
   * Middle wrapper (`flex().flex_grow().overflow_hidden()`):
     * Left docked element
     * Central filled element (`flex_grow()`)
     * Right docked element
   * Bottom docked element

---

### 5.3 The `dock_panel!` Macro Concept

To avoid manually declaring these nested wrappers, the `dock_panel!` macro generates this hierarchical structure from a flat declaration:

```rust
macro_rules! dock_panel {
    (
        $(top = $top:expr;)?
        $(left = $left:expr;)?
        $(right = $right:expr;)?
        $(bottom = $bottom:expr;)?
        fill = $fill:expr $(;)?
    ) => {{
        // Middle horizontal wrapper (Left + Fill + Right)
        let mut middle = ::gpui::div().flex().flex_grow().overflow_hidden();
        $( middle = middle.child($left); )?
        middle = middle.child(::gpui::div().flex_grow().child($fill));
        $( middle = middle.child($right); )?

        // Outer vertical container (Top + Middle + Bottom)
        let mut outer = ::gpui::div().flex().flex_col().size_full();
        $( outer = outer.child($top); )?
        outer = outer.child(middle);
        $( outer = outer.child($bottom); )?
        
        outer
    }};
}
```

### Example Usage
```rust
dock_panel! {
    top = header_view(),
    left = sidebar_view(),
    bottom = footer_view(),
    fill = main_editor_workspace(),
}
```


