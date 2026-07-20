# Issue: Enhancing ToolbarBuilder

**Status:** Done

This document details proposed enhancements to `ToolbarBuilder` to provide a robust, fluent builder API. The goal is to support clean declarative structures, flexible focus strategies (such as native desktop tab loops), and unified event routing directly from the toolbar.

---

## Current Limitations

1. **Hardcoded Focus strategy:** 
   `Toolbar::from_builder` hardcodes the creation of the underlying `ControlGroupControl` with `.roving_item_focus()`. There is no way to opt-out or switch to standard sequential tab navigation.
2. **Scattered Subscription Logic:**
   The host view must keep references to each individual sub-control spawned (e.g., individual `Button<bool>` or `Selector` entities) to register subscriptions. This separates layout from behavior and leads to verbose boilerplate in the view's initialization.
3. **No Central Event Stream:**
   The `Toolbar` control itself acts purely as a visual manager and does not emit updates when sub-items are interacted with.

---

## Proposed Enhancements

### 1. Flexible Focus Strategies

We should expose the focus strategy directly on `ToolbarBuilder` to support standard sequential tabbing (macOS style) alongside roving focus (web style).

```rust
impl ToolbarBuilder {
    /// Configure the focus strategy for the toolbar
    pub fn focus_strategy(mut self, strategy: ControlGroupFocusStrategy) -> Self {
        self.model.focus_strategy = strategy;
        self
    }

    /// Helper to enable roving item focus (arrows navigate, Tab exits)
    pub fn roving_item_focus(self) -> Self {
        self.focus_strategy(ControlGroupFocusStrategy::RovingItemFocus)
    }

    /// Helper to use native sequential tab stops for all items (desktop style)
    pub fn sequential_focus(self) -> Self {
        self.focus_strategy(ControlGroupFocusStrategy::ActiveDescendant) // or new sequential enum variant
    }
}
```

**Shipped:** Builder helpers + `Toolbar::set_focus_strategy`. Gallery pane can switch roving vs sequential at runtime.

---

### 2. Unified Event Stream (`ToolbarEvent`)

Instead of requiring views to subscribe directly to every button or selector, the `Toolbar` should publish unified events:

```rust
#[derive(Clone, Debug)]
pub enum ToolbarEvent {
    /// Triggered when a button or toggle is clicked
    Click { id: SharedString },
    /// Triggered when a toggle/selector state changes
    Change { id: SharedString, value: ToolbarValue },
}

#[derive(Clone, Debug)]
pub enum ToolbarValue {
    Bool(bool),
    String(SharedString),
}
```

The hosting view can then subscribe to the `Toolbar` entity itself:

```rust
cx.subscribe(&toolbar, |view, _, event: &ToolbarEvent, cx| {
    match event {
        ToolbarEvent::Click { id } => {
            view.status = format!("{id} clicked");
        }
        ToolbarEvent::Change { id, value } => {
            view.status = format!("{id} changed to {value:?}");
        }
    }
    cx.notify();
});
```

**Shipped:** `ToolbarEvent` / `ToolbarValue`, child fan-in via `ToolbarItemSource`, gallery uses a single toolbar subscription.

---

### 3. Inline Item-Level Actions

We should support attaching callback closures directly to `ToolbarItem` on construction:

```rust
let toolbar = look
    .toolbar("toolbar-demo")
    .item(
        look.toolbar_toggle("bold", LucideIcon::Bold, cx)
            .on_click(|cx| {
                cx.notify();
            })
    )
    .item(
        look.toolbar_textfield("search")
            .placeholder("Search...")
            .spawn(cx)
            .on_change(|_value, cx| {
                cx.notify();
            })
    );
```

#### Under the Hood Implementation:
* `ToolbarItem` stores optional click/change handlers.
* When a sourced child fires, the toolbar runs the item callback (if any), then emits `ToolbarEvent`.

**Shipped:**
* SDK: `.on_click` / `.on_change` on `ToolbarItem`, plus source wrappers (`command_button`, `toggle_button`, …).
* Look: `ShadcnToolbarItemExt` fluent factories (`toolbar_button`, `toolbar_toggle`, `toolbar_textfield`, menus, selector) so apps stay look-bound and lookless SDK stays theme-free.

---

## Verification and Testing Plan

Testing of these builder enhancements will be done incrementally in the gallery app's [pane.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/toolbar/pane.rs).

1. Enhance `ToolbarBuilder` and `ToolbarItem` in `crates/sdk/src/controls/toolbar/`. ✅
2. Refactor the gallery toolbar pane to use the enhanced fluent API. ✅
3. Validate that standard button clicks, textfield changes, and focus strategy changes all function under the single unified event stream. ✅ (gallery: unified `ToolbarEvent` status + Roving/Sequential focus buttons)
