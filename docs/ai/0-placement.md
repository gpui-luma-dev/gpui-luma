# Palette Popover Placement

## Problem

The Luma Studio Palette displays many editable color swatches in a wrapped,
vertically scrollable layout. Clicking a swatch should open the color picker
immediately below the clicked trigger, or above it when there is not enough
room below.

The observed failure was a picker appearing well below the `Foreground`
swatch instead of directly below it. The picker was horizontally close to the
trigger, but its vertical position included an additional layout/deferred
offset. This made the popup appear disconnected from the swatch and caused
placement to vary with the palette layout and scroll position.

## What the current implementation does

`ColorPickerPopover` owns a `Button<Hsla>` and a separate `AnchoredPanel`
entity. Its render path is effectively:

```text
ColorPickerPopover
  └─ wrapper
      ├─ trigger wrapper
      │   └─ Button<Hsla>
      └─ AnchoredPanel
          └─ deferred(anchored(color picker content))
```

The trigger wrapper calls `set_anchor_bounds` during prepaint. The bounds are
then stored in the separate `AnchoredPanel` entity. During a later render, the
panel resolves `SmartStart` and renders a deferred `anchored()` element.

The panel is also nested under the Palette's `overflow_y_scroll()` content.
This combines several separate concerns:

1. The trigger is measured in the scrollable palette.
2. The bounds are pushed to a different entity during prepaint.
3. The panel resolves placement using state that may be from the previous
   render pass.
4. The actual overlay is deferred and positioned in window coordinates.
5. The overlay is still composed inside the scrollable control hierarchy.

The result is not a simple “trigger bottom plus four pixels” relationship,
even though that is what `SmartStart` intends to calculate.

Removing `.relative()` from the wrapper does not solve this by itself. That
modifier changes the local CSS positioning context; it does not establish a
coordinate-space contract between the scrollable trigger, the separate panel
entity, and the deferred overlay.

Unique element IDs are still required for every palette picker. Duplicate
trigger or panel IDs can make GPUI associate bounds with the wrong swatch, but
the remaining placement problem exists even when all IDs are unique.

## Why `AnchoredPanel` is the wrong abstraction here

The SDK `AnchoredPanel` API is caller-measured: the caller must discover the
anchor bounds and call `set_anchor_bounds`. That can be useful for an advanced
integration with a stable, externally measured anchor, but it is a poor fit
for a reusable trigger-owned color picker.

The API does not enforce the important invariants:

- bounds must be final physical/window-space bounds;
- bounds must be measured at the right point in the render lifecycle;
- the panel and trigger must remain synchronized through scrolling;
- deferred rendering must not apply the trigger's parent offset twice;
- the panel should be placed from the exact trigger element, not an
  intermediate wrapper.

The code compiles when any of these assumptions is violated, and the result
can look almost correct in a simple layout. That makes the abstraction easy
to misuse and difficult to debug in a wrapped scrollable grid.

`anchored()` is not inherently broken. It correctly supports a window-space
position and standard below/above placement. The weakness is the current
`AnchoredPanel` ownership model: measurement belongs to one entity while
placement and deferred rendering belong to another.

## Recommended color swatch and picker fix

The color picker should use the same architecture as the working popup-button
control:

```text
ColorPickerButton / PopupButton
  ├─ trigger presenter: direct ColorSwatch button face
  └─ deferred anchored popup content
```

The control must own all of the following in one lifecycle:

- the swatch trigger;
- the trigger's physical bounds;
- open/close state;
- placement resolution;
- deferred popup content.

The swatch should remain a specialized button template that paints the
checkerboard, color fill, border, hover/pressed state, and focus state directly
on the button surface. It should not contain a second generic swatch element
that adds another border or layout box.

The popup-capable button should measure the actual trigger child using the
popup-button pattern:

```rust
let mut root = div()
    .on_children_prepainted(move |bounds, window, cx| {
        if let Some(trigger_bounds) = bounds.first() {
            set_trigger_bounds(*trigger_bounds, window, cx);
        }
    })
    .relative()
    .child(trigger);
```

It should then resolve and render the popup from the same stored bounds:

```rust
let placement = resolve_popup_placement(
    trigger_bounds,
    PopupPlacement::Smart,
    window.viewport_size(),
);

let overlay = anchored()
    .snap_to_window_with_margin(px(8.0))
    .anchor(placement.anchor)
    .position(placement.position)
    .offset(placement.offset)
    .child(popup_content);

root.child(deferred(overlay).with_priority(1))
```

For the palette picker, `Smart` means:

```text
if trigger.bottom + popup_height fits in viewport:
    anchor popup top-left to trigger.bottom-left
else:
    anchor popup bottom-left to trigger.top-left
```

The color-picker-specific version can be implemented in either of two ways:

1. Add a reusable SDK `PopupButton`/`PopoverButton` control that accepts a
   trigger presenter and arbitrary popup content. Use the existing popup-menu
   lifecycle and bounds handling, but do not force color-picker content into a
   menu-item model.
2. Keep the implementation in Luma Studio temporarily, but copy the
   popup-button ownership pattern into `ColorPickerPopover` rather than
   embedding a separate `AnchoredPanel` entity.

Option 1 is the preferred long-term fix because it prevents every future
custom popover from rebuilding the same bounds, lifecycle, and deferred
overlay logic.

## Existing popup-button model to leverage

The SDK popup menu already demonstrates the safe mechanics:

- it measures the actual trigger child with `on_children_prepainted`;
- it stores `trigger_bounds` on the popup control itself;
- it resolves placement from those stored bounds;
- it renders the deferred overlay from the same root as the trigger;
- it supports smart below/above placement and window snapping.

The color picker should reuse these mechanics, not necessarily the menu's
visual or item API. A custom trigger presenter is already a supported concept
in the popup menu model, so a generic popup-button abstraction can preserve the
direct canvas swatch while using the proven popup lifecycle.

## Future options for `AnchoredPanel`

There are three reasonable SDK directions.

### Option A: Make `AnchoredPanel` trigger-owned

Redesign it as a true popover control that accepts a trigger element or
presenter and arbitrary content. It measures the trigger internally and owns
the overlay lifecycle. This makes the safe path the default and is the best
general-purpose API.

### Option B: Keep the low-level panel, add a safe wrapper

Keep `AnchoredPanel` for advanced callers that already have stable physical
bounds. Add `PopoverButton`/`PopupButton` for normal trigger-driven use. Mark
`set_anchor_bounds` as an advanced integration point in its documentation and
avoid using it for ordinary controls.

### Option C: Improve the current bounds API

Keep the separate panel entity but provide an explicit anchor callback or
anchor handle that the SDK measures internally. The API would need to define
that bounds are window-space, update them after scrolling, and coordinate the
panel render with the trigger render. This is possible, but it retains more
cross-entity lifecycle complexity than Option A or B.

## Decision

For the Palette color picker, use the popup-button architecture with the
direct-canvas `ColorSwatchButtonTemplate` as the trigger face and `SmartStart`
placement for the arbitrary color-picker content. Do not use
`AnchoredPanel::set_anchor_bounds` from a nested swatch wrapper.

Keep `AnchoredPanel` available for advanced, caller-owned anchors, but do not
treat it as the default solution for trigger-owned popovers until its ownership
and coordinate-space contract is redesigned.

## Verification checklist

- Click the first-row `Foreground` swatch: the picker opens directly below it.
- Scroll the palette and click a swatch in a later row: the picker follows that
  swatch's current screen position.
- Move between columns and wrapped rows: no stale or cross-row anchor is
  used.
- Place a swatch near the bottom of the viewport: the picker opens above it.
- Open and close repeatedly: no one-frame jump caused by stale bounds.
- Confirm every picker trigger and popup has a unique element ID.
- Verify light/dark themes and translucent swatches retain the direct-canvas
  checkerboard and theme-driven button states.
