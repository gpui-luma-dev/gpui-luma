# ChoiceGroup Reconciliation (`choice_groups_recon.md`)

## Purpose

This document reconciles the current `ChoiceGroup` implementation with the original intent in `docs/choice_groups.md`, identifies the key architectural mistakes, and proposes a clean replacement plan.

The core conclusion: **the current implementation is not a stable foundation for the intended control model**. We should stop incremental patching and rebuild `ChoiceGroup` as a clean, semantically-owned control while keeping legacy controls for side-by-side comparison only.

---

## What went wrong

## 1) Mixed-control architecture in call sites
We ended up with mixed usage in the same pane (new `ChoiceGroup` + old `radio_group`) instead of a clean replacement slice.  
This created:
- inconsistent semantics in one screen
- unclear ownership of behavior contracts
- difficult review and comparison

**Impact:** even if pieces work, the architecture is hard to reason about and harder to validate.

## 2) Theme/type coupling to legacy control
`ChoiceGroup` template currently relies on `ToggleGroup*` appearance/theme types (e.g. `ToggleGroupItemAppearance`).  
This violates the requirement that `ChoiceGroup` be generic (within bounds) and own its own visual semantics.

**Impact:** `ChoiceGroup` is effectively a thin wrapper over legacy visual primitives, not a true replacement abstraction.

## 3) Heuristic icon mode instead of explicit semantics
Icon/toolbar visuals are inferred from shape-like conditions (e.g. variant/layout heuristics) instead of explicit semantic role/preset state.

**Impact:** fragile behavior; impossible to guarantee visual contract consistency.

## 4) Ghost selected-state contract is not enforced
Original requirement: selected must be obvious in ghost/icon toolbar mode; hover/pressed must compose on top of selected baseline.  
Current layering makes this unreliable.

**Impact:** selected state is hard to distinguish and can be visually overridden.

## 5) API drift and partial pattern adoption
We introduced useful APIs (`template_factory`, builder modifiers), but without a coherent semantic model beneath them.

**Impact:** API surface appears advanced, but behavior remains inconsistent.

---

## Non-negotiable requirements (restated)

1. `ChoiceGroup` must be a standalone control architecture (not visually typed to toggle/radio internals).
2. Ghost/icon toolbar mode must make selected state persistently obvious.
3. `selected` (persistent) and `active/focus` (interaction) must be separate signals.
4. Managed/unmanaged contract must be explicit and testable.
5. Legacy controls remain for comparison, not as hidden dependencies.

---

## Decision: Do not salvage incrementally

We should **freeze the current `ChoiceGroup` implementation as failed prototype** and build a clean replacement path.  
Patch-by-patch salvage will keep leaking old assumptions and produce more mixed semantics.

---

## Clean replacement plan (V2)

## Phase A — Reset architecture boundaries
- Keep current legacy controls untouched (`radio_group`, `toggle_group`).
- Keep current prototype code for reference, but treat as non-authoritative.
- Define `ChoiceGroup` as a clean ownership boundary:
  - model semantics
  - render model
  - template theme contract
  - event contract

## Phase B — Define ChoiceGroup-owned theme contract
Create new theme primitives:
- `ChoiceGroupTheme`
- `ChoiceGroupListAppearance`
- `ChoiceGroupItemAppearance`

No `ToggleGroup*` type references in `choice_group` module.

Define explicit semantic role resolution:
- e.g. `ChoiceGroupRole::ToolbarIconItem { selected }`
- selected baseline first, interaction overlays second.

## Phase C — Rebuild template pipeline semantically
Template render contract should include:
- persistent state: `selected`
- interaction state: `hovered`, `pressed`, `active`, `focus_visible`
- role/preset identity (toolbar icons, segmented text, etc.)

State layering order:
1. base appearance (role + selected)
2. hover/pressed overlays
3. focus ring overlay

This directly satisfies the ghost/icon selected-visibility requirement.

## Phase D — Rebuild control behavior core
Rebuild logic with explicit state mode:
- `Unmanaged` uses internal selection source
- `Managed` uses external source of truth
- `.selected(...)` remains initial/fallback only under managed mode
- user interactions always emit `Change` event payload
- parent decides commit

## Phase E — Wrapper presets after foundation is stable
Add wrapper APIs only after semantic core is correct:
- `toolbar_icons(...)`
- `toolbar_icons_multiple(...)`
- optional compact presets

Wrappers should only apply defaults; caller overrides remain last-wins.

## Phase F — Clean call-site migration slice
In `IntroductionPane`, create one fully coherent `ChoiceGroup` path (no mixing with legacy controls inside that slice).  
Keep old controls elsewhere for comparison panes only.

---

## Proposed acceptance criteria for V2

1. `choice_group` module has no direct dependency on `toggle_group` theme/appearance types.
2. Toolbar ghost selected state remains visually obvious when unfocused.
3. Hover/pressed visually compose on selected baseline.
4. Focus ring shape always matches icon-button shape in toolbar mode.
5. Managed state always wins render; `.selected(...)` works as fallback/default.
6. `IntroductionPane` replacement slice uses only `ChoiceGroup` for its choice semantics.
7. Legacy controls still compile and remain available for side-by-side comparison.
8. Unit tests cover:
   - selection transitions
   - managed precedence
   - event payload integrity
   - selected vs interaction layering hooks

---

## Immediate implementation notes

- Keep the existing API improvements (`template_factory`, builder modifiers) if useful, but re-bind them to V2 semantics.
- Remove any “icon mode by heuristic” behavior; use explicit preset/role flags in render model.
- Avoid introducing additional surface area until the visual contract and state model are stable.

---

## Risks and mitigations

## Risk: V2 rebuild takes longer than patching
Mitigation: strict phase gates and minimal initial scope (toolbar icon mode first).

## Risk: regressions in intro/gallery demos
Mitigation: side-by-side validation pane; keep legacy controls as reference baseline.

## Risk: design drift from original contract
Mitigation: enforce acceptance criteria above before broader migration.

---

## Summary

The current prototype demonstrated useful API directions but failed the core architectural requirement: a truly generic, semantically-owned `ChoiceGroup`.  
The correct path is a **clean V2 replacement**, not continued incremental salvage.