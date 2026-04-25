# Proto2: Next-Steps Plan (State-Aware Template Parameterization)

## Status

- **Type:** Implementation plan
- **Predecessor:** `docs/proto1.md`
- **Scope:** Keep changes localized to prototype paths until promotion criteria are met

---

## Objective

Advance the prototype from “single-value override + metadata” to a robust, scalable model with:

1. **State-specific parameterization**
2. **Explicit override semantics**
3. **Tooling-ready metadata + two-way editor behavior**
4. **Tested precedence guarantees**

---

## Inputs from Proto1

Proto1 validated:

- localized prototype architecture
- runtime parameter read/write
- metadata registry pattern
- gallery-based interactive proof

Proto1 identified key gaps:

- no first-class per-state value model
- unclear inherit/set/clear semantics for nullable fields
- limited effective-value introspection
- limited merge/precedence test coverage

---

## Proto2 Success Criteria

Proto2 is complete when all are true:

1. Visual params support state-aware overrides (`default`, `hovered`, `pressed`, `focused`, `disabled`)
2. Nullable params support explicit **inherit / set / clear** semantics
3. Effective values are inspectable at runtime for a selected state
4. Metadata reflects state support and value semantics
5. Precedence behavior is covered by tests and documented
6. All changes remain localized to prototype paths

---

## Workstreams

## 1) State-Specific Parameter Model

### Goal
Represent visual overrides per interaction state without exploding flat fields.

### Deliverables
- Add a reusable stateful wrapper for prototype parameters:
  - base override
  - per-state override map/fields
- Apply to visual fields first:
  - background
  - foreground
  - border
  - focus ring

### Acceptance
- Any visual field can be overridden for specific state(s)
- Default behavior remains theme-derived when no override is present

---

## 2) Override Semantics (Inherit / Set / Clear)

### Goal
Eliminate ambiguity for nullable values (e.g. focus ring).

### Deliverables
- Introduce explicit override value mode:
  - `Inherit`
  - `Set(T)`
  - `Clear` (for nullable destinations)
- Use mode consistently across applicable fields

### Acceptance
- Focus ring behavior is deterministic in every state
- Semantics are encoded in metadata and docs

---

## 3) Deterministic Resolution Pipeline

### Goal
Formalize and enforce precedence.

### Required precedence
1. State-specific override
2. Base override
3. Theme-resolved value

### Deliverables
- Centralized resolve path in prototype template (`ThemedProtoButtonTemplate::resolve_appearance`)
- Short spec section documenting exact merge behavior

### ProtoButton precedence spec (Proto2)
For each render pass:

1. Resolve theme appearance using:
   - `variant`
   - `role = ButtonFamilyRole::Text`
   - `size`
   - runtime interaction state
2. Resolve visual state from interaction state with fixed priority:
   - `disabled > pressed > hovered > focused > default`
3. Apply color-like stateful overrides with precedence:
   - state-specific override
   - base override
   - theme value
4. Apply nullable override semantics (focus ring):
   - `Inherit` => keep theme value
   - `Set(T)` => force value `T`
   - `Clear` => force `None`
5. Apply non-stateful structural overrides (`radius`, padding, gap, height, typography) last.
6. Apply behavioral flags during render (`disabled_opacity`, pointer cursor policy).

This order is deterministic and side-effect free: same inputs must yield the same appearance.

### Acceptance
- Same inputs always produce same output
- Covered by unit tests (state > base > theme fallback and inherit/set/clear semantics)

---

## 4) Metadata v2 for Template Params

### Goal
Make metadata sufficient for editor generation and diagnostics.

### Deliverables
Extend metadata entries with:
- state applicability
- value kind (color, pixels, number, bool, enum)
- override semantics support (inherit/set/clear)
- default source
- field mapping

### Acceptance
- Parameter panel can render type/semantics without hardcoded per-field logic
- Metadata has no missing entries for exposed params

---

## 5) Two-Way Parameter Panel Enhancements (Optional)

### Goal
Turn the panel from descriptive + controls into a true editor/inspector loop.

### Deliverables
- State selector (`default`, `hovered`, `pressed`, `focused`, `disabled`)
- Show:
  - raw override value
  - effective resolved value for selected state
  - source indicator (`state override`, `base override`, `theme`)
- Editors for a limited core set (radius, opacity, background) with consistent behavior

### Acceptance
- User action updates template params and visual result immediately
- UI clearly indicates effective source

---

## 6) Testing Plan

### Unit tests
- precedence for each state
- inherit/set/clear semantics
- fallback to theme when overrides are absent
- metadata well-formedness

### Integration checks
- gallery prototype still renders and updates correctly
- runtime edits do not panic or deadlock

### Acceptance
- All new tests pass
- Existing workspace checks remain clean

---

## 7) Documentation Updates

### Deliverables
- Update `docs/proto1.md` with Proto2 references
- Add final Proto2 decisions and examples in this doc as implementation lands
- Document migration constraints for eventual core adoption

### Acceptance
- Docs reflect actual runtime behavior and current API shapes

---

## Implementation Sequence

## Phase A — Foundations
1. Introduce stateful override model
2. Add explicit inherit/set/clear mode
3. Refactor template resolve path around new model

## Phase B — Metadata + UI
4. Upgrade metadata schema
5. (Optional) Update parameter panel for selected-state editing and effective-value display

## Phase C — Hardening
6. Add precedence and semantic tests
7. Validate workspace checks and prototype interaction flows
8. Capture final design notes and promotion readiness

---

## Risks and Mitigations

### Risk: Parameter model gets too complex
- **Mitigation:** limit Proto2 scope to core visual fields first

### Risk: State explosion
- **Mitigation:** use fixed state set in Proto2; defer composite states unless proven necessary

### Risk: API churn before core adoption
- **Mitigation:** keep all changes in prototype namespace and mark as provisional

---

## Non-Goals (Proto2)

- Migrating all core controls to this model
- Building a full dependency-property system
- Finalizing long-term public API guarantees for non-prototype controls

---

## Exit Gate for Proto3 / Core Promotion Discussion

Proceed only if:

1. state-aware overrides are stable and understandable
2. semantics are explicit and test-backed
3. metadata supports scalable tooling
4. gallery demonstrates real two-way control with effective-value visibility
5. prototype remains isolated and non-disruptive

---

## Summary

Proto2 focuses on making the prototype architecture **state-aware, deterministic, and tooling-complete** while keeping risk low through strict localization. The output should be a convincing, test-backed model ready for incremental promotion decisions.
