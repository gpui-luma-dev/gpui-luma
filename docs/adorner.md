# Adorner Architecture

## Status

This document reflects the **current direction and partial implementation** of the adorner system.

Implemented so far:

- Button family templates use a **host + visual control** structure.
- Focus decoration for button family is now resolved as **theme-driven adorner specs**.
- Adorner rendering primitives live under `theme/adorner.rs`.
- Button family appearance now carries `adorners` instead of a single `focus_ring` color.

Still in progress:

- Migration of non-button controls from wrapper/border-mutation patterns.
- Broader adorner kinds (caret/badge/etc.) beyond focus ring.

---

## Problem Summary

Historically, controls rendered focus visuals through mixed strategies:

1. Wrapper div that adds structural padding/border
2. Absolute overlay ad-hoc per control
3. Direct border mutation on control visuals

The wrapper strategy caused the main geometry issue: decoration consumed layout space.

---

## Current Architecture

## 1) Theme decides decoration intent

Theme resolution returns decoration intent as a list of adorner specs on appearance types.

For button family this is now:

- `ButtonFamilyAppearance.adorners: Vec<AdornerSpec>`

This allows theme policy to vary by control state/variant/role without template branching on decoration semantics.

## 2) Template owns structure only

Template builds:

- host root (`relative`, event/focus owner)
- visual control child
- rendered adorner children

The host layer prevents visual-control clipping from constraining oversize adorners.

## 3) Shared renderer paints specs

`theme/adorner.rs` contains adorner rendering primitives and an adapter that maps `AdornerSpec` to concrete `Div` overlays.

Current spec support:

- `AdornerSpec::FocusRing(FocusRingAdornerSpec)`

Current placement support:

- `AdornerPlacement::Inset`
- `AdornerPlacement::Oversize`

---

## Button Family Policy (current)

In `DefaultButtonFamilyTheme::resolve(...)`, focused state produces a focus-ring adorner spec.

Policy is variant-driven:

- `Ghost` → inset focus ring
- `Standard` / `Prominent` → oversize focus ring

Icon-role buttons are still focusable controls and currently receive focus adorners under this policy.

---

## Design Rules

1. Decoration must not consume layout geometry.
2. Template code should not hardcode decoration policy values.
3. Theme is the source of truth for adorner policy.
4. Adorners are decorative-only (no handlers, no focus ownership).
5. Control root/host owns `id`, focus tracking, and handlers.

---

## Geometry Notes

Inset focus ring:

- inset distance is based on `gap` (or generic distance)
- border width draws inward from that edge

Oversize focus ring:

- uses negative offsets from host bounds
- can be clipped by ancestor overflow constraints

Oversize should only be used where host/ancestor layout allows it.

---

## Migration Plan

### Phase 1 (done)

- Theme adorner module introduced.
- Button family switched to theme-driven `adorners` list.
- Button template switched to host + rendered adorner children.

### Phase 2

Migrate wrapper-based controls to adorner specs:

- `TextField`
- `TextArea`
- `Switch`
- `Checkbox`
- `RadioButton`

### Phase 3

Migrate border-mutation controls to adorner specs:

- `PopupMenu`
- `ContextMenu`
- `NavigationSidebar`
- `Scrollbar`

### Phase 4

Normalize existing overlay controls to shared adorner specs:

- `ChoiceGroup` / `ToggleGroup`
- `Slider` (thumb focus)

---

## Acceptance Criteria

1. Focus on/off does not change layout bounds.
2. No sibling jitter in flow/flex/grid containers.
3. Focus behavior remains accessible (keyboard + hit testing unchanged).
4. Focus visuals are theme-driven (variant/role/state aware).
5. No mixed token-source drift in runtime theme resolution paths.

---

## Future Considerations (not part of current change)

- Add more adorner kinds (leading caret, badge, underline, selection marks).
- Layer intent metadata (`underlay` vs `overlay`) for deterministic ordering.
- Add optional control-level ergonomics such as a `not_focusable` mode where appropriate.
- Expose richer adorner policy in theme TOML schema once cross-control model stabilizes.
