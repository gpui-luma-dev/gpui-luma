# Proposed Architecture Roadmap

Status: proposal for post-spike planning. This document is intentionally not an implementation task list yet.

## Purpose

The current workspace has proved that a look-driven SDK can work, but the first implementation also exposed several boundaries that are too tightly coupled:

- `crates/sdk` contains foundational controls, but also an expanding color-control surface.
- `crates/look-shadcn` contains both look infrastructure and higher-level, look-specific components such as buttons, cards, badges, and themed control templates.
- `apps/luma-studio` is a development workbench and visual proof surface, not the long-term home for reusable component systems.
- Shadcn semantic variants and SDK variants were practical during discovery, but neither should define the permanent cross-look vocabulary.
- Shadcn chart tokens exist in the CSS theme even though graphing controls are not part of the SDK.

The proposed direction is additive and staged. Preserve the working Shadcn behavior, create a clean Radix implementation, and extract shared infrastructure only after a second look demonstrates what is truly common.

## Guiding principles

1. Do not destabilize `look-shadcn` in order to design `look-common` speculatively.
2. Keep the SDK focused on primitives, interaction contracts, layout, accessibility, and rendering infrastructure.
3. Treat Luma Studio as a workbench, executable specification, and visual regression surface.
4. Keep look-specific vocabulary out of the SDK wherever possible.
5. Prefer look-owned recipes over universal variant enums.
6. Preserve authored theme data and expose derived or mapped values with provenance.
7. Let Radix scales remain Radix concepts; normalize only at the semantic boundary.
8. Extract code into `look-common` only when both looks need the same behavior.
9. Keep graphing and advanced color editing as separate systems from the SDK core.
10. Avoid committing until the public API and crate boundaries have been reviewed.

## Target crate topology

```text
crates/sdk
  Foundational primitives and interaction infrastructure

crates/look-common
  Shared look runtime contracts, recipes, resolution, and provenance

crates/look-shadcn
  CSS catalog, Shadcn mappings, compatibility behavior, and Shadcn components

crates/look-radix
  Radix families, 12-step scales, mappings, and Radix components

crates/color-controls       (later)
  Reusable color editing and palette exploration controls

crates/graphing             (later)
  Charting, plotting, scales, series, interaction, and visualization rendering

apps/luma-studio
  Workbench, demonstrations, inspectors, experiments, and visual verification
```

The exact crate names are provisional. The ownership boundaries are the important part.

## Boundary model

### SDK

The SDK should provide:

- layout, text, icons, scrolling, focus, keyboard, and accessibility infrastructure;
- primitive interactive controls such as buttons, fields, sliders, selection controls, and overlays;
- control state and event contracts;
- LMTP model/control/template seams;
- generic theme inputs where a primitive cannot function without them.

The SDK should not be the permanent home of:

- Shadcn or Radix variant vocabularies;
- cards, tags, badges, or other primarily compositional visual components;
- calendar/date-picker product systems;
- color palette editors and theme authoring workflows;
- charting or plotting systems.

Some existing SDK modules may remain there for compatibility while their future ownership is clarified.

### `look-common`

`look-common` should contain only concepts that can work without knowing whether the source is CSS variables, Radix scales, Tailwind colors, or another look:

- look lifecycle and mode switching;
- theme snapshots, revisions, and override propagation;
- semantic color resolution;
- metric, typography, radius, and elevation resolution;
- resolved values and provenance;
- stylesheet/recipe resolution that is not Shadcn-specific;
- shared appearance recipe structures;
- Luma Studio-facing usage metadata;
- shared look-bound control/template seams where duplication is proven.

It should not contain CSS parsing rules, Radix step logic, Shadcn token names, or a universal color-generation algorithm.

### `look-shadcn`

Initially retain the existing implementation, including:

- CSS catalog parsing;
- authored token preservation;
- Shadcn palette and state behavior;
- current `style.toml` compatibility;
- existing control factories and templates;
- Look Extensions such as derived `warning` and `success` tokens;
- Shadcn built-in themes and assets.

Migration into `look-common` should happen incrementally after `look-radix` creates evidence of duplication.

### `look-radix`

`look-radix` should own:

- named color families;
- light and dark 12-step scales;
- accent, gray, destructive, success, warning, and optional additional families;
- semantic-to-step mappings;
- contrast/foreground rules;
- Radix-specific component recipes;
- Tailwind seed-color support, if retained;
- Radix palette data for Studio inspection;
- Radix built-in themes.

The Radix implementation should initially be small and vertical rather than a copy of the entire Shadcn crate.

## Button direction

Buttons are the primary boundary test.

The current SDK and Shadcn variants were useful for discovery, but the permanent model should separate:

```text
SDK button
  behavior, content, layout, focus, disabled/loading state

Look-owned button recipe
  background, foreground, border, hover, pressed, focus, disabled,
  radius, typography, elevation, and provenance
```

Avoid adding more universal names such as `primary`, `secondary`, or `outline` to the SDK.

A small common treatment vocabulary may eventually be useful (`solid`, `soft`, `outline`, `ghost`, `link`), but it must not be confused with product intent or a complete variant system.

Look-specific recipe names can remain outside the SDK. For example, Shadcn and Radix may expose different recipe inventories while producing the same resolved appearance structure.

Existing SDK and Shadcn variant APIs should be treated as compatibility surfaces during migration, not as evidence that the model is final.

## Composed components

The architecture should distinguish primitives from component systems.

### Cards and tags

Cards, tags, badges, status labels, and similar controls are mostly compositions plus look-owned recipes. They should not force the SDK to define a universal visual vocabulary.

### Calendars

A calendar is a reusable component system, but not a foundational SDK control. Date calculation, localization, range selection, keyboard navigation, and disabled-date rules belong in a component crate. Its view should be composed from SDK primitives and active-look recipes.

### Luma Studio

Luma Studio should demonstrate and prove these components without owning their reusable implementation:

```text
Luma Studio proves a control
The owning crate defines the control
```

Studio prototypes may remain app-local while an API is unstable. Once a component is reusable and behaviorally proven, it should move to an appropriate crate rather than making Studio the architectural boundary.

## Color controls

The existing SDK color modules prove substantial reusable behavior, but they also contain domain-specific color editing concerns:

- color-space conversion;
- saturation/value, hue, alpha, ring, arc, and slider interaction;
- swatches and checkerboard rendering;
- synchronized multi-control compositions;
- palette editing and token provenance.

The eventual direction is a `color-controls` crate built on SDK primitives. It should consume a generic color editing/palette model rather than depend directly on `ShadcnLook` or `RadixLook`.

Potential shared data contracts include:

```text
ColorValue
ColorFamily
ColorStep
SemanticColor
ColorSource
ColorUsage
```

The first milestone should not move code solely for organizational purity. Use the current implementation to stabilize behavior and extract only after the public API is understood.

## Graphing and plotting

Chart tokens in Shadcn CSS should be treated as optional theme capabilities, not as evidence that chart controls belong in the SDK.

A future `graphing` crate should own:

- axes, scales, ticks, labels, and legends;
- categorical, sequential, and diverging palettes;
- series identity and stable color assignment;
- tooltips, hover, selection, zooming, and annotations;
- positive/negative/threshold semantics;
- accessibility and non-color encodings;
- performance and dense rendering concerns.

Looks should supply a `ChartTheme` or equivalent palette capability:

```text
categorical series palette
sequential scale
diverging scale
positive / negative / neutral
grid / axis / label
tooltip / selection
```

`chart-1` through `chart-5` can remain available in `look-shadcn` for compatibility, but Studio should identify them as theme tokens with no current SDK consumer.

## Color and provenance model

Do not make the generic CSS parser invent Radix-like scales or automatically derive every missing semantic token.

Represent the source and normalized meaning separately:

```text
ResolvedColor
  value
  semantic role
  source kind
  provenance
```

Examples:

```text
primary
  value: blue-9
  source: Radix scale
  provenance: accent family, step 9

primary
  value: --primary
  source: authored CSS variable
  provenance: CSS catalog

success
  value: derived color
  source: Look Extension
  provenance: configured derivation rule
```

Luma Studio should be able to distinguish authored, mapped, scale-based, derived, and fallback values.

## Proposed roadmap

### Phase 0: Public-release hardening

Goal: publish the current work honestly and reproducibly before starting the architecture migration.

Baby steps:

1. Inspect `git status` and decide which changes are intentionally public.
2. Review the current diff file by file; remove experiments that are not part of the release story.
3. Confirm no secrets, local paths, generated binaries, screenshots, or private assets are tracked.
4. Review `Cargo.toml`, package names, license metadata, README files, and workspace membership.
5. Add or verify a root README explaining that the project is alpha and identifying SDK, look, Studio, and experimental areas.
6. Document that Luma Studio is a development workbench, not a production application.
7. Identify known limitations, especially Shadcn-specific APIs, incomplete theme portability, chart tokens without chart controls, and experimental color controls.
8. Run formatting and the relevant check/test commands.
9. Launch Luma Studio and manually inspect the main control, color, palette, theme, and prototype surfaces.
10. Verify that theme edits update every demonstrated consumer that claims to be live.
11. Verify light/dark switching, reset behavior, and malformed/missing token behavior.
12. Confirm the public API examples compile and do not rely on private workspace paths.
13. Record known test failures separately from release-blocking failures.
14. Review documentation links for local-only `file://` paths and replace public-facing references with repository-relative links where appropriate.
15. Create a release checklist and do not commit until the intended public snapshot is approved.

Release gate:

- clean intentional diff;
- no secrets or private artifacts;
- documented alpha status;
- relevant checks pass or known failures are documented;
- Studio opens and demonstrates the advertised controls;
- no claim is made that all SDK controls or charting are production-ready.

### Phase 1: Inventory and boundary map

Goal: create an evidence-based map before moving code.

1. Classify SDK modules as primitive, shared infrastructure, composed component, or experiment.
2. Classify `look-shadcn` modules as source interpretation, shared-looking runtime, Shadcn policy, or component recipe.
3. Inventory all public `Shadcn*` types and extension traits.
4. Inventory current Studio prototypes and distinguish demonstrations from reusable components.
5. Record all color/token consumers, including currently unused chart tokens.
6. Record which controls require look factories and which can use generic SDK theme interfaces.
7. Create a small compatibility matrix for Shadcn, proposed Radix, and future looks.
8. Do not change behavior during this phase.

Deliverable: a reviewed boundary inventory and a list of candidate shared seams.

### Phase 2: Establish `look-common` minimally

Goal: create a small shared foundation without refactoring Shadcn wholesale.

Candidate first contents:

- theme mode and snapshot lifecycle;
- revision/override propagation;
- resolved color, metric, typography, and provenance records;
- semantic color role contracts;
- shared recipe/stylesheet resolution pieces proven by Phase 1;
- Studio usage metadata contracts.

Migration rule: move one seam at a time, retain Shadcn compatibility adapters, and keep the behavior covered before and after each move.

Do not initially move:

- CSS parsing;
- `ShadcnToken`;
- Shadcn palette fallback algorithms;
- Shadcn control naming;
- Radix scale concepts.

### Phase 3: Build a minimal `look-radix`

Goal: validate the new boundary with a useful but intentionally narrow look.

Initial scope:

- gray and accent families;
- destructive family;
- light/dark 12-step scales;
- background, surface, border, foreground, primary, destructive, and focus mappings;
- one solid, one soft, one outline, and one ghost button recipe;
- card, badge/tag, text field, and a small palette inspector;
- provenance showing family and step;
- live mode switching and color editing where the model supports it.

Acceptance criteria:

- no Radix token names are required in SDK APIs;
- no Shadcn variant names are required in Radix APIs;
- the same SDK primitives can render the demonstrated controls;
- Studio can inspect semantic role, source family, step, and usage;
- hover/pressed/disabled states use explicit scale steps where appropriate;
- authored and derived values remain distinguishable.

### Phase 4: Extract proven shared control infrastructure

Goal: reduce duplication revealed by the Radix implementation.

Likely candidates:

1. Shared look context and control binding.
2. Shared template adapters parameterized by resolved appearance.
3. Shared stylesheet/recipe resolver.
4. Shared provenance and usage inspection.
5. Shared control factory contracts.
6. Shared card/tag/badge recipe machinery, if both looks require it.

Keep source-specific palette construction and component policy in each look.

### Phase 5: Migrate `look-shadcn` incrementally

Goal: make Shadcn a consumer of the shared runtime without changing its user-visible behavior.

Migration order:

1. resolved values and provenance;
2. snapshot/mode/override lifecycle;
3. stylesheet resolution;
4. shared template adapters;
5. common control factories;
6. Studio inspection adapters.

Preserve compatibility aliases for existing Shadcn APIs. Remove or rename APIs only in a deliberate breaking-release plan.

### Phase 6: Move higher-level systems out of the SDK

Goal: clarify ownership after look parity is proven.

Potential extractions:

- `color-controls` for palette and color editing;
- a component crate for calendar/date controls;
- a component crate for cards, tags, badges, and other composed surfaces;
- `graphing` for professional charting and plotting.

Each extraction should start with a proven Studio prototype and a narrow public API. Avoid moving code merely to achieve a tidy directory tree.

### Phase 7: Expand Luma Studio into a workbench matrix

Goal: use Studio to demonstrate capability without making it the owner.

Add or organize workbench surfaces for:

- Template;
- Colors;
- Sizes;
- semantic and look-owned button recipes;
- Radix 12-step palettes;
- color controls;
- component compositions;
- graphing when the separate crate exists;
- theme usage and provenance.

Every surface should identify whether it demonstrates an SDK primitive, a look component, a shared component crate, or an app-local prototype.

## Testing and verification strategy

Every phase should include:

- compile checks for affected crates;
- focused unit tests for parsing, mapping, state resolution, and provenance;
- light/dark manual verification;
- live color override verification;
- Studio visual inspection;
- compatibility checks for existing Shadcn examples.

For Radix specifically, test:

- scale-step mapping;
- contrast/foreground selection;
- light/dark family pairing;
- semantic recipe enumeration;
- missing optional families;
- authored overrides;
- provenance and Studio usage display.

## Decisions to defer

Do not decide these until the first Radix vertical slice exists:

- whether `look-common` should be one crate or several smaller crates;
- whether button treatment names belong in `look-common`;
- whether color controls should be one crate or split into primitives and editors;
- whether calendars belong in a general components crate or a date-specific crate;
- whether graphing should use GPUI primitives directly or a renderer abstraction;
- whether Tailwind colors are source palettes, seed inputs, or compatibility data;
- whether existing SDK variants should be deprecated immediately or retained indefinitely.

## Definition of success

The architecture is successful when:

- Shadcn behavior remains stable while its internals can be migrated gradually;
- Radix can express its 12-step system without Shadcn-shaped SDK APIs;
- the SDK exposes primitives rather than an ever-growing component catalog;
- cards, tags, calendars, color editing, and graphing can evolve independently;
- Luma Studio demonstrates all systems without owning their reusable implementation;
- Studio can explain where every visible color came from;
- adding a third look does not require another large copy of `look-shadcn`.

