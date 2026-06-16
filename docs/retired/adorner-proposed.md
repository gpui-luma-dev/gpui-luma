# Focus Adorner — Proposed Revision

## Intent

This proposal keeps the original adorner direction, but tightens the architecture around two explicit invariants:

1. **Decoration must not consume layout geometry** (no structural wrapper tax).
2. **All focus decoration colors must resolve from one active theme source** (no token-source drift).

The first is solved by adorners. The second must be enforced in theme resolution and is not automatically solved by adorner migration.

---

## Problem (refined)

The current SDK has two separate classes of issues:

### A) Geometry coupling (render strategy problem)

Focus visuals are currently implemented via multiple strategies (wrapper, absolute overlay, direct border mutation), which causes structural inconsistency and template complexity.

### B) Theme source divergence (token resolution problem)

Some control paths resolve focus ring from fallback/default token constructors, while others resolve from native TOML theme tokens. This can produce different focus colors for controls that should match.

> **Important:** Adorner migration solves (A), but not (B) unless theme-source unification is enforced.

---

## Proposed Architecture

## 1) Adorner model (layout-safe decoration)

Adorners are absolute decorative children attached to a relative control root. They do not alter padding/border/size of the control itself.

Root structure:

- `root` (relative, focus/event owner)
  - control visual content
  - zero or more decorative adorner children

Design rules:

- Adorners are decorative-only.
- Adorners do not own handlers.
- Control root owns id/focus/events.

## 2) Theme-source model (single source of truth)

All adorner colors and states must resolve from the currently active theme tokens only.

Design rules:

- Runtime UI must not mix fallback token defaults with native theme tokens.
- `focus.ring` must resolve through the same token source for all controls.
- Missing/invalid theme tokens should fail validation (load-time or test-time), not silently fall back to another palette.

---

## Adorner Primitive Contract

The shared adorner module should define:

- `FocusRingAdorner` (inset by default)
- `LeadingCaretAdorner`
- future: badges/underlines/selection marks

Each primitive should specify:

- Geometry parameters (`gap`, `width`, `radius`, `inset`)
- Paint color input
- Layer intent (`underlay` vs `overlay`)

### Layering convention

To avoid ordering bugs, define and document ordering:

1. underlay adorners (selection fills/caret backgrounds)
2. content
3. overlay adorners (focus rings/badges)

---

## Inset vs Outset Policy

Default policy:

- **Use inset adorners by default** for all controls.

Rationale:

- avoids clipping by `overflow_hidden` ancestors
- avoids negative-offset edge cases
- preserves layout invariants cleanly

Outset is opt-in only when a component explicitly guarantees unclipped ancestry.

---

## Geometry Note (focus ring math)

For inset ring placement, offset should be based on **gap**; border width then defines inward thickness naturally.

If ring border is offset by `gap + width`, it is typically over-inset by one border width.

So the default mental model should be:

- outer ring edge inset = `gap`
- ring thickness = `width`
- inner edge inset = `gap + width`

---

## Migration Plan (revised)

### Phase 0 — Guardrail first (theme-source unification)

Before/alongside adorner migration, enforce single token source for runtime defaults to prevent color drift during migration.

### Phase 1 — Shared primitives

Create `controls/adorner.rs` with focus ring + leading caret primitives and shared docs.

### Phase 2 — Button family

Migrate `Button`/`IconButton`/`ToggleButton`/`TextField`/`TextArea` from wrapper strategy to adorner injection.

### Phase 3 — Switch + Checkbox

Replace wrapper focus rings on indicator/track with adorner children on relative visual roots.

### Phase 4 — Border-mutation controls

Migrate `PopupMenu`, `ContextMenu`, `NavigationSidebar`, `Scrollbar` from direct border mutation to adorner overlays.

### Phase 5 — Normalize existing absolute overlays

Align `ToggleGroup` and similar patterns to shared adorner primitives.

---

## Acceptance Criteria

A migration is complete when all criteria pass:

1. Focus on/off does not change layout bounds.
2. No sibling jitter in flex/grid/flow containers.
3. No clipping regressions in overflow-hidden and scroll contexts.
4. Focus ring color parity across button/icon/toggle variants in same theme mode.
5. No runtime path resolves focus ring from a different token source.
6. Existing keyboard focus behavior and hit testing remain unchanged.

---

## Suggested File Impact

- `controls/adorner.rs` (new shared primitives)
- control templates currently doing wrapper or direct border mutation
- theme/default constructors to guarantee single runtime token source

(Exact file list can mirror `docs/adorner.md`, but should include explicit theme-source unification tasks.)

---

## Non-Goals

- Redesign of token schema.
- Introducing per-control bespoke adorner systems.
- Silent fallback to alternate palette values at runtime.

---

## Summary

Adorners are the correct rendering abstraction for focus and related decoration.

To prevent repeated inconsistency, pair adorner migration with a strict theme-source invariant: **one active token source for all runtime decoration resolution**.
