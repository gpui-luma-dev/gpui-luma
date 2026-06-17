# Split View Work Plan

This document defines the first Luma `SplitView` effort. The control is inspired by macOS
`NavigationSplitView`, but should be designed around the needs and architecture of GPUI-Luma.

The first real consumer is the gallery app shell.

## Goal

Build a Luma-native split view control for navigation/content application layouts.

The initial version should provide a two-pane layout:

- a sidebar pane for navigation,
- a content pane for the active work area,
- a draggable separator,
- configurable sidebar width constraints,
- optional sidebar collapse and expand behavior,
- fill-parent layout suitable for application shells.

The control should follow the SDK's current control shape:

```text
crates/sdk/src/controls/split_view/
  control.rs
  model.rs
  template.rs
  mod.rs
```

Behavior belongs in the control entity, stored configuration belongs in the model and builder, and
visual structure belongs in the template.

## Non-Goals

The first version is not:

- a complete arbitrary layout manager,
- a docking system,
- a window manager,
- a general-purpose resizable panel group,
- a direct port of the Opal experimental implementation,
- a full adaptive navigation framework.

The control can grow toward those use cases only after the gallery and other real consumers create
clear pressure for them.

## Initial Scope

The first implementation should be intentionally narrow:

- two panes: sidebar and content,
- sidebar on the left,
- pixel-based sidebar width,
- minimum and maximum sidebar width,
- collapsed and expanded states,
- optional collapsed width,
- pointer resizing through a vertical separator,
- resize start/change/end events,
- no fixed internal frame width or height,
- no Opal chrome or theme dependency,
- gallery integration.

The split view should usually fill the space given by its parent. The parent app shell should own
outer application sizing.

## Later Scope

Likely later additions:

- right-side sidebar placement,
- three-pane mode: sidebar, content, detail,
- compact icon rail,
- adaptive behavior for narrow widths,
- keyboard separator focus and resizing,
- template and theme customization beyond the default separator,
- persistence helpers for width and collapsed state,
- reusable pair-resize logic for multiple panes.

These should not block the first gallery shell version.

## Public API Sketch

The builder should feel like the existing SDK controls: deterministic, typed, and cheap to
construct.

```rust
let split_view = SplitView::new("gallery-shell")
    .sidebar_width(px(280.0))
    .sidebar_min_width(px(220.0))
    .sidebar_max_width(px(420.0))
    .sidebar_collapsed_width(px(0.0))
    .collapsed(false)
    .resizable(true)
    .spawn(cx);
```

The exact slot API still needs a final decision. Two viable directions are:

```rust
SplitView::new("gallery-shell")
    .sidebar(|| navigation)
    .content(|| active_page)
```

or an entity-driven shell where the owning app provides sidebar/content elements through mutation
methods before render.

Prefer the approach that fits GPUI-Luma's current entity, builder, and template patterns with the
least special casing.

## Model Sketch

The model should describe behavior and structure, not app-specific presentation.

```rust
pub struct SplitViewModel {
    id: SharedString,
    sidebar_width: Pixels,
    sidebar_min_width: Pixels,
    sidebar_max_width: Pixels,
    sidebar_collapsed_width: Pixels,
    collapsed: bool,
    resizable: bool,
    enabled: bool,
    separator_visibility: SplitViewSeparatorVisibility,
    template: Arc<dyn SplitViewTemplate>,
}
```

Potential supporting types:

```rust
pub enum SplitViewSidebarPlacement {
    Left,
    Right,
}

pub enum SplitViewSeparatorVisibility {
    Always,
    Hover,
}
```

Only add `SplitViewSidebarPlacement` in the first version if right placement is actually included.

## Events

The control should emit semantic events rather than exposing pointer details.

```rust
pub enum SplitViewEvent {
    ResizeStart,
    SidebarWidthChanged { width: Pixels },
    ResizeEnd { width: Pixels },
    CollapsedChanged { collapsed: bool },
}
```

Resize changes should clamp to the configured sidebar min and max widths before emission.

## Behavior Rules

- Width changes should be clamped to `sidebar_min_width..=sidebar_max_width`.
- Collapse should not destroy the last expanded width.
- Expanding should restore the previous clamped expanded width.
- Dragging the separator should emit `ResizeStart` once, `SidebarWidthChanged` on meaningful
  changes, and `ResizeEnd` once.
- Disabled or non-resizable split views should not start drags.
- The visible separator can be narrow, but the pointer hit target should be comfortably wider.
- The control should avoid fixed frame dimensions and should compose inside parent layouts.

## Template Notes

The default template should render:

- root fill container,
- sidebar lane,
- separator hit lane,
- separator visual cue,
- content lane.

The separator should use a wide hit lane and a narrower visual cue. The Opal prototype used a
20-pixel hit lane with a 4-pixel idle cue and 8-pixel hover cue. That is a reasonable starting
point, but final values should be expressed through the Luma template/theme path rather than copied
as global design policy.

## Opal Salvage Notes

The old Opal experimental code is useful as prior art, not as a direct source transplant.

Prior art source:

- `gpui-opal/crates/sdk/src/controls/layout/split_view`
- `gpui-opal/crates/sdk/src/controls/layout/resizable`

Good concepts to reuse:

- sidebar/content terminology,
- collapsed state and effective width calculation,
- min/max width clamping,
- separator hit lane and cue geometry,
- drag start axis tracking,
- drag delta signing for left/right placement,
- click suppression after a real drag,
- pair-resize min/max math from the generic resizable control for future three-pane work.

The collapse and expand paths are especially important. All internal separator clicks and external
application controls should route through `set_collapsed` or `toggle_collapsed`, so the event
emission, drag suppression, and state cleanup remain consistent.

Avoid carrying over:

- fixed internal frame width and height,
- Opal chrome theme coupling,
- presentation-heavy variants as the first API surface,
- app-specific floating/inset/icon styling,
- slot override mechanisms unless Luma needs them,
- treating split view as the whole layout system.

## Gallery Integration Plan

The first gallery use should prove the app shell workflow:

```text
root focus scope
  split view
    sidebar: gallery navigation
    content: active gallery page
```

The first gallery integration can use placeholder navigation if `NavigationSidebar` is not ready yet.
That keeps `SplitView` independent from the sidebar navigation effort.

Recommended sequence:

1. Add the SDK `split_view` module with model, control, template, and exports.
2. Implement width clamping and collapse state tests.
3. Implement the default two-pane template and draggable separator.
4. Add gallery shell usage with a simple sidebar.
5. Move current gallery demos into content pages.
6. Add `NavigationSidebar` after split view behavior is stable.

## Open Questions

- Should pane slots be stored as render closures, supplied through template render model data, or
  passed by the owning app during render?
- Should the first SDK version expose collapse controls, or should applications own explicit
  collapse buttons?
- Should separator focus and keyboard resizing ship in the first version or follow after pointer
  resizing?
- Should `SplitView` live directly under `controls/split_view` now, or should layout controls get a
  `controls/layout/` namespace once more than one layout control exists?

## Decision Bias

Prefer a small, useful control that makes the gallery better immediately.

Do not design the whole layout future upfront. Let the gallery, then later multi-pane examples,
force the next API decisions.
