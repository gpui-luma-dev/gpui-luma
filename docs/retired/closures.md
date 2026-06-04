# SDK Control Closures & State Ownership Analysis

This document outlines the architectural problems regarding state ownership, data duplication, and closure signatures for each interactive control in the `gpui-luma` SDK, along with the proposed declarative remedies.

---

## Checkbox & Switch

### The Problem
Checkboxes and switches are currently spawned as stateful entities (`Entity<Button<bool>>` or custom view wraps). This model creates:
1. **Double Storage**: The checked boolean state is stored both in the parent view's state and inside the control's internal button model.
2. **Sync Boilerplate**: The parent view must subscribe to the control's change events, and then call `.update(cx)` on the control entity to programmatically sync values.

### The Remedy
Convert checkboxes and switches to **stateless visual elements** that return `impl IntoElement`. They should be declared inline in `render` and bind directly to the parent state:
```rust
// Proposed stateless API
pub fn switch(checked: bool) -> Switch { ... }

// Usage in render()
switch()
    .checked(self.notifications_enabled)
    .on_toggle(cx.listener(|this, checked, cx| {
        this.notifications_enabled = checked;
    }))
```

---

## RadioButton & RadioGroup

### The Problem
`RadioButton` is a type alias for a spawned entity (`Entity<Button<bool>>`), and `RadioGroup` is a spawned entity that owns a list of items and tracks selection internally.
- Changing selections requires capturing events, updating the parent state, and then calling `button.update(cx)` to sync the selection highlights.

### The Remedy
Render radio buttons as stateless elements configured dynamically in `render()` based on comparisons against the parent state:
```rust
// Usage in render()
vstack! {
    gap=4;
    radio_button()
        .label("Starter Plan")
        .selected(self.active_plan == Plan::Starter)
        .on_select(cx.listener(|this, _, cx| this.active_plan = Plan::Starter)),
    radio_button()
        .label("Pro Plan")
        .selected(self.active_plan == Plan::Pro)
        .on_select(cx.listener(|this, _, cx| this.active_plan = Plan::Pro)),
}
```

---

## Selector, Autocomplete & Combobox

### The Problem
These selection dropdowns are spawned entities that own duplicate copies of the list items and the selected ID. Changing options or selections programmatically requires calling `.update(cx, |s, cx| s.set_selected_id(...))` to force visual synchronization.

### The Remedy
Redesign them as stateless rendering overlays. The items and active selection ID are passed directly during the render pass, making them immediate lenses over the parent view model:
```rust
selector()
    .items(&self.available_themes)
    .selected(&self.active_theme_id)
    .on_change(cx.listener(|this, new_id, cx| this.change_theme(new_id, cx)))
```

---

## ListView (Scrolling & Paging)

### The Problem
The list view controls are spawned entities that own a duplicate list of items (`items: Vec<T>`). When items are added, removed, or updated in the application state, the list view must be imperatively updated with the new dataset, introducing sync friction.

### The Remedy
Decouple data ownership from layout state. The `ListView` entity should only track layout/view states (scroll position, active navigation index), while the items slice is passed dynamically during the render pass:
```rust
// The list view is rendered as a lens over the parent's data vector
self.list_view.render(cx, &self.tasks)
```

---

## NavigationSidebar

### The Problem
`NavigationSidebar` is a spawned entity that owns the hierarchical tree nodes (`NavNode`). Programmatic navigation switches or node expansions require imperatively modifying tree state inside the sidebar entity.

### The Remedy
Render `NavigationSidebar` as a stateless visual component. It takes the route taxonomy and the active route path directly in `render`, invoking navigation events upward:
```rust
navigation_sidebar()
    .routes(self.routes)
    .active_route(&self.current_route)
    .on_navigate(cx.listener(|this, new_route, cx| this.navigate(new_route, cx)))
```

---

## TreeView (`TreeViewControl`)

### The Problem
`TreeViewControl` is a spawned entity (`Entity<TreeViewControl<T>>`) that takes ownership of a duplicate data tree (`items: Vec<TreeNode<T>>`) and maintains its own internal flat row list caches.
- Programmatically selecting nodes, expanding paths, or replacing the items tree requires calling imperative update methods (like `.set_items(items, cx)`) on the control entity.
- The control stores the selected node IDs in an internal `selected_ids` set, forcing parent views to subscribe to change events to keep state synchronized.

### The Remedy
Decouple the data tree model from the control. The `TreeView` control entity should only track lightweight layout states (scroll offsets, active focus index, list of expanded node IDs). The data tree slice is passed dynamically during the render pass, making the tree view a stateless lens over the parent's data:
```rust
// The tree view is rendered inline, borrowing the parent's tree model
self.tree_view.render(cx, &self.file_tree, |node| {
    // Return the visual row representation
    tree_row(node.label)
        .icon(node.icon)
        .selected(self.selected_node_id == node.id)
})
```
This removes the necessity for `set_items` imperative synchronizations and manual state tracking.
