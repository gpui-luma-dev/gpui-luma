# Proto1: Localized Template Parameterization (Spec & Guidelines)

## Status

- **Type:** Prototype spec
- **Scope:** Localized to `controls/prototypes` and gallery prototype pane(s)
- **Goal:** Validate architecture before promoting to core controls

---

## Problem Statement

Current control customization has two competing needs:

1. **Simple theming for most users**  
   Users should be able to apply theme/tokens without learning control internals.

2. **Deep customization for advanced scenarios**  
   Some use cases need strong per-control visual and structural customization without rewriting entire templates.

Historically, this tension causes one of two failures:

- Theme layer becomes overloaded with control-specific render internals.
- Template layer becomes difficult to inspect, tune, and evolve safely.

Proto1 is intended to test a middle path.

---

## What Proto1 Is Solving

Proto1 validates a design where:

- Template parameterization exists and is practical.
- Parameter metadata is introspectable in runtime tooling (gallery panel).
- Runtime parameters can be read and updated (two-way flow).
- All of this remains **localized** to prototype code and does not destabilize existing controls.

In short: **prove the customization architecture without breaking production paths.**

---

## Core Design Principles

1. **Localization First**
   - New architecture experiments live under `controls/prototypes/*`.
   - Existing non-prototype controls remain unchanged unless explicitly migrated.

2. **Theme Remains “Themey”**
   - Theme provides global primitives/semantics.
   - Theme should not become a direct bag of control render internals for every control.

3. **Template Owns Render Contract**
   - Template applies parameters onto resolved base appearance.
   - Template controls precedence and final render output.

4. **Metadata Is First-Class**
   - Parameters are discoverable with typed usage metadata.
   - Tooling can render editors and documentation from metadata.

5. **Escape Hatch Stays Simple**
   - For dramatic redesigns, write a custom template.
   - Parameterization is for structured variation, not replacing custom templates.

---

## Proto1 Architecture (Current)

### 1) Prototype Control

`ProtoButton` exists as a standalone prototype control stack:

- model
- control
- template

This isolates experimentation from stable controls.

### 2) Template Parameters

`ProtoButtonTemplateParams` provides a tunable parameter surface, including:

- base variant/size
- structural knobs (radius, spacing, height, typography)
- visual overrides (background/foreground/border/focus ring)
- behavior knobs (disabled opacity, cursor policy)

### 3) Runtime Read/Write

The template supports runtime parameter access and mutation through trait methods:

- read current params
- write new params

This enables two-way tooling.

### 4) Metadata Registry

A template usage registry describes each parameter:

- name
- description
- type
- states
- mapped fields
- default source

This mirrors the successful theme metadata pattern and powers discovery/UI.

### 5) Gallery Prototype Integration

The prototype gallery pane demonstrates:

- live control rendering
- runtime parameter updates
- side panel metadata inspection

---

## Parameter Resolution Semantics

Proto1 uses the following conceptual precedence:

1. Theme-resolved base values
2. Template parameter overrides
3. Runtime updates to template params (same override layer, later write)

For each render pass:

- resolve base from theme
- apply current params
- render result

This ensures deterministic behavior and supports live editing.

---

## Benefits

1. **Low risk experimentation**
   - Prototype code path avoids destabilizing stable controls.

2. **Better API boundaries**
   - Keeps theme focused and avoids leaking internals as primary public API.

3. **Tooling-friendly**
   - Metadata allows dynamic UI generation and auditability.

4. **Operational clarity**
   - Explicit parameter model + precedence reduces ambiguity.

5. **Migration-ready**
   - If successful, patterns can be promoted incrementally to real controls.

---

## Known Gaps / Next Work

1. **State-specific parameterization**
   - Current params are mostly single-value overrides.
   - Need structured per-state overrides (`default`, `hovered`, `pressed`, `focused`, `disabled`) for visual fields.

2. **Explicit override semantics**
   - Formalize inherit vs set vs clear behavior for nullable values (e.g., focus ring).

3. **Metadata/editor convergence**
   - Metadata currently documents shape; future should drive richer editors and effective-value display.

4. **Tests for merge/precedence behavior**
   - Add coverage for theme+params+runtime updates and state interactions.

---

## Guidelines for Contributors

### Scope & Safety

- Keep Proto1 changes localized under prototype namespaces.
- Do not refactor production control APIs as part of Proto1 unless explicitly requested.

### API Design

- Prefer typed fields/enums over ad-hoc string keys.
- Keep defaults explicit and documented.
- Preserve deterministic precedence rules.

### Metadata

- Any new parameter should include metadata entry.
- Metadata should include state applicability and default source.
- Metadata names/descriptions should be editor-friendly.

### Gallery

- Keep the prototype pane focused on architecture validation, not visual polish.
- Use controls in pane to demonstrate real read/write behavior.
- Keep panel code modular so prototype pane logic stays focused.

### Promotion Criteria (Proto -> Core)

A pattern should be considered for promotion only after:

- API ergonomics are acceptable in real usage.
- precedence/override behavior is well-tested.
- state-specific model is defined.
- metadata is complete enough for tooling.

---

## Non-Goals (Proto1)

- Rewriting all controls to the new parameter model.
- Building a full dependency-property system.
- Eliminating custom templates as the advanced path.
- Finalizing every future API detail in this prototype stage.

---

## Summary

Proto1 establishes a practical prototype for localized template parameterization with runtime introspection and mutation. It addresses the key architectural tension between theme simplicity and control-level customization while minimizing risk to existing controls.

The immediate path forward is to add state-specific parameterization and tighten override semantics, then evaluate incremental promotion to core controls.