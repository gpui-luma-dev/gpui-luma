# Issue #0: Unified Theme Synchronization using GPUI Global Observation

## Description
When the global theme or light/dark mode switch is toggled, persistent view entities (such as `TextField`, `TextArea`, and the trigger button of `Selector`/`ComboBox`) do not update their visual styling. This happens because GPUI caches element rendering for views, and these persistent control entities have no native way of knowing the global theme state has changed to call `cx.notify()` on themselves.

Previously, custom "notification buses" ran into synchronization, memory leak, and re-entrancy borrow panics. As a workaround, the Gallery app uses manual tree-walking/propagation methods (`notify_controls`), while the Theme Studio recreates the entire board of panels (`refresh_demos`).

The most idiomatic and robust solution is to leverage GPUI's native **Global Observation** pattern. By wrapping the active look in a GPUI `Global`, persistent controls can subscribe to updates during their initialization. GPUI queues these notifications safely to prevent re-entrancy panics and automatically handles cleanup when views are dropped.

## Proposed Design

### 1. Define the Global Wrapper
In the styling/look integration layer (e.g., `gpui-luma-look-shadcn`), wrap the active look in a type implementing `gpui::Global`:

```rust
pub struct ActiveLook(pub Arc<ShadcnLook>);

impl gpui::Global for ActiveLook {}
```

### 2. Register Global Observations inside Control Constructors
For all persistent themeable controls in the SDK (e.g., `TextFieldControl`, `TextAreaControl`, `Selector`, `Button`), subscribe to the global look within their `from_builder` constructors:

```rust
impl<T> Selector<T>
where
    T: SelectorItemLike + 'static,
{
    pub(crate) fn from_builder(builder: SelectorBuilder<T>, cx: &mut Context<Self>) -> Self {
        // Observe the global active look for theme/mode shifts
        cx.observe_global::<ActiveLook>(|this, cx| {
            // Clear any layout cache and repaint
            this.layout_cache = None;
            cx.notify();
        }).detach();

        // ... rest of constructor
    }
}
```

### 3. Cleanup Legacy Workarounds
Once global observation is implemented:
- Simplify `ThemeStudioApp`'s mode toggle to update the global look and trigger view notifies.
- Remove manual `notify_controls` tree-walking code from the Gallery app.

## Tasks
- [ ] Create the `ActiveLook` global wrapper implementing `gpui::Global`.
- [ ] Add `cx.observe_global::<ActiveLook>` subscriptions in:
  - [ ] `TextFieldControl` constructor in [textfield/control.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/textfield/control.rs)
  - [ ] `TextAreaControl` constructor in [textarea/control.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/textarea/control.rs)
  - [ ] `Selector` constructor in [selector/control.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/selector/control.rs)
  - [ ] `Button` constructor in [command/button/control.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/command/button/control.rs)
- [ ] Update Theme Studio and Gallery app title-bar toggles to mutate the global look.
- [ ] Remove manual pane notification methods (`notify_controls`) from [apps/gallery/src/gallery/panes/registry.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/registry.rs).

## Acceptance Criteria
- Toggling the light/dark mode switch in either app instantly updates all persistent UI components (text fields, textareas, buttons, and dropdown selector trigger buttons) on screen.
- Theme switching does not produce re-entrancy `BorrowMutError` panics or memory leaks when panels are mounted/unmounted.
