# TODO: Popup infrastructure refactor (`popup_pane` + `popup_list`)

## Why this is needed

Selection controls currently split popup responsibilities inconsistently:

- `Selector` items template includes outer popup chrome (background, border, radius, shadow).
- `Autocomplete` and `ComboBox` historically split shell and items differently, and are currently in transition.
- `SearchSelector` composes a custom popup layout (search header + list) with separate shell ownership.
- `PopupMenu` has similar outer chrome requirements, but through a separate implementation path.

This inconsistency causes:

- template matrix/pane confusion (what is being previewed vs what is runtime chrome)
- visual drift (shadow/border/radius behavior diverges between controls)
- duplicated popup shell + scrolling logic
- higher regression risk when changing popup appearance or scroll behavior

## What to build

### 1) `popup_pane` (shared outer popup container)

A reusable primitive that owns popup shell and scrolling viewport behavior.

Responsibilities:

- panel chrome: `background`, `border`, `radius`, `shadow`
- clipping/occlusion behavior
- width/height constraints
- scroll viewport and scrollbar integration
- optional fixed slots for structured layouts:
  - `header` (e.g. search field)
  - `content` (scrollable area)
  - optional `footer`

Non-goals:

- row semantics (hover, highlighted, selected) for list items
- domain-specific filtering logic

### 2) `popup_list` (shared list behavior/content layer)

A reusable list-oriented content renderer to place inside `popup_pane`.

Responsibilities:

- item row rendering via item template
- interactive handlers (hover/click)
- highlighted/active/selected row state presentation
- list-empty fallback rendering hooks

Future extension (not required for first pass):

- richer node model (`option`, `category`, `separator`, etc.)

## How to make it better (implementation strategy)

## Phase 1 (stabilize shared shell)

1. Introduce `popup_pane` under `crates/sdk/src/controls/`.
2. Move shared popup shell appearance wiring into `popup_pane`.
3. Route existing popup controls to render their current content inside `popup_pane` with no behavior change.
4. Keep public APIs minimally changed (prefer additive API).

Success criteria:

- same visuals as before for selector/autocomplete/combobox/search-selector/popup-menu
- no duplicated shell-border-shadow blocks in control templates

## Phase 2 (unify list content path)

1. Introduce `popup_list` abstraction with current list row capabilities.
2. Adapt `Selector`, `Autocomplete`, `ComboBox`, `SearchSelector` to use it.
3. Keep control-specific behavior logic (querying/filtering/open-close) in their controls.
4. Ensure template pane previews are faithful to runtime composition.

Success criteria:

- one shared row/list behavior pipeline
- consistent highlighted/hovered/selected semantics
- reduced per-control duplicate template code

## Phase 3 (enhancements)

1. Add richer list nodes (`category`, `separator`, etc.) without breaking existing option-only users.
2. Add explicit keyboard navigation policy for non-selectable nodes.
3. Add empty/loading/error content slots as needed.

## Design constraints

- Preserve current error-handling style; do not introduce panics in library code.
- Keep public API changes minimal and documented.
- Prefer explicit, readable types.
- Use small, reviewable diffs.

## Validation checklist

- `cargo fmt --all`
- `cargo check` (workspace)
- `cargo clippy --workspace --all-targets` (as feasible)
- targeted gallery verification:
  - Selector templates pane
  - Autocomplete pane
  - ComboBox pane
  - SearchSelector pane
  - PopupMenu pane

## Notes for docs updates after implementation

When architecture/API changes land, update:

- `docs/ai/architecture.md` (popup layering and ownership)
- `docs/ai/module-map.md` (new modules and ownership boundaries)
