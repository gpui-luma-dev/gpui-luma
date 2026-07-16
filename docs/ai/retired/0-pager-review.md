# Issue: Pager Button Integration Reviews The Wrong Seam

## Description

`crates/sdk/src/controls/pager/` used to integrate with buttons by **reconstructing and tuning `ButtonFamilyLook` directly** instead of treating buttons as a higher-level control surface.

That worked, but it coupled pager to the internal shape of button look resolution:

- `ButtonFamilyLook`
- `compose_button_family_look(...)`
- button padding / gap / height fields
- button typography internals
- button role-specific geometry decisions

This became visible during the button size/radius preview experiment: pager had to change when button-look internals changed, even though pager itself was not part of the feature being explored.

The concern was not that pager was broken. The concern was that pager was using the **wrong ownership seam**.

## Status

Fixed in the current implementation.

Pager now asks its theme for a pager-scoped button template via `PagerTheme::button_template(&PagerLook)`. The SDK pager template no longer installs a per-button `ButtonRenderModel::look` closure and no longer calls a pager-owned `resolve_button_look(...)`.

The concrete shadcn pager-button geometry translation now lives in the shadcn look crate:

- `crates/look-shadcn/src/controls/pager.rs`
- `crates/look-shadcn/src/controls/templates.rs`

This keeps pager behavior and layout in the SDK while keeping button appearance resolution on the shadcn/button side.

## Closure Notes

Completed changes:

- Removed `PagerTheme::resolve_button_look(...)` and the pager-owned `tune_pager_button_look(...)` helper from `crates/sdk/src/controls/pager/theme.rs`.
- Updated `crates/sdk/src/controls/pager/template.rs` so pager buttons request `theme.button_template(&PagerLook)` instead of installing a per-button look closure.
- Added `pager_button_look(...)` in `crates/look-shadcn/src/controls/pager.rs` for shadcn-owned pager button geometry and typography translation.
- Added `ShadcnPagerButtonTheme` in `crates/look-shadcn/src/controls/templates.rs` so focused-probe and normal button rendering use the same shadcn pager-button adapter.
- Renamed active shadcn adapter structs and comments away from Radix terminology. Remaining active `radix` matches are standard `from_str_radix` numeric parser calls.

Verification completed:

- `cargo fmt --all`
- `cargo check -p gpui-luma -p gpui-luma-look-shadcn -p gpui-luma-gallery -p gpui-luma-theme-studio`
- `cargo clippy -p gpui-luma -p gpui-luma-look-shadcn --lib -- -D warnings`
- Visual verification completed by user.

Known unrelated blockers:

- `cargo test -p gpui-luma pager` is blocked by an unrelated selector test compile error in `crates/sdk/src/controls/selector/template.rs`.
- App-level clippy still has pre-existing Theme Studio baseline warnings.

## Original Shape

Pager previously did two things:

1. resolves its own `PagerLook`
2. manually maps that look into a custom `ButtonFamilyLook`

Relevant paths:

- `crates/sdk/src/controls/pager/theme.rs`
- `crates/sdk/src/controls/pager/template.rs`

Previously `PagerTheme::resolve_button_look(...)`:

- resolves the default button palette,
- builds a synthetic `StandardBoxScale`,
- calls `compose_button_family_look(...)`,
- then further mutates the resulting `ButtonFamilyLook` in `tune_pager_button_look(...)`.

That meant pager was not just styling the rendered button output. It was participating directly in the button look-construction pipeline.

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

Pager previously reached below “use a button” and instead said:

- “give me button parts, I will rebuild the look I want.”

That is a lower seam than most consumers should use.

### 3. Pager duplicates button ownership decisions

Pager previously restated button concerns like:

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

- [x] Audit `crates/sdk/src/controls/pager/theme.rs` for every place pager reconstructs button look internals.
- [x] Decide whether pager needs:
  - a pager-specific button theme adapter,
  - a pager-specific button template,
  - or a semantic button variant hook in the look layer.
- [x] Move pager-specific button geometry translation out of `PagerTheme::resolve_button_look(...)`.
- [x] Reduce pager’s dependency on `compose_button_family_look(...)` and direct `ButtonFamilyLook` mutation.
- [x] Verify that future button look changes do not require pager edits unless pager behavior itself changes.

## Acceptance Criteria

- Pager no longer needs to manually assemble and tune `ButtonFamilyLook` field-by-field.
- Button-family internal struct changes do not automatically force pager updates.
- Pager still supports compact numeric/minimal variants without raw `div`-only button reimplementation.
- Button ownership remains with the button/look seam rather than migrating into pager.
