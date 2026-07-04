# Issue: Pager Button Integration Reviews The Wrong Seam

## Description

`crates/sdk/src/controls/pager/` currently integrates with buttons by **reconstructing and tuning `ButtonFamilyLook` directly** instead of treating buttons as a higher-level control surface.

That works, but it couples pager to the internal shape of button look resolution:

- `ButtonFamilyLook`
- `compose_button_family_look(...)`
- button padding / gap / height fields
- button typography internals
- button role-specific geometry decisions

This became visible during the button size/radius preview experiment: pager had to change when button-look internals changed, even though pager itself was not part of the feature being explored.

The concern is not that pager is broken today. The concern is that pager is using the **wrong ownership seam**.

## Current Shape

Pager currently does two things:

1. resolves its own `PagerLook`
2. manually maps that look into a custom `ButtonFamilyLook`

Relevant paths:

- `crates/sdk/src/controls/pager/theme.rs`
- `crates/sdk/src/controls/pager/template.rs`

Today `PagerTheme::resolve_button_look(...)`:

- resolves the default button palette,
- builds a synthetic `StandardBoxScale`,
- calls `compose_button_family_look(...)`,
- then further mutates the resulting `ButtonFamilyLook` in `tune_pager_button_look(...)`.

That means pager is not just styling the rendered button output. It is participating directly in the button look-construction pipeline.

## Why This Is A Problem

### 1. Pager is too sensitive to button internals

If button look internals change, pager becomes a downstream maintenance site.

Examples:

- adding a new field to `ButtonFamilyLook`
- changing how icon sizing should relate to typography
- changing role-specific radius behavior
- moving size semantics into a richer button-local abstraction

Pager should not need review every time button look composition evolves.

### 2. Pager is bypassing the intended control/template seam

The repo architecture says:

- model defines semantic inputs,
- template renders resolved values,
- theme resolves semantic inputs into concrete values.

Pager currently reaches below “use a button” and instead says:

- “give me button parts, I will rebuild the look I want.”

That is a lower seam than most consumers should use.

### 3. Pager duplicates button ownership decisions

Pager currently restates button concerns like:

- height
- padding
- gap
- typography
- icon sizing implications
- icon-vs-text geometry branching

Those are button-family concerns. Pager may need a compact pager-specific variant, but it should not need to manually own the full translation.

### 4. This makes local experiments ripple through SDK consumers

The button size/radius experiment demonstrated the real cost:

- a local Theme Studio exploration touched pager
- not because pager behavior needed work
- but because pager had adopted the button look’s internal data shape as its dependency surface

That is exactly the kind of ripple the SDK should avoid.

## What Pager Probably Should Do Instead

Pager should move toward one of these seams, in order of preference:

### Option A: Pager uses a button-facing semantic theme seam

Expose a higher-level way for pager to say:

- use pager button variant
- use pager size
- use pager radius style
- use pager typography style

Then the button/look layer resolves that into a real `ButtonFamilyLook`.

Pager stays an input source, not a look assembler.

### Option B: Pager uses a specialized button template/theme adapter

Instead of reconstructing the button look manually, pager could request:

- a pager-specific button template
- or a pager-scoped button theme adapter

That adapter would live on the button/look side, not inside pager itself.

This still allows pager-specific behavior, but keeps button ownership in the button seam.

### Option C: Pager uses template modifiers for render-only adjustments

If the needed differences are mostly presentational, pager should prefer:

- template modifiers
- layout wrappers
- pager-local container composition

over direct mutation of button look internals.

This will not cover every pager requirement, but it is the right first choice when pager only needs minor output shaping.

## Recommended Direction

The recommended fix is:

- **stop treating `ButtonFamilyLook` as pager’s public dependency surface**
- give pager a more semantic integration point with buttons

Concretely:

- keep `PagerLook` as pager-owned
- remove direct construction/tuning of `ButtonFamilyLook` from pager over time
- introduce a button/look-side translation layer for pager button appearance

In other words:

- pager should describe what kind of button it wants
- button/look code should decide how that becomes concrete geometry and typography

## Tasks

- [ ] Audit `crates/sdk/src/controls/pager/theme.rs` for every place pager reconstructs button look internals.
- [ ] Decide whether pager needs:
  - a pager-specific button theme adapter,
  - a pager-specific button template,
  - or a semantic button variant hook in the look layer.
- [ ] Move pager-specific button geometry translation out of `PagerTheme::resolve_button_look(...)`.
- [ ] Reduce pager’s dependency on `compose_button_family_look(...)` and direct `ButtonFamilyLook` mutation.
- [ ] Verify that future button look changes do not require pager edits unless pager behavior itself changes.

## Acceptance Criteria

- Pager no longer needs to manually assemble and tune `ButtonFamilyLook` field-by-field.
- Button-family internal struct changes do not automatically force pager updates.
- Pager still supports compact numeric/minimal variants without raw `div`-only button reimplementation.
- Button ownership remains with the button/look seam rather than migrating into pager.
