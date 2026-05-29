# ListView Refinements & Declarative Macros (Phase 2)

This document outlines the Phase 2 goals for refining the `ListView` control in `gpui-luma`, focusing on a declarative macro construction syntax similar to WPF's XAML.

---

## 1. The `list_view!` Macro Goal

The goal is to provide a clean, code-first declarative macro `list_view!` that eliminates builder boilerplate and makes list view instantiation self-documenting.

### 1.1 Simple ItemTemplate View

For a simple single-column list, the macro allows specifying the backing collection and the item rendering template inline:

```rust
let list_view = list_view! {
    id = "user-list";
    items = load_10000_users();
    selection = ListSelectionMode::Single;
    
    // Automatically binds the item mapping
    item_template = |model, user, window, cx| {
        div()
            .flex()
            .items_center()
            .gap_4()
            .child(user.name.clone())
            .child(user.email.clone())
    };
}.spawn(cx);
```

### 1.2 Removing `ListViewItemLike` Boilerplate

Previously, any custom type `T` used in `ListView<T>` was required to implement the `ListViewItemLike` trait, forcing boilerplate like:

```rust
// BEFORE (Required boilerplate):
struct DemoUser {
    name: SharedString,
    email: SharedString,
}

impl ListViewItemLike for DemoUser {
    fn label(&self) -> &SharedString {
        &self.name
    }
}
```

With the new design, the `ListViewItemLike` trait bound is completely removed. You can now pass any plain Rust struct directly without implementing any traits:

```rust
// AFTER (No boilerplate needed!):
struct DemoUser {
    name: SharedString,
    email: SharedString,
}
```

### 1.3 GridView Column Bindings

For multi-column table views, the macro will support a grid layout syntax that maps cells to columns dynamically, mimicking WPF's `GridView`:

```rust
let grid_view = list_view! {
    id = "user-grid";
    items = load_10000_users();
    selection = ListSelectionMode::Multiple;
    
    // Renders as a multi-column table layout with headers
    grid_view = {
        column!("Name", width=180 => |user| user.name.clone()),
        column!("Email", width=250 => |user| user.email.clone()),
        column!("Role", width=120 => |user| user.role.clone()),
    };
}.spawn(cx);
```

---

## 2. Macro Architecture & Implementation Plan

### 2.1 Macro Definition

The macro will be defined in `crates/sdk/src/controls/list_view/macros.rs` and exported at the crate root. It parses key-value style configuration arguments:

```rust
#[macro_export]
macro_rules! list_view {
    (
        id = $id:expr;
        items = $items:expr;
        $( selection = $selection:expr; )?
        $( selected_index = $selected_index:expr; )?
        $( active_index = $active_index:expr; )?
        $( item_template = |$model:ident, $item:ident, $win:ident, $cx:ident| $body:expr; )?
        $( header_template = |$hdr_model:ident, $hdr_win:ident, $hdr_cx:ident| $hdr_body:expr; )?
    ) => {
        {
            let builder = $crate::controls::list_view::new_typed($id)
                .items($items);
            
            $( let builder = builder.selection_mode($selection); )?
            $( let builder = builder.selected_indices(vec![$selected_index]); )?
            $( let builder = builder.active_index($active_index); )?
            
            $(
                let builder = builder.item_template(move |$model, $win, $cx| {
                    let $item = $model.item;
                    $body
                });
            )?

            $(
                let builder = builder.header_template(move |$hdr_model, $hdr_win, $hdr_cx| {
                    $hdr_body
                });
            )?

            builder
        }
    };
}
```

### 2.2 GridView Auto-Layout Mechanics

When `grid_view` is specified instead of `item_template`, the macro will:
1. Automatically construct a horizontal layout row template.
2. Distribute columns using specified widths (flexible or fixed).
3. Generate the `header_template` automatically using the specified column headers.
