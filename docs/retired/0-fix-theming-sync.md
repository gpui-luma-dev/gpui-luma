# Issue #0: Unified Theme Synchronization using GPUI Global Observation

## Description
When the global theme or light/dark mode switch is toggled, persistent view entities (such as `TextField`, `TextArea`, and the trigger button of `Selector`/`ComboBox`) do not update their visual styling. This happens because GPUI caches element rendering for views, and these persistent control entities have no native way of knowing the global theme state has changed to call `cx.notify()` on themselves.

Previously, custom "notification buses" ran into synchronization, memory leak, and re-entrancy borrow panics. As a workaround:
- The **Gallery** app uses manual tree-walking/propagation methods (`notify_controls`).
- The **Theme Studio** app recreates the entire board of panels (`refresh_demos`) upon override, but misses updating them on mode toggle.
- The **Neumorphic Demo** app uses static, specialized neumorphic templates independent of the global look but still uses the same SDK control definitions.

The most idiomatic and robust solution is to leverage GPUI's native **Global Observation** pattern. By wrapping the active look in a GPUI `Global`, persistent controls can subscribe to updates during their initialization. GPUI queues these notifications safely to prevent re-entrancy panics and automatically handles cleanup when views are dropped.

---

## App-Specific Audits

### 1. Theme Studio (`apps/theme-studio`)
- **Current Behavior:** Changes to global color variables trigger `apply_theme_overrides(cx)`, which in turn calls `refresh_demos(cx)` to rebuild the demo panel entities from scratch. However, toggling the light/dark mode switch in the title bar only mutates `look.mode` and calls `refresh_content_pane(cx)`. It does not notify or recreate the demo panel views, leaving all persistent inputs, buttons, and selector triggers in the old mode.
- **Target Design:** Toggling the mode will update the global `ActiveLook` wrapper. All persistent view entities on the board will automatically invalidate their caches, removing the need for manual panel recreation.

### 2. Gallery App (`apps/gallery`)
- **Current Behavior:** Toggling the light/dark switch calls `this.panes.notify_controls(cx)`. This function explicitly calls `notify_entity` on dozens of individual component handles across all panes.
- **Target Design:** The global `ActiveLook` change automatically triggers view invalidation in all active controls. The entire `notify_controls` plumbing in [registry.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/registry.rs) and the individual panes can be completely removed.

### 3. Neumorphic Demo (`apps/neumorphic-demo`)
- **Current Behavior:** Uses custom templates (`neumorphic_switch_template`, etc.) that render neumorphic shadows directly. Because it does not use `ShadcnLook` or a theme mode toggle, it remains styled statically.
- **Target Design:** By integrating `observe_global::<ActiveLook>` inside the SDK controls, we use `cx.try_global::<ActiveLook>()`. If no global look is registered, the controls fall back gracefully, ensuring the Neumorphic Demo continues to work seamlessly without dependency on the global theme.

---

## SDK Control-by-Control Audit

The following persistent control view entities in `crates/sdk` must be updated to participate in the theme-change subscription:

| Control | Crate Path | Lifecycle & Rendering |
| :--- | :--- | :--- |
| **TextField** | [textfield/control.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/textfield/control.rs) | Persistent view entity. Caches text layout previews and colors. |
| **TextArea** | [textarea/control.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/textarea/control.rs) | Persistent view entity. Handles caret and line caching. |
| **Selector** | [selector/control.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/selector/control.rs) | Persistent view entity. Its trigger button is always visible and cached. |
| **ComboBox** | [combobox/control.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/combobox/control.rs) | Persistent view entity wrapping textfields and select options. |
| **Button / Toggle** | [command/button/control.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/command/button/control.rs) | Persistent view entity. Caches state interactions and borders. |
| **TabsNavigation** | [tabs_navigation/control.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/tabs_navigation/control.rs) | Persistent view entity. Highlights tab indicators dynamically. |
| **NavigationSidebar** | [navigation_sidebar/control.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/navigation_sidebar/control.rs) | Persistent view entity. Caches tree node layout. |
| **Slider** | [slider/control.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/slider/control.rs) | Persistent view entity. Caches knob and track positions. |
| **Scrollbar** | [scrollbar/control.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/scrollbar/control.rs) | Persistent view entity. Caches viewport offsets. |
| **Progress** | [progress/control.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/progress/control.rs) | Persistent view entity. Caches bar fill level. |
| **Accordion** | [accordion/control.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/accordion/control.rs) | Persistent view entity. Caches collapse/expand heights. |
| **TreeView** | [tree_view/control.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/tree_view/control.rs) | Persistent view entity. Caches open/closed branches. |

---

## Proposed Design & Implementation Plan

### 1. Global Wrapper Definition
Define the `ActiveLook` wrapper in the SDK or look crate:

```rust
pub struct ActiveLook(pub Arc<ShadcnLook>);

impl gpui::Global for ActiveLook {}
```

### 2. Standard Observation Helper Pattern
To simplify observation across all persistent SDK controls, add a helper or directly subscribe inside control constructors (`from_builder`):

```rust
// Inside control constructor (e.g. from_builder in selector/control.rs)
if cx.has_global::<ActiveLook>() {
    cx.observe_global::<ActiveLook>(|this, cx| {
        // Clear caches
        this.layout_cache = None;
        cx.notify();
    }).detach();
}
```

### 3. Mutating the Global Theme Mode
When toggling theme mode (or loading new color schemes), update the global instance:

```rust
cx.update_global::<ActiveLook>(|active, cx| {
    active.0.set_mode(new_mode);
    // GPUI automatically schedules redraws for all observers here
});
```

---

## Verification Plan

### Manual Verification Checklist
- [ ] Run Theme Studio (`just theme-studio`). Set global color overrides, then toggle light/dark mode. Verify all textfields, select inputs, textareas, and buttons update immediately.
- [ ] Run Gallery (`just gallery`). Verify theme changes apply instantly on all pages without manual propagation.
- [ ] Run Neumorphic Demo (`just neumorphic-demo` or cargo run). Confirm that controls resolve correctly using their custom templates and do not panic due to the missing `ActiveLook` global.
