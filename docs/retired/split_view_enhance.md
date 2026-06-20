# Split View Enhancement Design

This document captures the feature set from the older Opal split-view work and describes how those
ideas should evolve into the Luma `SplitView`.

The current Luma implementation intentionally starts small: two panes, left sidebar, draggable
separator, collapse/expand, width constraints, and gallery shell integration. The features below are
future enhancements, not prerequisites for moving on to gallery navigation.

## Prior Art

Opal split-view source:

- `gpui-opal/crates/sdk/src/controls/layout/split_view`
- `gpui-opal/apps/gallery/src/gallery/pages/layout/split_view.rs`
- `gpui-opal/apps/gallery/src/gallery/pages/layout/split_view_floating.rs`
- `gpui-opal/apps/gallery/src/gallery/pages/layout/split_view_inset.rs`
- `gpui-opal/apps/gallery/src/gallery/pages/layout/split_view_icons.rs`
- `gpui-opal/apps/gallery/src/gallery/pages/layout/split_view_shared.rs`

Related Opal resizable source:

- `gpui-opal/crates/sdk/src/controls/layout/resizable`
- `gpui-opal/apps/gallery/src/gallery/pages/layout/resizable.rs`

Treat these files as prior art. Luma should reuse behavior lessons and test cases, not port Opal's
theme coupling or fixed demo frame model directly.

## Current Luma Baseline

The first Luma `SplitView` supports:

- sidebar/content panes,
- left sidebar placement,
- parent-supplied pane content,
- configurable sidebar width,
- min/max sidebar width clamping,
- configurable collapsed width,
- click-to-collapse and click-to-expand,
- pointer resize through a separator,
- drag click suppression,
- semantic events for resize and collapse,
- default template with wide separator hit lane and narrow cue.

This is enough for the first gallery shell.

## Feature Inventory From Opal

### Unified Split View

Opal's `common` variant demonstrated a plain shared-frame split view: sidebar, separator, content.

Luma path:

- Keep this as the default `SplitView` behavior.
- Do not expose a `Unified` variant unless multiple templates need to distinguish it explicitly.
- Add gallery coverage once content pages exist.

### Detached Or Floating Split View

Opal's `floating` variant rendered sidebar and content as independent surfaces separated by a gap.
The gallery demo also included a content-pane icon button that toggled sidebar collapse.

Luma path:

- Prefer implementing this as a custom template or app composition first.
- Keep `toggle_collapsed` as the single behavior path for external buttons.
- Add a gallery demo with a toolbar collapse button before deciding whether this belongs in the SDK
  default template.

Potential API:

```rust
SplitView::new("detached-shell")
    .template(detached_split_view_template())
```

Avoid adding a `SplitViewVariant::Detached` enum until there is a concrete reason that template
selection cannot cover the use case.

### Layered Inset Split View

Opal's `inset` variant placed nav and content inside a shared dark surface with the content pane
inset as its own rounded panel.

Luma path:

- Treat this as a visual recipe.
- Let the parent shell provide outer padding and background.
- Use a custom template only if separator geometry needs to know about the inset.

Potential API:

```rust
SplitView::new("inset-shell")
    .template(inset_split_view_template())
```

The first Luma implementation should continue to avoid owning fixed frame width and height.

### Icon Rail

Opal's `icons` variant collapsed the sidebar to a nonzero width and masked text, leaving an icon rail
visible.

Luma path:

- This is the most natural next enhancement because Luma already has `sidebar_collapsed_width`.
- Add a gallery demo using `sidebar_collapsed_width(px(56.0))`.
- Make sidebar content responsible for rendering acceptably at narrow width.
- Later, consider adding a formal compact/collapsed render slot if repeated consumers need one.

Potential API:

```rust
SplitView::new("gallery-shell")
    .sidebar_collapsed_width(px(56.0))
```

Possible later API:

```rust
SplitView::new("gallery-shell")
    .sidebar(expanded_sidebar)
    .collapsed_sidebar(icon_rail_sidebar)
```

Do not add the second slot until app composition proves the need.

### Runtime Separator Visibility

Opal demos exposed a switch between `Always` and `Hover` separator visibility.

Luma already has `SplitViewSeparatorVisibility` in the model. It needs a mutation method.

Recommended API:

```rust
pub fn set_separator_visibility(
    &mut self,
    visibility: SplitViewSeparatorVisibility,
    cx: &mut Context<Self>,
)
```

This is low-risk and should be one of the first enhancements.

**Status:** implemented.

### Theme Integration

Split view separator colors must resolve from a live theme at render time, not from snapshot `Hsla`
values captured at construction.

Luma path (implemented):

- `SplitViewTheme` trait with `resolve(hovered, enabled) -> SplitViewLook`
- `DefaultSplitViewTheme` for token-based apps
- `SplitViewBuilder::theme(...)` and `SplitView::set_theme(...)`
- Optional `.separator_color()` / `.separator_hover_color()` overrides for tests or fixed demos
- `gpui_luma_look_shadcn::ShadcnLook::split_view_theme()` and `.split_view(id)` factories

Gallery and theme-studio shell split views should use the Shadcn theme factory so light/dark
toggles update separator cues without rebuilding the control.

### External Collapse Actions

Opal showed collapse being triggered from content controls, not only from the separator.

Luma already supports this through:

```rust
split_view.update(cx, |split_view, cx| {
    split_view.toggle_collapsed(cx);
});
```

Design rule:

- All collapse and expand paths must route through `set_collapsed` or `toggle_collapsed`.
- Separator clicks, toolbar buttons, menu commands, and future keyboard actions should share the
  same state transition and event emission.

### Right Placement

Opal supported left and right nav pane placement.

Luma path:

- Add only after the gallery or another consumer needs it.
- The control logic should sign resize deltas based on placement.
- The template should mirror sidebar/content order and collapsed expand hit lane placement.

Potential API:

```rust
pub enum SplitViewSidebarPlacement {
    Left,
    Right,
}

SplitView::new("inspector-shell")
    .sidebar_placement(SplitViewSidebarPlacement::Right)
```

### Resize Lifecycle Events

Opal's generic resizable control had `ResizeStart`, `Change`, and `ResizeEnd`.

Luma currently has:

- `ResizeStart`,
- `SidebarWidthChanged`,
- `ResizeEnd`,
- `CollapsedChanged`.

Enhancement notes:

- Keep semantic split-view names rather than generic panel names.
- Emit `ResizeStart` only when movement passes the click-suppression threshold.
- Emit `ResizeEnd` only after an actual resize.
- Add tests for these event rules if GPUI entity tests become practical.

### Controlled Width

Opal resizable demos showed programmatic reset and controlled panel sizes.

Luma path:

- `set_sidebar_width` already supports programmatic width changes.
- Add gallery controls later: reset width, collapse, expand, and maybe narrow/wide presets.
- Do not add persistence helpers until the app has a concrete storage location.

Possible future helpers:

```rust
split_view.set_sidebar_width(px(280.0), cx);
split_view.set_collapsed(false, cx);
```

### Nested Resizable Layouts

Opal's generic `Resizable` supported nested horizontal/vertical panel groups.

Luma path:

- Do not fold arbitrary nested layouts into `SplitView`.
- If needed later, add a separate `ResizablePanels` or `PanelGroup` control.
- Reuse Opal's pair-resize min/max math there, not in navigation split view unless three-pane
  support requires it.

### Three-Pane Navigation

The original product direction mentions moving from two panes to two-plus panes. Opal did not have a
clean SwiftUI-like `NavigationSplitView` three-column API, but its resize math can inform one.

Luma path:

- Add only after the gallery has a real need for sidebar/content/detail.
- Keep the first extension explicit rather than arbitrary:
  sidebar, content, detail.
- Use separate widths and collapse rules for sidebar and detail.

Potential API:

```rust
SplitView::new("gallery-shell")
    .sidebar(sidebar)
    .content(content)
    .detail(detail)
```

Open questions:

- Can detail collapse independently?
- Does content remain flexible between sidebar and detail?
- Are both separators draggable?
- Does narrow viewport collapse sidebar first, detail first, or both?

## Suggested Enhancement Order

1. ~~Add `set_separator_visibility`.~~ Done.
2. ~~Add theme integration (`SplitViewTheme`, Shadcn factory, gallery wiring).~~ Done.
3. Add a gallery action button that calls `toggle_collapsed`.
4. Add icon-rail gallery coverage using nonzero `sidebar_collapsed_width`.
5. Add telemetry for effective width, collapsed width, and last event.
6. Add a custom detached/floating template if the gallery wants that visual treatment.
7. Add right placement only when an inspector/details layout needs it.
8. Add three-pane support after the gallery has real page/detail content.
9. Consider a separate resizable panel group control for arbitrary nested layouts.

## Design Guardrails

- Keep `SplitView` focused on navigation/content shell layouts.
- Avoid turning it into a full layout manager.
- Keep behavior in `control.rs`, configuration in `model.rs`, and visual structure in `template.rs`.
- Prefer template customization over enum variants for visual recipes.
- Preserve external action support by keeping mutation methods public and deterministic.
- Do not reintroduce fixed internal frame dimensions.
- Keep collapsed state separate from last expanded width.
- Keep click suppression around separator drag.

## Gallery Coverage Target

Eventually the Luma gallery should demonstrate:

- default unified split view,
- hover vs always separator visibility,
- external collapse button,
- zero-width collapsed sidebar,
- icon-rail collapsed sidebar,
- programmatic width reset,
- a detached visual template,
- an inset visual template,
- later right placement,
- later three-pane layout.

These examples should be added incrementally as the gallery registry and navigation system mature.
