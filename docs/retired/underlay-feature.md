# Underlay Feature Prototype

## Purpose

This note outlines a prototype implementation of a new **`underlay`** decoration seam for controls, developed entirely within the gallery application.

The immediate goal is to experiment with a **gradient elevation treatment under a button**: a soft, floating pedestal that sits below the control surface.

By implementing this entirely as a prototype in the gallery app, we can iterate on the rendering primitives, directional layout padding, and interaction state transitions without introducing any changes to the SDK (`crates/sdk`) or stylesheet layers (`crates/look-shadcn`).

---

## Decision

Prototype the generic `underlay` channel as a custom `ButtonTemplate` in the gallery application.

### Why Prototyping is Isolated
* **No SDK/Look Changes**: We do not touch `crates/sdk` or `crates/look-shadcn` crates.
* **Safer Iteration**: Prototyping allows visual tuning of gradient dimensions, insets, and opacity stops before committing to structural changes in core library code.
* **Exemplar Context**: The gallery already features a dedicated `Prototypes` group suitable for hosting design experiments.

---

## Current State

### Gallery Prototyping Seams
* [registry.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/registry.rs)
* [proto_button/pane.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/prototypes/proto_button/pane.rs)
* [mod_button/pane.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/prototypes/mod_button/pane.rs)

The `Prototypes -> Decorated Button` page is the designated landing zone to test the underlay treatment.

---

## Non-Goals

This prototype pass does **not**:
* Add or modify any files in `crates/sdk` or `crates/look-shadcn`.
* Change any production stylesheet/configuration rules.
* Unify the SDK's `adorner` and `underlay` seams.
* Create a general luma-studio interface for underlays.

---

## Prototype Design & Shape

### Underlay Specification
The prototype will represent the underlay using a local struct inside the gallery:

```rust
pub struct PrototypeUnderlaySpec {
    pub color: Hsla,
    pub width_inset: f32,
    pub height: f32,
    pub y_offset: f32,
    pub radius: f32,
    pub opacity_start: f32,
    pub opacity_end: f32,
}
```

### Rendering Strategy
We will render the pedestal using GPUI's native `linear_gradient` background primitive (as seen in `apps/neumorphic-demo/src/background.rs`):
* An absolute-positioned `Div` underlay sits behind the button control.
* A vertical linear gradient (`180.0` degrees) transitions from `opacity_start` to `opacity_end` of the specified `color`.

---

## State Handling

The prototype underlay is designed to be fully interactive:

### Disabled State
* **Behavior**: The underlay is suppressed entirely (`None`) when `state.disabled` is true.
* **Rationale**: A disabled button should look completely flat and inactive. Dimming an elevated shadow leads to muddy visuals and visually contradicts its disabled state.

### Hovered and Pressed States
* **Behavior**: When pressed (`state.pressed`), the underlay adjusts dynamically:
  * Reduce `y_offset`
  * Reduce `opacity_start`
* **Rationale**: This simulates physical feedback, making the button visually "sink" closer to the pedestal as it is clicked.

---

## Template Composition

The prototype underlay is composed using a custom `ButtonTemplate` in the gallery pane. 

Desired composition order:
1. Resolve prototype colors and geometry.
2. Render the primary button control node.
3. Render the absolute-positioned underlay node.
4. Wrap both in a relative parent container (rendering the underlay behind the control).
5. Apply asymmetric directional padding to the parent (using `y_offset + height` on the bottom) to avoid vertical clipping without wasting horizontal layout space.

---

## Prototyping Steps

### Phase 1: Custom Template & Spec
* Define `PrototypeUnderlaySpec` in `proto_button/pane.rs`.
* Create `PrototypeUnderlayButtonTemplate` implementing `ButtonTemplate`.

### Phase 2: Stacking & Padding
* Assemble the underlay behind the control.
* Add bottom padding matching the vertical projection of the pedestal.

### Phase 3: Interaction Wiring
* Suppress the underlay when `state.disabled`.
* Animate/transition the offset and opacity when `state.pressed` is true.
* Add interactive sliders to the pane to dynamically tune parameters (height, inset, offset, color).

---

## Open Questions

1. Does the native `linear_gradient` provide a soft enough elevation read without needing blur filter effects?
2. What are the ideal defaults for `width_inset` and `y_offset` that make standard text buttons look clean?
3. How does the underlay interact visually when a button has a focus ring (`adorner`) active at the same time?

