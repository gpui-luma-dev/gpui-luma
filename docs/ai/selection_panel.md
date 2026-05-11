# Selection Panel: Preparation Task

## Purpose

Define and build a first-class `selection_panel` control that selector-family controls compose, instead of each control owning popup-list behavior/rendering.

This is prep work for composition-first selectors.

## Why now

Current selector-family controls still carry too much control-local panel/list wiring (`autocomplete`, `combobox`, `search_selector`, `selector`).

A standalone `selection_panel` should centralize:
- row rendering shell
- active/selected/hovered/disabled row state
- scroll surface + ensure-visible behavior
- row interaction event wiring
- optional content presenter

## Exemplar: Navigation Sidebar

Use `navigation_sidebar` as the structural exemplar:
- richer item model (`id`, label, icon, children-ish metadata patterns)
- presenter-driven custom row content (`presenter` support)
- clear separation: model/control/template/theme
- focused event wiring in control; content customization via presenter

The selection panel should adopt this style for item richness + presenter usage, but remain list/panel-focused (not tree/sidebar-focused).

## Inputs from existing selector work

Already available and must be reused where possible:
- shared selector behavior utilities in `selector_components` (filtering, selection, popup-list highlight movement)
- shared row/list visual primitives in `selector_components`
- shared panel/popup shell helpers
- shared panel appearance mapping helper (`resolve_selector_components_panel_appearance`)

## Hard constraint: DO NOT use `listbox`

`selection_panel` must **not** be derived from, wrapped around, copied from, or implemented on top of `listbox`.

This is a strict requirement:
- Do **not** use `listbox` control internals, templates, theme shapes, or scrolling layout assumptions.
- Do **not** "start from listbox and adapt".
- Do **not** re-export or alias `listbox` behavior as `selection_panel`.

`selection_panel` must be grounded in selector popup and floating-menu panel patterns only.

## Inputs from floating menu (must reuse)

`floating_menu` already solves several panel/list concerns. `selection_panel` should reuse patterns and code paths where practical, instead of rebuilding equivalents.

Reuse directly:
- Pure state object style from `floating_menu/state.rs`:
  - explicit step/boundary/activate methods
  - data-only state transitions, testable without rendering
- Render contract shape from `floating_menu/template.rs`:
  - `RenderModel` + `TemplateHandlers` split
  - template owns structure/chrome only; behavior stays in control/state
- Appearance contract style from `floating_menu/theme.rs`:
  - explicit panel + row appearance fields
  - token mapping parity for background/border/hover/disabled states

Adapt (do not copy blindly):
- Keep `selection_panel` flat-list first (`source_index`/`visible_index`), not submenu path-based.
- Use selector-rich item contract instead of `MenuItem` coupling.
- Keep official control split (`model/control/template/theme`) for `selection_panel`.

Avoid duplicating:
- row shell implementation that already exists in shared selector-components row/list primitives
- panel token mapping logic that can map from shared appearance helpers

## Inputs from gallery selector samples

The gallery selector pane demonstrates concrete customization needs:
- custom row content (swatch + metadata)
- alternate visual semantics while preserving selection behavior

`selection_panel` must support this directly, without requiring per-control duplicated items-template stacks.

## Task definition

### 1) Introduce `selection_panel` control as official SDK control

Create control module with standard split:
- `model.rs`
- `control.rs`
- `template.rs`
- `theme.rs` (or explicit shared appearance usage contract)
- `mod.rs`

### 2) Define rich item contract

Panel items must support at least:
- stable `id`
- `label`
- `value` (optional but modeled)
- `enabled`
- optional visual metadata (icon/accessory payload)

### 3) Define presenter contract

Add row content presenter for panel rows.

Minimum presenter model should include:
- control/panel id
- item reference
- source index + visible index
- selected
- active
- hovered
- pressed
- focused/focus_visible
- enabled
- sibling_count

Default presenter remains label text.

### 4) Define panel behavior ownership

`selection_panel` owns:
- visible row rendering
- row interaction hit targets
- hover/active visual state
- selection-marker region (if enabled by template/theme)
- scroll surface integration and ensure-visible API

Selector-family controls own:
- trigger behavior
- query/filter policy
- control-specific events and policies
- mapping core state into panel input model

### 5) Define event surface

Panel emits focused row interaction events (minimum):
- row hover change
- row click/activate
- optional active index change callback path

### 6) Template/theme constraints

- One panel template contract (no per-selector-family duplicate item template stacks)
- One row shell path via shared selector-components row/list primitives
- Theme path should route through shared selector appearance mapping, with controlled overrides

## Non-goals

- No compatibility/legacy wrapper APIs
- No attempt to solve all selector composition in this task
- No trigger composition changes in this task


## Demonstration
- Create gallery specific pane that demonstrates this new panel using similar structure and features as the the 'Floating Menu' pane in the gallery application

## Acceptance criteria

- A single panel control renders selector-family popup lists.
- Gallery selector sample custom row content is implemented through panel presenter, not duplicated control-local template stacks.
- No duplicated control-local popup list files remain for selector-family controls.
- Selector-family controls differ primarily by trigger/query/event specializations.

## Risks / guardrails

- Do not change public behavior semantics while swapping panel internals.
- Verify keyboard navigation parity after each migration step.
- Keep source vs visible indices explicit everywhere.
