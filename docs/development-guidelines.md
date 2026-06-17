# GPUI-Luma Development Guidelines

This document details practical implementation playbooks, coding standards, macro layout helpers, re-entrancy warnings, and verify checklists for both SDK controls and consumer applications.

---

## 1. SDK Implementation Guidelines

When writing or modifying controls in the SDK, always adhere to the following playbooks:

### 1.1 Model & Builder Patterns
Builders own initial models and build them cheap.
*   Use `SharedString` for user-facing texts (IDs, labels, placeholders).
*   Spawn entities using `.spawn(cx)` which wraps `cx.new(|cx| ...)`.
*   Store templates as `Arc<dyn <Control>Template>`.
*   Coerce values (min/max range snapping) at the builder stage using `ControlRange` to avoid layout drifts during draw cycles.

### 1.2 Shared Interaction Primitives
Always reuse core SDK interaction layers instead of building custom pointer listeners:
*   Use `ControlInteraction` for single-surface controls to automatically manage hover, press, disabled, focus, and tab-stop states.
*   Use `CompositeItemState` for multi-item controls (e.g., choice rows or navigation lists) to map active/selected/hover indicators.
*   Disabled controls must clear hover/press states immediately and remove themselves from the GPUI tab-traversal ring.

### 1.3 Pointer & Keyboard Event Parity
Keep pointer and keyboard events bound to the same semantic execution path.
```rust
fn activate(&mut self, cx: &mut Context<Self>) -> bool {
    if !self.model.enabled {
        return false;
    }
    cx.emit(ButtonEvent::Click);
    true
}
// Pointer handler:
fn handle_click(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
    self.activate(cx);
}
// Keyboard handler:
fn handle_activate_control(&mut self, _: &ActivateControl, _: &mut Window, cx: &mut Context<Self>) {
    self.activate(cx);
}
```

---

## 2. Application Best Practices (`apps/*`)

Consumer applications must compose SDK controls, not reinvent them.

### 2.1 Use Layout Helper Macros (Strict Rules)
In `apps/gallery` and all other application layout trees, **raw tail-chained flex layouts are strictly prohibited** for multi-child layouts. You must reuse the layout macros defined in `gpui_luma::macros` (`vstack!`, `hstack!`, `wrappanel!`, `dock_panel!`, `declare_form!`).

#### The Layout Rule:
If a container renders **more than one child**, you must use `vstack!`, `hstack!`, or `wrappanel!` instead of manually chaining `.flex().flex_col().gap(...)`.

##### 1. Vertical Stack (`vstack!`)
*   **Prohibited:**
    ```rust
    div()
        .flex()
        .flex_col()
        .gap(px(16.0))
        .items_center()
        .justify_center()
        .child(header)
        .child(body)
    ```
*   **Required:**
    ```rust
    vstack! {
        gap=16.0 align=center justify=center;
        header,
        body,
    }
    ```

##### 2. Horizontal Stack (`hstack!`)
*   **Prohibited:**
    ```rust
    div()
        .flex()
        .gap(px(8.0))
        .items_center()
        .child(icon)
        .child(label)
    ```
*   **Required:**
    ```rust
    hstack! {
        gap=8.0 align=center;
        icon,
        label,
    }
    ```

##### 3. Grid / Wrap Flow (`wrappanel!`)
*   **Prohibited:**
    ```rust
    div()
        .flex()
        .flex_wrap()
        .gap(px(10.0))
        .child(item_a)
        .child(item_b)
    ```
*   **Required:**
    ```rust
    wrappanel! {
        gap=10.0;
        item_a,
        item_b,
    }
    ```
    *Note: `flow!` is a legacy alias for `wrappanel!(orientation=horizontal)`. Use `wrappanel!` directly in new code.*

##### Permitted Exceptions (Plain Containers):
You may use `div()` directly **only** for single-child structural wrappers, such as:
1. Adding padding around a sub-tree: `div().p(px(12.0)).child(content)`
2. Absolute overlays: `div().absolute().top_0().child(overlay)`
3. Theme/background borders: `div().bg(look.color(ShadcnToken::Card)).child(inner)`

### 2.2 Prototype Structure
When writing a new prototype inside `apps/gallery` or `apps/theme-studio`:
*   Separate the demo pane (scaffolding shell, setup controls, copy texts) from the core control being evaluated.
*   **Recommended pattern:** Keep `pane.rs` as the demo/shell container, and place the actual prototype behavior/rendering engine inside a sibling file (e.g. `control.rs` or `template.rs`). This allows successful prototypes to graduate into the SDK crate with minimal untangling.

---

## 3. Rust Code Style Conventions

*   **File LOC Limit:** Keep `.rs` files under **300 lines of code** where possible. Split modules aggressively at logical boundaries.
*   **Function LOC Limit:** Keep functions under **50 lines of code**.
*   **Safety Invariant:** All code must forbid unsafe behavior:
    ```rust
    #[forbid(unsafe_code)]
    ```
*   **Conventions:** Prefer newtypes for domain wrappers, leverage standard `#[derive(Debug, Clone, PartialEq, Eq)]` macros, use `thiserror` for library-level error reporting, and always format code via `cargo fmt` and check with `cargo clippy`.

---

## 4. GPUI Entity Re-entrancy & Snapshots

To prevent runtime leasing panics (`cannot read/update App while already being updated`), observe the **Snapshot Discipline**:

```text
               ThemeStudioApp ( authoritative coordinator )
                             ↓
             computes immutable BoardSnapshot
                             ↓
                 pushes snapshot downward
                             ↓
             ContentPaneHost ( stores local snapshot )
                             ↓
          Renders entirely from local snapshot (No reads!)
```

### Guideline Rules
1.  **Read Path:** Child views must never call `.read(cx)` on a parent or sibling entity inside their `Render::render` method. Instead, parents push an immutable `*Snapshot` copy down during layout updates.
2.  **Write Path:** When a child view mutates parent state, trigger `parent.update(..)` from an event handler, and re-cache any local values *after* the update closure has completed and returned.
3.  **Subscriptions:** When subscribing via `cx.subscribe`, keep in mind that the handler updates the subscriber. Avoid calling `subscriber.update` inside its own event hook.

---

## 5. Pre-Merge Verification Checklist

Verify this checklist before finalizing any changes:
- [ ] Did I format using `cargo fmt`?
- [ ] Does `cargo clippy --all-targets` compile clean without warnings?
- [ ] Am I using layout macros instead of raw flex chains?
- [ ] Are all builders cheap and value snapping/coercion applied early?
- [ ] Do pointer and keyboard activation paths share a unified handler?
- [ ] If I'm using parent/child views, does `render` avoid all `.read` calls on parent entities?
- [ ] If this is a prototype, is it structured cleanly to graduate to the SDK?
