# SDK organization notes

The SDK’s LMTP split is solid. The weak point is that `controls/` is a flat catalog of ~40 siblings mixing primitives, specializations, shared engines, layout chrome, overlays, and a 58-file color domain. Architecture docs already group these families; the filesystem and public names do not.

## Highest-value naming fixes

These confuse callers today and are cheap to alias:

| Current | Problem | Prefer |
|---|---|---|
| `command::button` | Buttons live under a key-profile name. Callers write `controls::command::button::Button`. | `button` / `icon_button`; keep `command` as internal keyboard/core |
| `button_family` | Theme recipe, not a control | `button::family` or `theme::button_family` |
| `button_group` → `IconGroup*` | Module, look factory (`button_group()`), and types disagree | Pick one: `icon_group` or `button_group` end-to-end |
| `tabs_navigation` | Verbose vs `tree_view` / `list_view` | `tabs` (`Tabs`, `TabsBuilder`) |
| `AutocompleteTextBox*` | Module is `autocomplete` | `Autocomplete` |
| `ToolbarControl = Entity<Toolbar>` | Inverted vs `Checkbox = Entity<CheckboxControl>` | `Toolbar = Entity<ToolbarControl>` |
| `SidebarControlEntity` | Third name besides `SidebarControl` / `sidebar()` | Drop the alias |
| `overlay_window` internals `Dialog*` | Leftover rename | Rename internals to `OverlayWindow*` |
| `SelectorState` / `CheckboxState` / … | All aliases of `InteractionState` | Stop aliasing; use `InteractionState` |
| `ScrollingListView` | Alias of `ListView` | Delete |
| `split_button.rs` | Type alias of `PopupMenu` pretending to be a control | Document as preset, or a real thin wrapper |
| `keyhandling` | Not snake_case | `key_handling` |
| `choice_indicator_layout` | Shadow-projection helpers, not choice layout | `shadow_layout` or fold into `theme::shadow` |
| `label.rs` | Only `field_label()`, not a Label control | `field_label` helper, not a control module |

### Worst cluster

`selector`, `selector_panel`, `selector_item_template`, `selection_panel`, `search_selector`.

- `selector` = popup dropdown
- `selector_panel` = shared item-list template used by selector/combobox/autocomplete
- `selector_item_template` = types that `selector_panel` re-exports
- `selection_panel` = a different list control
- `search_selector` = searchable selector

Suggested names:

- `selector` stays
- `selector_panel` → `selector_list` or `item_list` (shared list chrome, not a control)
- `selector_item_template.rs` folds into that module
- `selection_panel` → `option_list` / `choice_list` (or keep if it is the ARIA listbox-in-a-panel)
- `search_selector` → `searchable_selector` only if you keep the compound; otherwise leave it

## Split shared infra out of the control catalog

These are not controls, but they sit next to `checkbox` and `slider` in `controls/mod.rs`:

- **LMTP seams:** `template`, `state`, `presenter`, `interaction`, `value`
- **Motion/lifecycle:** `overlay_presence`, `popup_lifecycle` (crate-root `animation.rs` is a third home)
- **Menus:** `menu_item`, `menu_navigation`
- **Chrome helpers:** `icon`, `rounded_shell`, `popup_scroll_surface`, `choice_indicator_layout`

Move them to `sdk/src/infra/` (or `controls/infra/`) and re-export. `controls/` should only declare spawnable families.

Crate-root motion is split the same way: `VisualTransition` at root, overlay enter/exit under controls. One `motion/` module (transition + presence + popup lifecycle) would match how templates actually use them.

## Group families in the tree (keep old paths as `pub use`)

`controls/mod.rs` is unordered and ungrouped. A filesystem that matches `docs/architecture.md`:

```text
controls/
  button/          button, icon_button, split_button, button_family
  choice/          checkbox, radio_button, switch, toggle, control_group
                   + thin presets: listbox, radio_group, button_group, toolbar
  text/            textfield, textarea, shared editing (already crate-private `text/`)
  overlay/         popup_menu, context_menu, floating_menu, popover_button,
                   overlay_window, slide_panel
  selection/       selector, combobox, autocomplete, search_selector,
                   list_view, selection_panel  (+ shared selector_list)
  navigation/      tabs, sidebar, accordion, tree_view, pager, stepper
  layout/          split_view, resizable_panels, dock_splitter,
                   scrollbar, scroll_container
  range/           slider, progress
  color/           leave, then extract (already in the roadmap)
```

Do this with compatibility re-exports (`pub use controls::button as command` if needed). Don’t break `gpui_luma::controls::…` in the same change as behavior.

## Three different “layout” namespaces

| Path | What it is |
|---|---|
| `src/layout.rs` + `src/layouts/` | `DockPanel`, `GridLayout`, `LayerStack` |
| `src/theme/layout.rs` | `StandardBoxScale`, `ListRowScale` |
| `controls/split_view`, `dock_splitter`, `resizable_panels` | Interactive layout controls |

`layout.rs` using `#[path = "layouts/dock_panel.rs"]` is also odd — prefer `mod layouts` and re-export.

Interactive splitters belong next to each other; primitive stacks belong under one `layout/` module. Theme scales should not be named `layout`.

`shell/` (title bar only) is a fourth chrome home. Fine if it stays window chrome; don’t grow it into another control dump.

## LMTP is the convention — several modules ignore it

Most controls are `model` / `control` / `template` / `theme`. Exceptions that make the crate harder to navigate:

- **`popover_button/mod.rs`:** large file with nested `mod panel { … }` instead of files
- **`radio_group`, `button_group`:** logic in `mod.rs`
- **`scroll_container.rs`:** 500+ line single file
- **`floating_menu`:** `state.rs` + templates, no entity `control.rs` (a renderer, named like a control)
- **`slide_panel`:** `handle` / `overlay` / `resize` / `state` — different vocabulary
- **`listbox`:** preset over `control_group` (good), but looks like a full control in the catalog

New controls should follow LMTP. These exceptions are the ones worth splitting when touched.

## Color is a product system inside the primitive crate

`controls/color/` is ~58 files with a different vocabulary (`delegates`, `raster`, `visual`, `sync`, `track_context`). `color_ring` and `color_arc` are near-duplicates.

That matches the roadmap (`color-controls` later). Until extraction: keep it as one opt-in subtree, don’t let `color::style::Size` leak next to `ControlSize`, and don’t add more color types at the `controls/` root.

## Public surface is deeper than it needs to be

- `lib.rs` re-exports animation/layout/`init`, almost no controls — so every app learns `gpui_luma::controls::<family>::…`
- No prelude
- Shared types are imported from random depths (`presenter::HasPresenter`, `overlay_presence::…`, `menu_item::MenuItem`)

A small `gpui_luma::prelude` (builders, events, `ControlSize`, `InteractionState`, `IconSource`) plus keeping family modules would cut Studio/look import noise without flattening everything.

`theme/tokens.rs` (~700 lines) is the other kitchen-sink: palettes, metrics, typography, elevation. Splitting `palette` / `metrics` / `typography` would match how looks actually consume them.

## Suggested order (no big-bang move)

1. Rename/alias the collisions (`command/button`, `IconGroup`, `tabs_navigation`, `AutocompleteTextBox`, toolbar type inversion, `Dialog*` internals).
2. Park infra (`template`, `state`, `presenter`, motion, menu helpers) out of the control list.
3. Collapse selector shared code into one module; keep `selector` / `selection_panel` as the two public controls.
4. Family folders with `pub use` shims.
5. Leave color and graphing until the planned crates exist.

The LMTP contract and lookless SDK split don’t need to change. The catalog just needs names that match what things are, and a tree that matches the families you already documented.
