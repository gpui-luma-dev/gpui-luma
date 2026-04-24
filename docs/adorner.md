# Focus Adorner

## Problem

The SDK currently draws focus rings using a "direct draw" scheme embedded inside
control templates. Three distinct strategies have emerged across the codebase:

| Strategy | Controls | Mechanism |
|---|---|---|
| **Wrapper div** | `Button`, `IconButton`, `ToggleButton`, `Switch`, `Checkbox`, `TextField`, `TextArea` | Wraps the control in a new `div` with `padding + border`, pushing the outer edge outward |
| **Absolute overlay** | `ToggleGroup` items | Adds an `absolute`-positioned child `div` inset by 1 px on all sides |
| **Direct border mutation** | `PopupMenu`, `ContextMenu`, `NavigationSidebar` rows, `Scrollbar` | Conditionally calls `.border_1().border_color(focus_ring)` on the control div |

### Core problem: geometry consumption (Wrapper div strategy)

`render_button_family_focus_ring` in `button_family_template.rs` wraps the
visual control in an outer div:

```rust
div()
    .p(px(FOCUS_RING_GAP))   // 1 px — ring gap (structural padding)
    .border_1()              // 1 px — ring itself
    .border_color(ring_color)
    .rounded(px(ring_radius))
    .child(control)
```

- The outer div receives `.id()`, `.track_focus()`, and all event handlers.
- The control is **permanently 2 px wider/taller** than its visual size, even
  when unfocused, because the gap padding is structural.
- Focused/unfocused transitions change border *color* only — size is constant —
  but the outer wrapper still imposes a fixed layout cost on surrounding elements.
- Templates are entangled with focus mechanics and cannot be simplified without
  rethinking the layout structure.

---

## Proposed Solution: Adorner

An **adorner** is a purely decorative, absolute-positioned overlay rendered as
a child of the control root. It draws on top of the control's own visuals
without participating in layout — consuming zero geometry.

This generalises beyond focus rings. Any decoration that would otherwise require
modifying the control's own border, padding, or size is a candidate for an
adorner. Concrete examples:

- **Focus ring** — border around the entire control, inset from the edge
- **Caret / accent bar** — thin vertical (or horizontal) bar pinned to one edge,
  indicating selection or current state (e.g. active nav item, active tab)
- **Badge / indicator dot** — small overlay in a corner

### Layout model

```
root (relative, Stateful<Div>)  ←  id · track_focus · event handlers
  ├─ control visuals (bg, border, padding, content)
  ├─ adorner₁ (absolute, no event handlers)  e.g. focus ring
  └─ adorner₂ (absolute, no event handlers)  e.g. leading-edge caret
```

GPUI renders children in painter's order, so adorners always draw on top.
Because they carry no event handlers they are fully transparent to hit testing.
Multiple adorners can coexist on the same root without interfering with each
other or with the control's geometry.

### Advantages over the current approach

- Control div size is **entirely decoupled from visual decoration state** — no
  jitter when focus or selection changes.
- Templates become simpler: the root IS the visual control; no wrapper is needed.
- The `Stateful<Div>` contract between templates and `control.rs` is preserved
  — `control.rs` still chains `.track_focus()` onto the returned value.
- Appearance structs are unchanged.
- Migration is incremental and per-control.
- New decoration types (caret, badge, …) require no structural changes to
  templates — just add another adorner child.

---

## Implementation Plan

### Phase 1 — Shared adorner primitives

Create `crates/sdk/src/controls/adorner.rs` with one function per adorner kind:

```rust
/// Inset focus ring — border drawn inside the control bounds.
/// The control's own background fills the gap, producing the visual
/// appearance of a ring surrounding the control.
/// Returns `None` when `color` is `None` (not focused).
pub fn render_focus_ring_adorner(color: Option<Hsla>, radius: f32, gap: f32, width: f32) -> Option<Div> {
    let color = color?;
    let inset = gap + width;

    Some(
        div()
            .absolute()
            .top(px(inset))
            .left(px(inset))
            .right(px(inset))
            .bottom(px(inset))
            .border(px(width))
            .border_color(color)
            .rounded(px((radius - inset).max(0.0))),
    )
}

/// Leading-edge caret — thin vertical accent bar pinned to the left side.
/// Returns `None` when `color` is `None` (not selected / not active).
pub fn render_leading_caret_adorner(color: Option<Hsla>, width: f32, inset: f32, radius: f32) -> Option<Div> {
    let color = color?;

    Some(
        div()
            .absolute()
            .left(px(0.0))
            .top(px(inset))
            .bottom(px(inset))
            .w(px(width))
            .bg(color)
            .rounded(px(radius)),
    )
}
```

Adorners are only added to the element tree when their `color` is `Some`,
eliminating the always-present transparent elements of the current scheme.

### Phase 2 — Migrate the button family

Replace `render_button_family_focus_ring` (wrapper pattern) with inline adorner
injection. Before:

```rust
// template.rs — outer wrapper holds id + focus ring
let control = div()...;  // visual only
let root = render_button_family_focus_ring(model.id.clone(), control, ...);
```

After:

```rust
// template.rs — visual control IS the root; adorner is a child
let adorner = render_focus_adorner(FocusAdornerStyle {
    color: appearance.focus_ring,
    radius: appearance.radius,
    gap: FOCUS_RING_GAP,
    width: FOCUS_RING_WIDTH,
});

let mut root = div()
    .id(model.id)
    .relative()
    // ... all visual styles ...
    .child(label);

if let Some(adorner) = adorner {
    root = root.child(adorner);
}
```

Controls affected: `Button`, `IconButton`, `ToggleButton`, `TextField`,
`TextArea`.

### Phase 3 — Migrate Switch and Checkbox

**Switch**: the focus ring currently wraps the `track` div. Replace with an
adorner child on the track (which is already `relative`).

**Checkbox**: the focus ring currently wraps only the indicator box. Replace
with an adorner child on the indicator div.

### Phase 4 — Migrate direct border mutation controls

`PopupMenu`, `ContextMenu`, `NavigationSidebar` rows, and `Scrollbar` currently
call `.border_color(focus_ring)` conditionally, overwriting the control's visual
border. Replace with an adorner child on the respective root div (which must be
marked `.relative()` if not already).

---

## Risk Assessment

### Inset vs. outset ring

The adorner can be placed in two ways:

**Inset** — the adorner sits inside the control bounds, offset inward from the
edge by the gap amount. The control's own background fills the gap between its
edge and the ring border, producing the visual appearance of a ring surrounding
the control. This is the preferred default: no negative offsets needed, no
clipping risk, and the visual result is identical to an outset ring from the
user's perspective.

```
┌─ control edge ──────────────────┐
│  gap (control bg shows through) │
│  ┌─ ring border ──────────────┐ │
│  │  control content          │ │
```

**Outset** — the adorner extends beyond the control bounds using negative `px()`
offsets (`top(px(-2.0))` etc.). GPUI supports negative absolute offsets, and the
element is positioned relative to its nearest `relative` ancestor. However, any
ancestor with `overflow_hidden` (e.g. `ToggleGroup`'s container, scroll
containers) will clip the ring.

> **Existing precedent**: `scrollbar/template.rs` already renders an inset
> adorner — `div().absolute().size_full().border_1().border_color(focus_ring)`
> — proving the GPUI absolute overlay pattern works in this codebase today.

**Convention**: use **inset** as the default for all controls. Reserve outset
only for controls guaranteed to render in unclipped contexts, and document that
choice at the call site.

---

## Files Affected

| File | Change |
|---|---|
| `controls/adorner.rs` | **[NEW]** Shared adorner primitives (`focus_ring`, `leading_caret`, …) |
| `controls/button_family_template.rs` | Replace `render_button_family_focus_ring` with adorner |
| `controls/button/template.rs` | Use focus ring adorner |
| `controls/icon_button/template.rs` | Use focus ring adorner |
| `controls/toggle_button/template.rs` | Use focus ring adorner |
| `controls/textfield/template.rs` | Use focus ring adorner |
| `controls/textarea/template.rs` | Use focus ring adorner |
| `controls/switch/template.rs` | Replace `render_switch_focus_ring` with adorner |
| `controls/checkbox/template.rs` | Replace `render_checkbox_focus_ring` with adorner |
| `controls/scrollbar/template.rs` | Replace direct border mutation with adorner |
| `controls/popup_menu/template.rs` | Replace direct border mutation with adorner |
| `controls/context_menu/template.rs` | Replace direct border mutation with adorner |
| `controls/navigation_sidebar/template.rs` | Replace direct border mutation + add leading caret adorner for active rows |
| `controls/toggle_group/template.rs` | Align existing absolute overlay to shared primitive |
| `controls/mod.rs` | Export `adorner` module |
