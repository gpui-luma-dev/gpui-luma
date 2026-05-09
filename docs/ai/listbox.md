# ListBox Implementation Plan

This document defines the implementation plan for a new `ListBox` control in the SDK.

It is intentionally aligned with:

- `docs/ai/project-overview.md` (workspace scope, module conventions, crate boundaries)
- `docs/ai/architecture.md` (control layering, template/theme composition, event model)

## Goal

Create a selector-style `ListBox` control that initially renders as a static panel/list with selectable rows.

Initial feature set:

- single-select and multi-select modes
- presenter-driven row content (`ControlPresenter<M>`) so item visuals can be plain text, checkbox-like, icon+text, or fully custom
- keyboard and pointer interaction
- themable default template

Out of scope for initial milestone: popup behavior, search/filter, virtualization, async providers.

## Architectural Requirements

Follow the established control module pattern documented in `docs/ai/architecture.md`:

- `model.rs` – builder + immutable model + render payloads
- `control.rs` – runtime state, event wiring, interaction behavior
- `template.rs` – render contract and handler interfaces
- `theme.rs` – appearance structs, default theme, theme traits
- `mod.rs` – curated public exports and convenience constructor

Integrate in SDK controls root described in `docs/ai/project-overview.md`:

- add `listbox` module under `crates/sdk/src/controls/`
- export from `crates/sdk/src/controls/mod.rs`

## Proposed Public API

- `listbox::new(id, items) -> ListBoxBuilder`
- `ListBoxSelectionMode`:
  - `Single`
  - `Multiple`
- `ListBoxItem`:
  - `id`, `label`, `value`, `enabled`
- `ListBoxEvent`:
  - `Change { selected_ids }`
  - optional follow-up: `Activate { id }`

Builder shape (initial):

- `.selection_mode(...)`
- `.selected(...)` / `.selected_ids(...)`
- `.template(...)`
- `.theme(...)` (or theme-specific builder methods)
- `.content(...)` via `HasPresenter`
- `.spawn(cx)`

Optional but recommended in v1 if low effort:

- managed vs unmanaged state mode parity with choice-style controls

## Data Model and Presenter Contract

Define in `model.rs`:

- `ListBoxModel`
- `ListBoxRenderModel`
- `ListBoxItemContentModel` (input to row presenter)
- `type ListBoxContent = ControlPresenter<ListBoxItemContentModel>`

`ListBoxItemContentModel` should include at least:

- identity: `listbox_id`, `item_id`, `item_label`, `item_value`
- interaction state: `selected`, `focused`, `hovered`, `pressed`, `enabled`
- positioning: `index`, `sibling_count` (optional)
- mode metadata: `selection_mode`

Trait bridge:

- `impl HasPresenter<ListBoxItemContentModel> for ListBoxBuilder`

This provides `.content(...)` and `.label(...)` ergonomics automatically.

## Interaction and Selection Behavior

Implement deterministic selection behavior in `control.rs`:

- pointer:
  - click row in `Single` mode selects exactly one
  - click row in `Multiple` mode toggles membership
- keyboard:
  - `Up` / `Down` move active/focused row
  - `Home` / `End` jump first/last enabled row
  - `Space` / `Enter` apply selection/toggle to focused row
- disabled rows are not interactive

Event emission:

- emit `ListBoxEvent::Change` when effective selection changes

Keep behavior logic independent from rendering details.

## Template Layer

In `template.rs` define:

- `trait ListBoxTemplate`
- `ListBoxTemplateHandlers` with row handlers (hover, mouse down/up, click, key down as needed)
- default template implementation that:
  - renders root container
  - renders rows with stateful visuals
  - invokes row presenter for row content

The template must remain lookless enough to be replaced but complete enough for first-party usage.

## Theme Layer

In `theme.rs` define:

- `ListBoxTheme` trait
- `ListBoxAppearance` (container + row appearance)
- `default_listbox_theme()`
- theme usage metadata for registry integration

Use existing palette tokens (`state.hover`, `state.selected`, `focus.ring`, form/border tokens) to align with existing controls.

## Control Wiring

In `control.rs` implement:

- `ListBoxControl` entity
- `EventEmitter<ListBoxEvent>`
- focus handling consistent with existing controls
- render-model construction + template invocation
- mutable APIs as needed (`set_items`, `set_selection_mode`, `set_selected_ids`, etc.)

Ensure update paths call `cx.notify()` and event paths use `cx.emit(...)` per project conventions.

## Gallery Integration Plan

Add a new gallery pane for immediate validation:

- `apps/gallery/src/gallery/panes/listbox/`
- examples:
  - single-select default rows
  - multi-select rows
  - custom presenter rows (e.g. checkbox-like / icon+label)

Wire into gallery registry/navigation modules.

## Incremental Task Breakdown

1. Create `controls/listbox/` module skeleton and exports.
2. Implement model types and builder surface.
3. Implement selection behavior/state in control runtime.
4. Implement template contract + default template.
5. Implement theme contract + default theme + usage metadata.
6. Hook control into SDK public exports.
7. Add gallery pane and register it.
8. Refine API names and ergonomics after first gallery pass.

## Deferred Backlog (Post-v1)

- shift-range selection semantics
- modifier-key additive policies
- typeahead within list
- list virtualization
- grouped/sectioned list items
- async-backed item providers

## Notes

- Keep diffs small and reviewable.
- Prefer direct module inspection and existing control patterns over speculative abstractions.
- This plan is intentionally consistent with workspace and architecture guidance in:
  - `docs/ai/project-overview.md`
  - `docs/ai/architecture.md`
