# Control Group Next Steps

## Context

`control_group` now has a useful separation between behavior, item visuals, and item layout:

- group behavior: focus handle, roving active item, pointer state, selection state, keyboard actions
- item visual: `item_element_template(...)`
- caller layout: `with_item_layout(...)`

This direction is correct for grouped controls and is a stepping stone toward toolbar-style composition. The next refactor should consolidate other grouped controls that still duplicate this machinery, especially `listbox` and `tabs_navigation`.

The end goal is WPF-like grouping:

- an item collection
- an item element/template
- an items layout/panel
- selection or activation behavior layered on top
- specialized controls like listbox, tabs, radio group, toggle group, and toolbar built from the same primitives

## Current State

### `control_group`

`control_group` owns the best shared behavior today:

- normalized selected IDs
- active item tracking
- hover/press state
- item handlers
- focus handling
- TabList key profile actions
- single-required, single-allow-none, and multiple selection modes
- item element and item layout seams

It is still selection-centric. That is fine for radio groups, toggle groups, listboxes, and tabs, but it is not yet a complete toolbar foundation because toolbars need mixed command/toggle/menu/separator items.

### `listbox`

`listbox` already uses `ControlGroupControl<ListBoxItem>`, so behavior is shared. Its template still owns the list shell and row layout directly.

That is reasonable for the default listbox, because listbox rows need:

- list surface styling
- row metrics
- focus adorners
- disabled/selected/pressed/hover states
- clipping and padding rules

However, it should still move toward the item element model so row visuals and list layout are expressed through the same framework instead of a bespoke template loop.

### `tabs_navigation`

`tabs_navigation` currently duplicates most of `control_group`:

- `active_id`
- hover/press state
- focus handle
- item handler vectors
- active/selected render state
- next/previous/first/last enabled item navigation
- click/keyboard activation
- template-level handler unpacking

This is the clearest refactor target. Tabs are a single-selected control group with tab-specific item visuals and activation events.

## Problem To Resolve: Selected Item vs Active Item

The codebase currently uses overloaded language:

- `control_group.active_id` means the roving focused item inside the group.
- `control_group.selected_ids` means committed selection.
- `tabs_navigation.active_id` means selected/active tab, not merely roving focus.
- rendered item state uses `selected`, `active`, and `focus_visible`, but each control interprets these slightly differently.

This needs to be made explicit before moving tabs and listbox deeper into the shared framework.

Recommended vocabulary:

- **Focused group**: the group root has focus.
- **Roving item / current item**: the item that receives keyboard focus indication inside the group.
- **Selected item(s)**: committed selection state.
- **Activated item**: command/event emitted by activating an item.
- **Current selected item**: for single-required controls, selected and current are often synchronized but should not be treated as the same concept in the shared model.

Recommended model adjustment:

- Keep `selected_ids` as committed selection.
- Rename or conceptually document `active_id` in `control_group` as current/roving item.
- Tabs should use single-required selection for the selected tab.
- Tabs should emit an activation event when selection changes or when the current selected tab is explicitly activated, preserving current behavior if needed.

Do not let tabs redefine `active_id` as selected state if they are rebuilt on `control_group`.

## Target Architecture

### Short Term

Keep `control_group` as the selection-group foundation.

Use it for:

- radio group
- toggle/button group
- listbox
- tabs navigation

Expose these seams consistently:

- `item_template(...)`: content inside an item
- `item_element_template(...)`: full item chrome/visual
- `with_item_layout(...)`: caller or default template arranges SDK-wired item elements

### Medium Term

After listbox and tabs converge, extract a lower-level primitive if the shared surface is still too selection-specific.

Possible future layering:

- `items_control`: collection, item element generation, item layout
- `composite_control`: roving focus and current item
- `selection_control`: selected IDs and selection modes
- `control_group`: selection-control convenience for grouped controls
- `toolbar`: composite-control with mixed item roles, not necessarily selection-control

Do not introduce this lower layer until listbox and tabs migrations prove the exact shared boundaries.

## Migration Plan

### Phase 1: Listbox Alignment

Goal: keep listbox behavior unchanged while expressing listbox rows as item elements.

Steps:

1. Create a listbox row item element template.
   - It should render the existing row chrome currently produced by `render_listbox_row_visual`.
   - It should preserve row metrics, focus adorners, selected/hover/pressed states, disabled opacity, and content handling.

2. Keep the listbox shell as a layout/template concern.
   - The shell still owns list background, border, padding, row gap, overflow, and list focus adorner.
   - The shell should arrange `ControlGroupItemElements` vertically.

3. Ensure `listbox::new`, `listbox::single`, and `listbox::multiple` still return `ControlGroupBuilder<ListBoxItem>`.

4. Avoid broad API changes unless required.
   - Existing listbox callers should not need to change.
   - Existing item template usage should still customize row content, not row behavior.

Acceptance criteria:

- Visual output is unchanged.
- Keyboard navigation and selection behavior are unchanged.
- Existing Gallery listbox examples work.
- No listbox template manually unpacks handler vectors.
- `cargo check -p gpui-luma-gallery`
- relevant listbox tests or Gallery smoke tests pass.

### Phase 2: Tabs Navigation On Control Group

Goal: remove duplicated focus/selection machinery from `tabs_navigation`.

Steps:

1. Make `TabsNavigationItem` implement or map to `ControlGroupItemLike`.

2. Decide the selected/current semantics before editing:
   - selected tab should map to `selected_ids[0]`
   - roving/current item should use control-group current item behavior
   - activation event should preserve existing `TabsNavigationEvent::Activate { tab_id, label }`

3. Rebuild `TabsNavigation` as a thin wrapper around `ControlGroupControl<TabsNavigationItem>` or convert its internals to delegate to `control_group`.

4. Move tab visual rendering into an item element template.
   - Preserve `TabsNavigationWidthMode::Intrinsic` and `Uniform`.
   - Preserve themed tab item visuals and list chrome.

5. Move tab list arrangement into item layout.
   - Default tab layout should be a horizontal row with existing gap/padding/radius/background/border.

6. Remove duplicated handler-vector and roving-navigation code from `tabs_navigation`.

Acceptance criteria:

- Existing Tabs Navigation Gallery examples look unchanged.
- Arrow/Home/End/Space/Enter behavior remains unchanged or is intentionally documented if corrected.
- `TabsNavigationEvent::Activate` remains compatible.
- No tabs template manually unpacks handler vectors.
- `cargo check -p gpui-luma-gallery`
- `cargo clippy -p gpui-luma-gallery -- -D warnings`

### Phase 3: Public API Review

After listbox and tabs use the same framework, review public naming:

- `ControlGroupItemElementTemplate`
- `ControlGroupItemElement`
- `ControlGroupItemElements`
- `with_item_layout`
- `item_element_template`

Decide whether these are general enough for toolbar work. If they feel selection-specific, stop and extract a lower-level `items_control` or `composite_control`.

### Phase 4: Toolbar Design

Only after tabs/listbox are aligned, revisit toolbar.

Toolbar likely needs:

- roving focus without required selection
- command items
- toggle items
- radio subsets
- separators
- menu buttons
- arbitrary hosted controls
- overflow behavior eventually

The toolbar should reuse item layout and item element generation, but it should not be forced into `selected_ids` unless the toolbar item role is actually selectable.

## Risks

- Collapsing tabs into `control_group` too quickly could blur selected vs current item semantics.
- Listbox has richer row metrics and adorners than simple radio/toggle groups; keep its shell specialized.
- Toolbar needs mixed roles; do not overfit `control_group` to toolbar before extracting the lower-level pieces.
- Public API churn should happen before external adoption, not after toolbar work starts.

## Suggested First Task

Start with tabs, not listbox.

Tabs duplicate the most control-group behavior and are semantically close to single-required selection. Migrating tabs will force the selected/current item decision while the visual scope is still manageable.

Then migrate listbox internals for consistency.
