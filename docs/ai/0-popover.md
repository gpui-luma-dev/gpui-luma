# PopoverButton Control Design Document

## Problem & Motivation

In modern desktop applications, "clicking something pops up something visually associated" is one of the most common UI patterns. Examples include color swatches opening color field pickers, tab items opening dropdown catalog menus, date fields opening calendar cards, and avatars opening user profile popovers.

Previously, components used `AnchoredPanel` by calling `set_anchor_bounds` during prepaint on one entity and passing bounds to another. In scrollable layouts (like the Luma Studio Palette grid), this split-entity ownership caused:

1. **Prepaint-to-Render Frame Lag**: Bounds measured during Pass 3 (prepaint) were read during Pass 1 (render) of the next frame, making popover placement lag 1 frame behind scrolling.
2. **Relative Stacking Context Offsets**: Nesting panels inside `.relative()` scroll containers caused GPUI's `anchored()` primitive to add parent relative offsets to window coordinates, double-counting container offsets (e.g. 55px vertical shifts).
3. **Unmeasured First-Frame Height**: Unmeasured initial content sizes caused `Smart` placement to jump between `BelowStart` and `AboveStart` across frames.
4. **Duplicate ID Collisions**: Non-unique string IDs across grid elements caused prepaint bounds to overwrite each other across grid cells.

`AnchoredPanel` is a low-level primitive intended for externally-measured anchors (such as text selection glyphs or canvas node bounding boxes). For 95% of application UIs, popovers should be **trigger-owned, single-entity controls**.

---

## Architecture: The `PopoverButton` Control

`PopoverButton<D = ()>` is a high-level, abuse-proof SDK control that unifies trigger measurement, open/close state, focus management, visual active state, and deferred overlay rendering into a single control lifecycle.

```text
PopoverButton<D>
  ├─ Root Container (listens for focus, escape, outside click)
  ├─ Internal PopupLifecycle (presence, guarded open, outside-click exclusion)
  ├─ Trigger Presenter (swatch, tab item, avatar, icon, button)
  │   └─ Measured via on_children_prepainted
  └─ Deferred Overlay Host
      └─ deferred(anchored(popover content))
```

### Core Design Principles

1. **Single-Entity Ownership**: The `PopoverButton` entity owns open state, focus trap, presence transition (`OverlayPresence`), and trigger bounds. No cross-entity bounds passing is required.
2. **Real-Time Trigger Measurement**: Uses `.on_children_prepainted()` on the root element to measure the exact trigger child's window bounds automatically on every frame.
3. **Internal `PopupLifecycle` Integration**: Consumes the SDK's existing `PopupLifecycle` primitive to guarantee:
   - **Guarded Opening (`toggle_guarded_from`)**: Protects against frame-1 self-dismissal from the opening click event.
   - **Multi-Surface Exclusion**: Automatically excludes both trigger bounds and popup content bounds from outside-click dismissals.
   - **Transition Retargeting**: Retargets `OverlayPresence` smoothly if the trigger is clicked rapidly while closing.
4. **Visual Link (`PopoverVisualState`)**: The trigger presenter receives `PopoverVisualState { open: bool, hovered: bool, focused: bool }` so the trigger stays visually highlighted (e.g., active ring outline, active theme background, chevron rotation) while its popover is open.
5. **Zero-Gap Placement Engine**: Supports `Smart` (auto below/above), `BelowStart`, `BelowCenter`, `AboveStart`, and `RightEnd` with 0-lag window coordinate snapping.

---

## Focus & Dismissal Lifecycle Contract

### Dismissal Policies

`PopoverDismissPolicy` mirrors proven desktop overlay behavior:

```rust
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PopoverDismissPolicy {
    #[default]
    CloseOnClickAway,
    CloseOnFocusLoss,
    CloseOnClickAwayOrFocusLoss,
    KeepOpen,
}
```

- **`CloseOnClickAway`**: Closes when pointer down occurs outside both the trigger bounds and popup content bounds.
- **`CloseOnFocusLoss`**: Closes when keyboard focus leaves the popup container.
- **`CloseOnClickAwayOrFocusLoss`**: Standard for tab menus and dropdowns.
- **`KeepOpen`**: Modal mode requiring explicit button dismiss.

### Focus Management & Escape Precedence

1. **Focus on Open**: Controlled via `.focus_on_open(bool)`. When `true`, focuses the popup content on open for immediate keyboard navigation; when `false`, focus remains on the trigger button.
2. **Escape Key Precedence**: Pressing Escape immediately closes the open popover and restores keyboard focus to the trigger button.
3. **Reason-Aware Focus Restoration**:
   - **Escape / Trigger Toggle**: Unconditionally restores focus to the opener trigger.
   - **Click-Away / Focus Loss**: Preserves focus on the newly targeted control without stealing focus.

---

## Detailed Use Cases

### Use Case 1: Color Swatch + Color Picker (`ColorPickerPopover`)

#### Trigger Presenter
The color swatch trigger uses `ColorSwatchButtonTemplate` to paint the checkerboard, color fill, single quad border, and hover/focus/active state directly on the canvas surface.

#### Visual Link
When the popover is open (`state.open == true`), the swatch button renders an active focus/ring outline around its border, making it visually obvious to the user which swatch in the grid owns the open picker.

#### Placement & Scrolling
- **Placement**: `PopoverPlacement::Smart` (attaches top edge of picker flush to `trigger.bottom` when space permits, or bottom edge flush to `trigger.top` near window bottom).
- **Scrolling**: As the Palette grid scrolls, `on_children_prepainted` updates the trigger bounds in real time, keeping the picker attached to the swatch.

#### Structure
```rust
let picker = look
    .popover_button(format!("palette-picker-{token}"))
    .typed(color)
    .trigger(move |model, cx| {
        ColorSwatchButton::new(model.data)
            .active(model.visual_state.open)
            .render(cx)
    })
    .content(move |_, cx| {
        render_color_picker_view(&color_field, &hue_slider, &alpha_slider, cx)
    })
    .placement(PopoverPlacement::Smart)
    .dismiss_policy(PopoverDismissPolicy::CloseOnClickAway)
    .spawn(cx);
```

---

### Use Case 2: Tabs Navigation Dropdown Menu (`ContentPaneHost` Tab + Drop Menu)

#### Trigger Presenter
The "Controls" tab item in `ContentPaneHost` acts as the popover trigger, rendering the tab label ("Controls") alongside a disclosure chevron indicator (`ChevronDown` / `ChevronUp`).

#### Visual Link
When the catalog dropdown menu is open (`state.open == true`):
- The tab item background and text retain active/selected tab styling.
- The disclosure chevron rotates 180° (`0° → 180°`) using `VisualTransition`.

#### Placement & Alignment
- **Placement**: `PopoverPlacement::BelowStart` or `PopoverPlacement::BelowCenter` (attaches top-left of the catalog card flush to the bottom of the "Controls" tab button).
- **Dismiss Policy**: `PopoverDismissPolicy::CloseOnClickAwayOrFocusLoss` (dismisses when clicking outside or navigating to a different main tab).

#### Structure
```rust
let catalog_menu = look
    .popover_button("luma-studio-controls-catalog-popover")
    .trigger(move |model, cx| {
        render_tab_item_with_chevron("Controls", model.visual_state.open, cx)
    })
    .content(move |_, cx| {
        render_control_catalog_picker(cx)
    })
    .placement(PopoverPlacement::BelowStart)
    .dismiss_policy(PopoverDismissPolicy::CloseOnClickAwayOrFocusLoss)
    .spawn(cx);
```

---

## SDK Module Layout & Typed API Surface

```text
crates/sdk/src/controls/popover_button/
  ├── mod.rs        // Public exports and type aliases
  ├── model.rs      // PopoverModel, PopoverPlacement, PopoverDismissPolicy, PopoverVisualState
  ├── control.rs    // PopoverButton entity and event handling (backed by PopupLifecycle)
  ├── template.rs   // DefaultPopoverButtonTemplate and presenter resolution
  └── builder.rs    // PopoverButtonBuilder and ShadcnLook extension
```

### Complete Builder API (`PopoverButtonBuilder<D>`)

```rust
pub struct PopoverButtonBuilder<D = ()> {
    id: SharedString,
    data: D,
    trigger: Option<ControlPresenter<PopoverRenderModel<D>>>,
    content: Option<ControlPresenter<PopoverRenderModel<D>>>,
    placement: PopoverPlacement,
    dismiss_policy: PopoverDismissPolicy,
    offset_y: Pixels,
    window_margin: Pixels,
    focus_on_open: bool,
    animated: bool,
}

impl<D: Default + 'static> PopoverButtonBuilder<D> {
    pub fn new(id: impl Into<SharedString>) -> Self;
    pub fn typed(mut self, data: D) -> Self;
    pub fn trigger(mut self, presenter: impl Fn(&PopoverRenderModel<D>, &mut Window, &mut App) -> AnyElement + 'static) -> Self;
    pub fn content(mut self, presenter: impl Fn(&PopoverRenderModel<D>, &mut Window, &mut App) -> AnyElement + 'static) -> Self;
    pub fn placement(mut self, placement: PopoverPlacement) -> Self;
    pub fn dismiss_policy(mut self, policy: PopoverDismissPolicy) -> Self;
    pub fn offset_y(mut self, offset: impl Into<Pixels>) -> Self;
    pub fn window_margin(mut self, margin: impl Into<Pixels>) -> Self;
    pub fn focus_on_open(mut self, focus: bool) -> Self;
    pub fn animated(mut self, animated: bool) -> Self;
    pub fn spawn(self, cx: &mut Context<PopoverButton<D>>) -> Entity<PopoverButton<D>>;
}
```

### `ShadcnLookControlExt` Extension

```rust
pub trait ShadcnLookControlExt {
    fn popover_button<D: Default + 'static>(&self, id: impl Into<SharedString>) -> PopoverButtonBuilder<D>;
}
```

---

## Migration Strategy

1. **SDK Primitive Restriction**: Mark `AnchoredPanel` as `pub(crate)` within `crates/sdk/src/controls/anchored_panel/`, restricting its use to SDK-internal advanced primitives.
2. **Implement `PopoverButton`**: Add `crates/sdk/src/controls/popover_button/` backed by `PopupLifecycle` with full template and presenter support.
3. **Migrate `ColorPickerPopover`**: Update `apps/luma-studio/src/studio/controls/color_picker.rs` to use `PopoverButton`.
4. **Migrate `ContentPaneHost`**: Update `apps/luma-studio/src/studio/content_tabs/host.rs` to use `PopoverButton` for the catalog picker dropdown, removing the manual `catalog_picker` entity and `AnchoredPanel` subscriptions.

---

## Verification Checklist

- [ ] **Color Swatch Picker**: Clicking `Foreground` opens the picker flush below the `Foreground` swatch with 0px vertical gap.
- [ ] **Tab Dropdown Menu**: Clicking the "Controls" tab opens the catalog picker flush below the "Controls" tab item, with the chevron rotating to 180°.
- [ ] **Scroll Stability**: Scrolling the Palette grid keeps open color pickers attached to their respective swatches without frame lag.
- [ ] **Smart Flip**: Swatches near the bottom of the window flip to `AboveStart` flush against the swatch's top edge.
- [ ] **Guarded Open**: Clicking the swatch/tab never triggers false immediate self-dismissal on frame 1.
- [ ] **Focus & Escape**: Pressing Escape closes the open popover and restores keyboard focus to the opener trigger.
- [ ] **Unique IDs**: Every instance uses a unique string ID (`format!("palette-picker-{token}")`), preventing bounds collisions across grid elements.
