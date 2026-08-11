# SDK Studio

## Summary

Create a separate `sdk-studio` application that serves as the SDK's canonical visual and
interaction test harness.

`sdk-studio` should be structurally similar to Luma Studio, but it should present the SDK with
one comprehensive SDK default theme and no Shadcn-specific product or theme-authoring surface.
Luma Studio remains the Shadcn/CSS look studio.

The two applications intentionally have different responsibilities:

```text
sdk-studio
  gpui-luma SDK + one comprehensive SDK default look
  validates SDK completeness and default behavior

luma-studio
  gpui-luma SDK + gpui-luma-look-shadcn
  validates downstream look integration and Shadcn visual behavior
```

This is not a request to render two themes side by side in one application.

## Problem

The SDK is intended to be lookless and reusable, but most visual validation currently happens in
Luma Studio, which is coupled to `gpui-luma-look-shadcn`. That makes it difficult to distinguish
between:

- an SDK control that is complete and look-independent;
- a control that only works because Shadcn supplies an implementation detail;
- a missing or incomplete SDK default theme implementation;
- a defect in the Shadcn look itself.

The SDK needs its own application where the SDK controls can be exercised without importing the
Shadcn look. This application should be a maintained development tool, not a second product
studio or a showcase of multiple visual systems.

## Goals

### Primary goals

- Create an `apps/sdk-studio` application with a Luma Studio-like workbench shell.
- Provide one comprehensive `SdkDefaultLook` / SDK default theme implementation.
- Exercise the SDK controls through their public builders, theme traits, templates, and event
  contracts.
- Keep the app independent of `gpui-luma-look-shadcn` and `gpui-luma-look-shadcn-inspect`.
- Make SDK changes visible through compilation, runtime interaction, and visual inspection.
- Ensure a new SDK control cannot be considered complete until it has a usable default look path.
- Keep downstream looks possible without requiring SDK or application changes.
- Establish a stable place for future SDK regression examples and interaction-state probes.

### Secondary goals

- Reuse genuinely SDK-neutral exposition infrastructure from Luma Studio where that does not
  introduce a Shadcn dependency.
- Keep the control catalog organized by SDK control family rather than by Shadcn component name.
- Make focus, keyboard traversal, disabled behavior, overlays, animation, resizing, and event
  emission easy to inspect.
- Use the app as a manual visual smoke test after changes to SDK templates, theme contracts, or
  control models.

## Non-goals

`sdk-studio` should not become another copy of the full Luma Studio product surface.

It should exclude:

- Cards and the cards/demo board.
- Dashboard content and dashboard panels.
- Theme Usage / Shadcn token provenance views.
- Theme selection among tweakcn or other Shadcn themes.
- CSS catalog editing or `style.toml` editing.
- Shadcn-specific palette editing and palette derivation.
- Shadcn-specific theme inspectors and provenance adapters.
- Product-specific shell demos that do not test SDK controls.
- A side-by-side or toggleable two-look rendering inside the same app.

Palette is excluded from the initial scope. It can be reconsidered later only if the SDK default
theme itself requires a small, SDK-owned token inspection surface. A Shadcn palette editor does
not belong in `sdk-studio`.

## Architectural position

The SDK core remains lookless. The default theme should not be implemented by adding Shadcn
knowledge to `crates/sdk`.

Preferred dependency shape:

```text
crates/sdk-default-theme  ──→ crates/sdk
crates/look-shadcn         ──→ crates/sdk
apps/sdk-studio            ──→ crates/sdk
apps/sdk-studio            ──→ crates/sdk-default-theme
apps/luma-studio           ──→ crates/sdk
apps/luma-studio           ──→ crates/look-shadcn
```

The exact crate name is open, but the default theme should be one cohesive implementation. The
fact that it implements individual SDK interfaces such as `ButtonTheme`, `TextFieldTheme`,
`SliderTheme`, or `SidebarTheme` must not turn those interfaces into separate user-facing themes.

```text
SdkDefaultLook
  ├─ button-family theme
  ├─ text-field and text-area themes
  ├─ choice-control themes
  ├─ selector and collection themes
  ├─ menu and overlay themes
  ├─ navigation and layout themes
  └─ range, progress, color, and scrollbar themes
```

The default theme must be comprehensive enough that `sdk-studio` does not need to fall back to
Shadcn implementations for ordinary SDK controls.

## Relationship to Luma Studio

Luma Studio should remain the Shadcn/CSS look studio. It owns:

- Shadcn CSS catalogs;
- `style.toml` semantic mappings;
- Shadcn variants and theme customization;
- theme sidebar editing;
- palette and token usage inspection;
- Shadcn-specific control inspection and provenance.

`sdk-studio` should use a similar application shell and content organization where useful, but its
visual source must be the SDK default theme. It should not import a Shadcn look merely to style the
workbench chrome.

The first version can be a focused subset of the existing Luma Studio structure:

```text
SDK Studio
  ├─ Controls
  ├─ Style Guide / SDK visual primitives
  └─ optional SDK diagnostics or interaction test pages
```

The exact tab names may differ. The important boundary is that every page exists to validate SDK
behavior or the SDK default theme.

## Functional requirements

### Application

- Add a workspace member for `apps/sdk-studio`.
- Provide a standalone binary and normal GPUI application entry point.
- Initialize the SDK, focus bindings, key handling, assets, and fonts required by the default
  theme.
- Provide a stable window shell with navigation and a content host.
- Keep the application launch path independent of Luma Studio's theme-selection arguments.

### Default theme

- Implement one default look/theme covering all SDK controls that are exposed in the studio.
- Resolve colors, metrics, typography, borders, radii, shadows, focus treatment, disabled state,
  and interaction-state visuals through the default theme boundary.
- Provide default builders or factories for every control shown by the app.
- Avoid importing Shadcn enums, Shadcn token names, Shadcn stylesheet mappings, or Shadcn look
  extensions into the default theme or SDK Studio.
- Prefer neutral, intentionally distinct visuals so accidental Shadcn coupling is obvious during
  manual inspection.
- Keep fallback behavior explicit. A missing default-theme implementation should be visible as a
  compile-time or clearly documented runtime gap, not silently replaced by Shadcn styling.

### Control coverage

The initial catalog should cover the SDK control families already represented in the SDK and
Luma Studio, including:

- command buttons and icon buttons;
- button-family controls;
- checkbox, radio button, radio group, switch, toggle, and toggle groups;
- text fields, text areas, and composed input controls;
- sliders and color controls where the default theme can provide their chrome;
- progress and stepper controls;
- selectors, list boxes, list views, tree views, and selection panels;
- tabs, toolbar, pager, accordion, and sidebar/navigation controls;
- popup menus, context menus, floating menus, anchored panels, overlay windows, and slide panels;
- split views, resizable panels, dock splitters, scrollbars, and scroll containers.

Coverage should be tracked by SDK control family, not just by whether a matching Luma Studio page
already exists. A control with multiple templates or modes needs representative coverage for each
publicly supported mode.

### Interaction and state coverage

Each relevant exposition should make it possible to inspect:

- default, hover, pressed, focused, focus-visible, disabled, selected, open, invalid, and loading
  states where supported;
- small, medium, and large sizes where supported;
- keyboard focus traversal and activation;
- semantic events and programmatic synchronization;
- overlay open, dismiss, focus restore, and Escape behavior;
- resizing, scrolling, and animation behavior;
- nested/composed controls using SDK-owned contracts.

## Prep work

### 1. Inventory the current Luma Studio surface

Before creating the app, classify existing Luma Studio code into three groups:

1. **SDK-neutral and reusable**
   - control exposition models;
   - event log presentation;
   - viewport and inspector split layout;
   - generic catalog selection infrastructure;
   - documentation shell primitives;
   - examples that only require SDK theme traits.

2. **Shadcn-dependent and excluded**
   - `ShadcnLook` signatures;
   - `ShadcnLookControlExt` factories;
   - `style.toml` and CSS provenance;
   - Shadcn variants and token names;
   - palette and theme usage panels;
   - look-shadcn-inspect adapters;
   - Shadcn-specific style-guide matrices.

3. **App/product-specific and excluded or redesigned**
   - cards;
   - dashboard panels;
   - theme sidebar editing;
   - product shell examples;
   - assumptions about Luma Studio's active theme and overrides.

Do not copy the entire `apps/luma-studio/src/studio` tree and remove files after the fact. The
inventory should identify reusable seams first, then establish a smaller SDK Studio module graph.

### 2. Define the default-theme contract

Document the single default theme's ownership and coverage before implementing the app.

The contract should specify:

- which SDK theme traits must be implemented;
- which SDK controls are intentionally unsupported, if any;
- how metrics and typography are resolved;
- how focus and disabled visuals are represented;
- which fallback values are allowed;
- how composed controls obtain child themes;
- how overlays obtain panel, backdrop, and focus behavior;
- how color-control chrome is provided without Shadcn token coupling.

The contract should make an incomplete implementation obvious. A new control should not be
considered SDK-complete merely because it compiles with a Shadcn look.

### 3. Audit SDK fallback leakage

Use the new app design to identify SDK code that constructs hidden defaults inside control
rendering or control models. Pay particular attention to:

- direct construction of default theme objects inside `control.rs`;
- default colors or metrics created during render;
- composed controls that resolve child themes internally;
- templates that assume a product-specific theme;
- APIs that require Shadcn factories rather than generic theme traits.

The SDK Studio is a consumer of the public SDK surface. If it cannot provide a default look without
reaching into implementation details, the relevant SDK boundary needs review.

### 4. Decide the reuse boundary

Prefer extracting shared, SDK-neutral modules only when reuse reduces duplication without making
the SDK Studio depend on Shadcn concepts.

Potential shared units include:

- generic control catalog metadata;
- exposition registration and selection;
- event log rendering;
- inspector split layout;
- documentation shell layout;
- generic preview handlers;
- SDK control sample data.

Do not share a module merely because the two applications display similar pixels. If a shared
module accepts `ShadcnLook`, `StylesheetConfig`, or Shadcn-specific variants, it belongs on the
Luma Studio side or needs a generic abstraction first.

### 5. Establish workspace and build conventions

Decide before implementation:

- package and binary names;
- whether the default theme is a new crate or initially app-local;
- workspace membership and `default-members` behavior;
- profile optimization entries;
- asset and font ownership;
- Linux-specific dependencies;
- launch arguments and development commands;
- whether screenshots or golden visual captures will be added later.

The preferred long-term location for the default theme is a separate crate. An app-local
implementation is acceptable only as a short-lived bootstrap if it is kept behind a clean theme
boundary and scheduled for extraction.

## Implementation plan

### Phase 0: Confirm scope and dependency boundaries

- Review the current architecture and Luma Studio module graph.
- Finalize the initial SDK control coverage list.
- Confirm that cards, dashboard, theme usage, and palette are excluded from the first release.
- Decide whether a minimal SDK typography/style guide page is included.
- Select the default-theme crate name and dependency direction.
- Record any SDK controls that cannot yet be supported by a generic default theme.

### Phase 1: Create the default theme foundation

- Add the default theme crate or establish the temporary app-local module.
- Implement the shared theme context and foundational tokens.
- Implement typography, spacing, radii, borders, focus, disabled, and elevation policies.
- Add the default theme implementations for the core button-family, input, choice, and layout
  contracts.
- Add unit tests for representative state and metric resolution.
- Ensure no Shadcn crate is present in the default theme dependency graph.

### Phase 2: Build the SDK Studio shell

- Add `apps/sdk-studio` to the workspace.
- Add the application entry point, app shell, window setup, assets, and SDK initialization.
- Create navigation and content-host modules modeled on Luma Studio where appropriate.
- Add the Controls page and generic control catalog picker.
- Add a minimal documentation or Style Guide page only if it validates SDK-owned primitives.
- Use the SDK default theme for all app chrome and content.

### Phase 3: Port SDK control expositions

- Port control exposition pages in control-family order.
- Replace Shadcn look factories with default-theme factories or generic SDK theme interfaces.
- Remove Shadcn descriptions, Shadcn variant names, CSS token references, and Shadcn snippets.
- Preserve event demonstrations and programmatic synchronization behavior.
- Add representative state matrices for each control family.
- Keep app composition within the SDK builder and theme boundaries.

Suggested order:

1. buttons and button-family controls;
2. checkbox, radio, switch, toggle, and groups;
3. text fields, text areas, and composed inputs;
4. sliders, progress, stepper, and color controls;
5. selectors, list boxes, list views, and tree views;
6. tabs, toolbar, pager, accordion, and sidebar;
7. menus, overlays, dialogs, and panels;
8. split views, resizable panels, dock splitters, scrollbars, and scroll containers.

### Phase 4: Extract reusable neutral infrastructure

- Identify duplicated code between `sdk-studio` and Luma Studio after the first port.
- Extract only modules that have a genuinely generic API.
- Keep Shadcn-specific inspection and theme-editing code in Luma Studio.
- Keep SDK Studio's catalog and exposition metadata free of CSS/token assumptions.
- Add documentation for any new shared crate or module boundary.

### Phase 5: Add regression and completeness checks

- Add a default-theme coverage checklist or registry.
- Ensure every exposed SDK control has a default-theme factory.
- Add tests for public builder construction where practical.
- Add compile-time coverage through the `sdk-studio` dependency graph.
- Run the app manually after SDK control changes.
- Verify that changes to Shadcn styling are not required for SDK Studio to build.
- Verify that Luma Studio still exercises the same SDK control contracts through Shadcn.

### Phase 6: Use both applications as separate validation gates

For an SDK control change:

```text
Change SDK control
  ↓
Build and inspect sdk-studio
  ↓
Confirm default-theme behavior and interaction states
  ↓
Build and inspect luma-studio
  ↓
Confirm Shadcn look integration and product-specific behavior
```

The applications should not render two looks together. They provide separate, single-look test
environments with different ownership boundaries.

## Acceptance criteria

### Application boundary

- `apps/sdk-studio` builds and launches as an independent application.
- Its dependency graph contains the SDK and the SDK default theme, but not the Shadcn look crates.
- It has no cards, dashboard, theme usage, or Shadcn palette surface.
- Palette is explicitly out of scope unless a later issue adds an SDK-owned replacement.

### SDK default theme

- One cohesive default theme provides the visual implementation for the controls exposed by the
  app.
- The theme is not split into multiple user-facing SDK themes.
- Default-theme implementations do not import Shadcn token names, variants, or stylesheet types.
- Missing theme coverage is reported explicitly rather than silently falling back to Shadcn.

### SDK validation

- Controls can be constructed using public SDK APIs and the default theme.
- The app demonstrates representative states, sizes, focus behavior, events, overlays, and
  programmatic updates.
- SDK control changes produce useful compile-time or visual feedback in `sdk-studio`.
- Composed controls do not construct hidden product-specific defaults during render.

### Downstream look validation

- Luma Studio remains independently buildable with `gpui-luma-look-shadcn`.
- The same SDK controls continue to receive Shadcn look implementations through look boundaries.
- SDK Studio does not need to know how Shadcn CSS or `style.toml` works.
- Luma Studio does not need to know how the SDK default theme is implemented.

## Risks and mitigations

### Duplicating Luma Studio

**Risk:** The new app becomes a fork of Luma Studio with two copies of every exposition.

**Mitigation:** Start with a module inventory and extract only generic infrastructure. Keep the
SDK Studio control catalog intentionally smaller and SDK-focused.

### Default theme becomes a second product theme

**Risk:** The default theme accumulates Shadcn-like variants or multiple competing style systems.

**Mitigation:** Treat it as one neutral reference implementation. Keep theme traits internal seams,
not separate user-facing themes.

### SDK remains accidentally dependent on Shadcn

**Risk:** The app compiles only because an SDK control reaches into Shadcn factories or token names.

**Mitigation:** Keep Shadcn crates out of the `sdk-studio` dependency graph and audit render-time
default construction in the SDK.

### Incomplete coverage creates false confidence

**Risk:** The app demonstrates only easy controls while overlays, composed inputs, or layout
controls remain untested.

**Mitigation:** Maintain a control-family coverage registry and require representative state
coverage for every exposed control.

### App chrome hides SDK defects

**Risk:** Custom app-level styling makes the SDK look complete even when SDK controls are not
properly themed.

**Mitigation:** Use the default theme for app chrome where possible and keep custom chrome minimal.
The application should compose SDK controls rather than replace them with raw interactive markup.

## Open decisions

- What should the default-theme crate be named?
- Should the first default theme live in a new crate immediately, or be bootstrapped in the app
  and extracted in the same change series?
- Which SDK color controls require default chrome in the first release?
- Should the SDK Style Guide page be included in the MVP, or should the first release contain only
  Controls and diagnostics?
- Should generic exposition metadata be extracted into a shared crate, or duplicated initially to
  keep the first implementation small?
- Should the app include a lightweight event log and focus-state diagnostic page?
- What visual regression format should be adopted later: screenshots, scripted interaction
  captures, or manual review checklists?

## Definition of done

The change is complete when `sdk-studio` is a standalone, Shadcn-free application that exercises
the SDK through one comprehensive default theme, while Luma Studio independently exercises the
same SDK through `ShadcnLook`. A developer can change an SDK control, open `sdk-studio` to inspect
the default behavior, and then open Luma Studio to confirm that downstream looks continue to
benefit from the SDK without either app rendering multiple themes together.
