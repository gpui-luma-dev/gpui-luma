# Adorner Architecture

## Status

**Focus-ring migration complete.** The shared focus-ring adorner infrastructure is now
the only focus-decoration contract exposed by the in-scope SDK look types. Additional
adorner kinds are follow-up work.

Implemented today:

- `theme/adorner.rs` defines `AdornerSpec::FocusRing` with inset and oversize placement.
- Command buttons render a relative host, visual control, and decorative adorner child.
- Checkbox, Radio Button, and Switch themes resolve `Option<AdornerSpec>` and their
  templates use the shared renderer.
- ListView and ListBox rows, plus the supporting TreeView and Accordion look contracts,
  have adorner-capable surfaces.
- Oversize extent and shadow projection are handled by shared choice/button-family
  layout helpers where required to prevent clipping and focus-toggle jitter.

## Problem Summary

Controls historically rendered focus visuals through mixed strategies:

1. Wrapper elements that add padding and a border
2. Absolute overlays implemented independently by each control
3. Direct border mutation on the visual control

These approaches made focus styling inconsistent and could change a control's measured
footprint or clip an oversize ring.

## Current Architecture

### 1. Theme owns decoration intent

Theme resolution should return typed decoration intent (`Option<AdornerSpec>` while the
current system supports one adorner per look). Templates should not decide focus-ring
color, width, distance, or placement.

The current shared spec is:

- `AdornerSpec::FocusRing(FocusRingAdornerSpec)`
- `AdornerPlacement::Inset`
- `AdornerPlacement::Oversize`

### 2. Template owns structure and event ownership

Templates own the host/visual hierarchy and render the adorner as a decorative child.
The host/control root owns `id`, focus tracking, hit testing, and handlers. Adorners must
not receive focus or install event handlers.

### 3. Geometry policy is explicit

Inset adorners paint inside the host bounds and do not require footprint reservation.
Oversize adorners paint outside the visual bounds and may be clipped by an ancestor with
`overflow_hidden`.

Some current templates reserve the maximum oversize extent (including a focused-state
probe) in a stable slot. This is an intentional footprint reservation used to prevent
clipping and sibling jitter; it is not a claim that oversize paint is layout-free. The
required invariant is that toggling focus does not change measured bounds.

## Button-Family Policy

The current button-family renderer derives a focus-ring spec from the resolved border,
focus color, and metric tokens:

- Borderless buttons use an inset ring.
- Bordered buttons use an oversize ring.
- Icon-role buttons remain focusable and receive the same focus treatment.

Theme adapters construct typed adorner specs directly from resolved colors and metrics;
no production template uses the old wrapper or mutates a focus border.

## Completed Closure Work

### 1. Lock the contract and add regression coverage

- Keep one typed optional focus-ring spec per look until multiple adorner kinds are needed.
- Define which controls reserve oversize extent and ensure the reservation is stable with
  and without focus.
- Existing geometry/layout tests cover button-family extent, shadow reservation, and
  slider geometry; SDK and Shadcn test suites pass.

### 2. Finish wrapper migration

- Converted TextField, TextArea, and Search Selector to relative hosts plus shared
  adorner children.
- Preserve input IDs, focus behavior, selection/caret rendering, shadows, and modifiers.
- Removed `render_button_family_focus_ring` after its final production consumer migrated.

### 3. Finish border-mutation migration

Converted these controls to shared adorner rendering:

- Popup Menu trigger
- Context Menu target
- Selector trigger
- Navigation Sidebar rows
- Scrollbar focus target

Each migrated path keeps focus and pointer ownership on the control host.

### 4. Normalize composite and thumb focus

- Converted Slider thumb focus and ControlGroup/ToggleGroup item focus to the shared
  adorner renderer.
- Audit Tabs Navigation, Search Selector, and other controls that reuse button-family
  visuals so they do not reintroduce wrapper or bespoke border logic.

### 5. Remove drift and verify

- Removed obsolete bespoke focus-ring helpers and compatibility color fields. Inspector
  metadata now points at the typed `adorner` field.
- Update look usage metadata and inspectors to report the adorner source consistently.
- Ran `cargo fmt`, workspace `cargo check`, workspace Clippy with `-D warnings`, and
  SDK/Shadcn tests: 351 SDK tests and 153 Shadcn tests passed.

## Acceptance Criteria

1. Focus on/off does not change measured bounds for every migrated control.
2. No sibling jitter occurs in flow, flex, or grid layouts.
3. Keyboard focus, pointer hit testing, and event behavior remain unchanged.
4. Focus visuals come from typed, theme-resolved adorner specs.
5. Migrated templates contain no focus-specific border mutation or wrapper-only focus
   ring logic.
6. Runtime theme resolution uses one token source for focus color and metrics.
7. Ancestor clipping behavior and any intentional oversize footprint reservation are
   documented and covered by tests or visual verification.

The issue is ready to close after visual QA in Gallery/Luma Studio confirms keyboard
focus, mouse hit testing, disabled states, oversize clipping, and flow/flex/grid sibling
stability.

## Out of Scope / Follow-Up

- Caret, badge, underline, validation, and selection-mark adorner kinds.
- Multi-adorner ordering metadata (`underlay` versus `overlay`).
- Rich adorner policy in the theme TOML schema.
- A general `not_focusable` control option.
