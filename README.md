# gpui-luma

> **Early version.** This project is pre-release / alpha. APIs, crate layout, and examples may change without notice.

GPUI component library. Requires a recent Rust stable toolchain (`rust-toolchain.toml` pins `stable`).

## Build

```bash
git clone https://github.com/scottcg/gpui-luma.git
cd gpui-luma
cargo build
```

Library crates: depend on **`gpui-luma`** (SDK + look-core facade), then **one** look — `gpui-luma-look-shadcn` *or* `gpui-luma-look-radix`. Optional: `gpui-luma-color`.

```toml
[dependencies]
luma = { package = "gpui-luma", git = "https://github.com/scottcg/gpui-luma", tag = "v0.1.0-alpha.1" }
luma-look-shadcn = { package = "gpui-luma-look-shadcn", git = "https://github.com/scottcg/gpui-luma", tag = "v0.1.0-alpha.1" }
```

Imports stay `use luma::…`.

Collection controls live in `luma::controls::collection`: `listbox` for flat
collections, `table` for tabular records, and `tree_view` for hierarchies. Value
pickers remain in `selection`; destination and section controls remain in
`navigation`. Short imports such as `luma::controls::table` still work, as do the
previous `selection::{table, listbox}` and `navigation::tree_view` paths. Studio's
Controls dropdown groups ListBox, Paging Table, Scrolling Table, and Tree View
under Collection.

TreeView supports validated construction with `.try_items(...)` and preserving
updates with `replace_items(items, cx)`. IDs must be unique across all loaded
descendants. Preserving updates retain eligible selection (including collapsed
nodes) and user expansion by ID; `try_set_items` validates a full state reset.
The legacy `.items(...)` and `set_items(...)` methods ignore invalid replacements.
Luma Studio → Controls → TreeView includes a state-update fixture for reordering,
reparenting, removal, and duplicate-ID rejection.

Shadcn TreeView includes a scrollbar driven by the virtualized list itself; SDK
builders opt in with `.scrollbar_template(...)`. Its `.scrollbar_visibility(...)`
uses `ScrollbarVisibility::{AutoHide, Hidden, AlwaysVisible}`. AutoHide is the
TreeView default: show while scrolling, then hide after 1.25 seconds idle. Hidden
removes the gutter without disabling scrolling. Controls → TreeView has buttons
for all three policies. `.require_focus_for_scroll(true)`
passes unfocused wheel input to the page and contains focused scrolling at the
endpoints. `.wrap_navigation(false)` bounds Up/Down navigation. Left/Right follow
eligible ancestors/direct children, and Home/End reveal rows without selecting
them. Studio enables the focus requirement and bounded navigation in Controls →
TreeView; hover scrolling and wrapping remain the library defaults.

TreeView's `.branch_content(...)` and `.leaf_content(...)` accept named functions
or owned UI closures inside the themed row shell. `.expand_on_row_click(false)`
separates disclosure from selection. Wrap embedded SDK controls with
`DragDropElementExt::drag_boundary()` to keep their clicks and drags independent.
Rows retain theme-defined heights; full custom templates remain available and
can opt into the new `render_node_with_content` seam.

Opt into single-node dragging with `.drag_drop(TreeViewDragDrop::new(scope, handler))`.
The handler receives a keyed `TreeDropProposal`: validate it, prepare both data
snapshots, apply preserving replacements, then report `committed` or `rejected`.
The host owns mutation. `capture_subtree_state` / `restore_subtree_state` transfer
selection and expansion across trees, including hidden selected descendants.
Controls → TreeView includes a two-workspace example with before/after/into moves,
root append, independent Info buttons, cancellation, and edge auto-scroll.

Add `.drag_selected(true)` to the `TreeViewDragDrop` configuration for grouped
moves. Dragging a selected row captures visible enabled selected roots in displayed
order, omitting descendants of other captured roots. Each root carries its entire
loaded subtree; separately hidden selected nodes stay behind. An unselected row
still drags alone. Hosts use `proposal.node_ids()` and
`capture_subtrees_state(proposal.node_ids())` to prepare and transfer the group in
one transaction. Validation checks every moved subtree for cycles and collisions.
Drag events include `node_ids`; the existing `node_id` denotes the first root.
Controls → TreeView → Move between workspaces enables Extended selection and
grouped dragging, with a group count in the preview and IDs in the enclosed log.
The host resolves before/after gaps before extraction, including a moving anchor,
so adjacent grouped drops can correctly report unchanged.

TreeView builders support `.filter(predicate)` and `.sort(compare)`; live controls
provide `set_filter`, `clear_filter`, `set_sort`, and `clear_sort`. Filtering checks
all loaded domain nodes and shows matches plus their ancestor paths. Required
ancestors stay open without changing saved user expansion; clearing the filter
restores it. A matching branch does not include unmatched descendants. Selection
survives filtering, and sorting is stable within each sibling group without
reordering `items()`. `visible_ids()` reports effective displayed preorder.
Data replacement reevaluates both callbacks; reapply them when captured state changes.
Filtered or sorted destinations accept Into/root drops only, with before/after
reordering disabled. Changing a projection invalidates existing drag proposals.
The two-workspace example includes name/detail search, sibling-order selection,
and visible/hidden-selection counts.

Live TreeViews expose `scroll_to`, `scroll_to_center`, and their `_smooth`
counterparts. Requests use node IDs, preserve selection/focus, and return whether
the target is displayed. Ordinary requests ignore collapsed, filtered or missing
nodes. `reveal_node` and `reveal_node_smooth` explicitly open loaded ancestors;
they never fetch data or override filtering. Positioning waits for layout, uses
measured bounds for final alignment, and clamps at the beginning/end of the tree.
Smooth movement uses the shared 200 ms cubic easing, with logical estimates for
unmeasured rows. Manual input, new requests, and data/projection/expansion changes
interrupt movement. Controls → TreeView → Positioning provides target and movement
selectors, Go, and a closed-folder example for testing these behaviors.

Opt into `.selection_mode(TreeViewSelectionMode::Extended)` for Ctrl/Cmd-click
toggles and Shift ranges in displayed preorder. Plain clicks replace selection;
Ctrl/Cmd+Shift adds a range. Disabled rows are skipped, and ranges never wrap.
Up/Down and Home/End move the active row; Shift also selects the range. Space
selects (Ctrl/Cmd toggles); Enter emits `NodeActivated` without expanding or
selecting. Ctrl/Cmd+A adds visible enabled rows while preserving hidden
selections; Ctrl/Cmd+Shift+A clears all selections. `select_all(cx)` and
`clear_selection(cx)` provide the same operations (`select_all` also supports
Multiple). Plain clicks and non-additive ranges replace hidden selections too.
`range_anchor_id()` exposes the stable anchor, reconciled on collapse, filtering,
replacement, and subtree transfer.

`.selection_policy(TreeViewSelectionPolicy { mode, toggle_off,
selection_follows_active })` adds optional plain-click deselection in Single and
Extended, and optional keyboard selection following in Single. Live controls
provide `selection_policy()` / `set_selection_policy(...)`, with a
`SelectionPolicyChanged` event. Mode changes to Single retain the selected active
row, then the first displayed selection, then a stable hidden selection; None
clears selection. Existing Single/Multiple defaults and their Enter/Space
expansion behavior remain unchanged. Controls → TreeView → Selection and ranges
includes modes, policy checkboxes, search, sort, selection actions, and event logging.

Controls → TreeView → Large tree provides 10,117 loaded nodes (100 folders of
100 documents plus a 16-level branch) with distant jumps, same-count sibling
reversal, filtering, and collapse/restore controls. Sample counts reports unique
rows constructed and total row-template calls since Reset counts, separately from
loaded/displayed counts. These are construction diagnostics, not native frame-rate
measurements. Source projection and the flat cache still traverse/clone displayed
data; row elements are virtualized. Simultaneous expansion/collapse transitions
affecting more than 256 displayed descendants settle before layout, avoiding
thousands of tiny animated rows defeating virtualization.

Shadcn themes load from strings with `ShadcnLook::from_css_str`. The bundled fallback
CSS is available as `luma_look_shadcn::FALLBACK_CSS`. The former `from_css_path` and
`from_css_path_with_stylesheet` helpers have been removed: applications own file
access and should enforce appropriate path and size restrictions before parsing.
For custom stylesheet TOML, use `StylesheetConfig::parse`, then
`ShadcnLook::from_css_str_with_stylesheet`. The string parsers do not impose size limits.

## Example programs

**Luma Studio** — Shadcn look workbench (control docs, theme inspection):

```bash
cargo run -p luma-studio
# or: just luma-studio
```

**Luma Radix Studio** — Radix look stub workbench:

```bash
cargo run -p luma-radix-studio
# or: just luma-radix
```

Release builds: `cargo run -p luma-studio --release` / `cargo run -p luma-radix-studio --release` (or `just luma-studio-rel` / `just luma-radix-rel`).

## Prebuilt executables

The [Build Luma Studio](https://github.com/scottcg/gpui-luma/actions/workflows/build.yml) workflow publishes macOS and Windows `luma-studio` binaries as Actions artifacts (download from a successful run).
