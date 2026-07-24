# TabsNavigation Phase 2 Redesign

## Status

Design note for the next SDK refactor. This should be handled before public release if possible, because `TabsNavigation` is a core SDK surface and recent Luma Studio work exposed that it is too narrow.

## Problem Summary

The Luma Studio Controls tab now needs to behave as both:

- a selected tab that shows the Controls content area
- a dropdown trigger that opens a picker for which control exposition to show

The pending app implementation already proves this shape, but it is buggy because the dropdown trigger and popup lifecycle are app-local. Click handling, outside-click dismissal, focus transfer, focus-loss dismissal, Escape, and open-state synchronization are too easy to get wrong when every caller builds the popup mechanics manually.

The implementation had to solve this by customizing the tab template, special-casing the `controls` tab, manually rendering a chevron, capturing tab bounds with `on_prepaint`, syncing open state through app-local shared state, and building a bespoke anchored popup.

That is the wrong long-term shape. It means the app had to invent interactive chrome that should belong to the SDK.

## Immediate Bug-Fix Goal

Before or alongside the larger refactor, fix the current Controls-tab dropdown so the product does not carry known interaction bugs while the SDK redesign is underway.

Current bugs/risks to address:

- leaving the Controls tab while the picker is open must close the picker and reset the tab open visual
- re-entering Controls after leaving another tab must not invert the intended open/closed state
- clicking the already-selected Controls tab should deterministically open or toggle the picker according to documented behavior
- click-away should close the picker without racing the original trigger click
- focus should move into the picker only after the popup is actually rendered and measurable
- focus-loss dismissal should not immediately close the picker on the same interaction that opened it
- Escape should close the picker and restore a sensible focus target
- selecting an item should close the picker, update selected exposition, and reset scroll
- clicking another top-level tab should close the picker before switching content

This bug-fix pass may remain app-local if needed, but it should be treated as a temporary stabilization layer. The SDK redesign below should remove the need for this custom lifecycle code.

## Current TabsNavigation Shape

`TabsNavigation` already delegates to `ControlGroupControl<TabsNavigationItem>` for the important interaction model:

- single required selection
- horizontal layout
- selection follows active item
- item hover/press state through control-group handlers
- focus and keyboard handling through the group
- `Change`, `Activate`, `FocusChanged`, and `ItemFocused` events

But `TabsNavigationItem` is still too thin:

```rust
id
label
enabled
```

The default tab template renders custom tab `Div`s directly. It does not reuse `Toggle` or button-family item chrome, and it has no first-class item accessory, disclosure state, or anchor reporting.

The result is a half-refactor:

```text
TabsNavigation
  -> ControlGroup selection/focus engine
  -> bespoke tabs item visual
```

The desired model is stronger:

```text
TabsNavigation
  -> ControlGroup of controlled Toggle-style items
  -> tabs-specific theme/template defaults
```

## Design Direction

### 1. Tabs Are Controlled Toggle Items

Tabs should be treated as toggle-button-like controls in a single-required group.

The group owns:

- selected item
- roving/current item
- keyboard navigation
- activation semantics
- focus state

The item chrome owns:

- label/icon/accessory layout
- selected visual state
- hover/pressed/focused/disabled visuals
- theme-resolved button/toggle look

There must be one selection owner. If real `Toggle` entities are hosted inside a group, they should operate in a controlled mode:

- selected state comes from `ControlGroup`
- click/keyboard activation asks the group to select or activate the item
- app observes group/tabs events, not per-toggle local state

This avoids two independent selection engines.

### 2. Prefer Reusing Toggle/Button-Family Logic

The per-item tab renderer should not duplicate toggle/button behavior. It should reuse the same button-family or toggle visual logic for:

- interaction state resolution
- selected state
- disabled treatment
- focus adorners
- sizing
- label plus leading/trailing accessory layout
- theme policy

This does not require spawning a `Toggle` entity per tab if that is awkward, but the behavior and rendering path should be equivalent to a controlled toggle item.

Actual hosted `Toggle` entities are acceptable if the ownership contract is clear. Entity count is not the limiting concern for tabs.

### 3. Item Accessories Are First-Class

`TabsNavigationItem` should support common item chrome without replacing the whole tab template:

```rust
TabsNavigationItem::new("controls")
    .label("Controls")
    .trailing_icon(LucideIcon::ChevronDown)
```

Likely model:

```rust
pub struct TabsNavigationItem {
    id: SharedString,
    label: SharedString,
    enabled: bool,
    leading: Option<TabItemAccessory>,
    trailing: Option<TabItemAccessory>,
    trigger_kind: TabTriggerKind,
}
```

Useful accessory variants:

```rust
pub enum TabItemAccessory {
    Icon(LucideIcon),
    Disclosure,
    Custom(TabItemAccessoryRenderer),
}
```

The default template/theme should own spacing, icon size, color, and width contribution. Apps should not copy `resolve_uniform_tab_width()` just to account for a chevron.

### 4. Dropdown Trigger Is An Item Capability

The Controls tab use case should be expressible as SDK configuration:

```rust
TabsNavigation::new("content-tabs")
    .item(TabsNavigationItem::new("cards").label("Cards"))
    .item(
        TabsNavigationItem::new("controls")
            .label("Controls")
            .trigger_kind(TabTriggerKind::Dropdown)
            .trailing_accessory(TabItemAccessory::Disclosure),
    )
```

`TabsNavigation` should not own the Controls picker. It should own the fact that a tab item can request a dropdown and provide the data needed to anchor it.

### 5. Explicit Reactivation / Trigger Events

The app should not infer reactivation by comparing its own `active_tab` against the event payload.

Add either:

```rust
TabsNavigationEvent::Reactivate {
    tab_id: SharedString,
    label: SharedString,
}
```

or enrich `Activate`:

```rust
TabsNavigationEvent::Activate {
    tab_id: SharedString,
    label: SharedString,
    was_selected: bool,
}
```

For dropdown-capable tabs, consider a more direct semantic event:

```rust
TabsNavigationEvent::DropdownRequested {
    tab_id: SharedString,
    label: SharedString,
    anchor: ItemAnchor,
}
```

This lets callers wire:

```rust
TabsNavigationEvent::DropdownRequested { tab_id, anchor, .. } => {
    picker.open_at(anchor, cx);
}
```

### 6. Item Anchor Reporting

Dropdowns/popovers need item geometry. Apps should not use `on_prepaint` in custom tab templates to capture bounds.

Add an SDK-provided anchor mechanism. Possible shapes:

```rust
pub struct ItemAnchor {
    pub item_id: SharedString,
    pub bounds: Bounds<Pixels>,
}
```

Events:

```rust
TabsNavigationEvent::ItemBoundsChanged {
    tab_id: SharedString,
    bounds: Bounds<Pixels>,
}
```

or include anchor data only on trigger events:

```rust
TabsNavigationEvent::DropdownRequested {
    tab_id: SharedString,
    label: SharedString,
    anchor: ItemAnchor,
}
```

The second option is smaller and better for this use case.

### 7. Controlled Open Visual State

The tab item needs an externally controlled open/closed visual state so the chevron and selected styling reflect the paired popup.

Possible API:

```rust
tabs.set_item_open("controls", true, cx);
```

or item model update:

```rust
tabs.set_item_accessory_state("controls", TabItemAccessoryState::Open, cx);
```

This open state is visual. The popup/dropdown control still owns dismissal and lifecycle.

## Related SDK Gap: DropdownPanel / Popover

This redesign should pair with, or at least leave room for, a generic anchored panel primitive.

The current app code needed:

- open/close state
- trigger bounds
- measurement before final placement
- snap-to-window
- click-away dismissal
- focus-loss dismissal
- Escape dismissal
- focus restore
- floating-menu look/elevation
- theme invalidation

Those are generic popup concerns, not Luma Studio concerns.

Target primitive:

```rust
DropdownPanel::new("controls-picker")
    .anchor(anchor)
    .open(open)
    .content(|model, cx| render_picker(model, cx))
    .spawn(cx)
```

`TabsNavigation` should produce the anchor and trigger events. `DropdownPanel` should own popup lifecycle.

### PopupMenu Enhancement vs New DropdownPanel

There are two viable SDK paths:

1. Enhance `PopupMenu` so it can use an external trigger/anchor and arbitrary panel content.
2. Add a sibling `DropdownPanel` / `Popover` primitive for arbitrary anchored content, leaving `PopupMenu` as the menu-item-specific control.

The current `PopupMenu` is too specialized for this use case because it owns both:

- its own trigger button
- a `MenuItem` / `FloatingMenu` content model

The Controls picker needs:

- trigger supplied by `TabsNavigation`
- anchor supplied by the selected tab item
- rich multi-column content, not only `MenuItem` rows
- controlled open visual state on the tab item
- popup lifecycle owned by a reusable overlay primitive

If `PopupMenu` is enhanced, it should grow these capabilities without weakening the simple menu-button case:

```rust
PopupMenu::new("controls-picker")
    .trigger(PopupMenuTrigger::External(anchor))
    .content(|model, cx| render_picker(model, cx))
    .dismiss_policy(PopupDismissPolicy::ClickAwayOrFocusLoss)
```

If a new primitive is introduced, prefer keeping `PopupMenu` as a convenience wrapper over it:

```text
DropdownPanel / Popover
  -> arbitrary anchored content and dismissal lifecycle

PopupMenu
  -> button trigger + MenuItem model + FloatingMenu content
  -> internally uses DropdownPanel / Popover for placement and dismissal
```

The second path is cleaner if we expect more anchored custom panels beyond menus.

## Acceptance Criteria

The phase is not complete unless the Luma Studio Controls-tab trigger can be rewritten without:

- custom tab template solely for the caret/dropdown target
- hardcoded `"controls"` logic inside a tab renderer
- app-local `ControlsTabChrome`
- app-local `on_prepaint` bounds capture
- manual chevron width math
- raw dropdown-trigger tab `Div`
- popup open visual state stored separately from the SDK tab item

The desired app-level shape should be close to:

```rust
let tabs = look
    .tabs_navigation("luma-studio-content-tabs")
    .item(TabsNavigationItem::new("cards").label("Cards"))
    .item(
        TabsNavigationItem::new("controls")
            .label("Controls")
            .trigger_kind(TabTriggerKind::Dropdown)
            .trailing_accessory(TabItemAccessory::Disclosure),
    )
    .spawn(cx);

cx.subscribe(&tabs, |host, _, event, cx| match event {
    TabsNavigationEvent::Change { tab_id, .. } => {
        host.set_active_tab(tab_id, cx);
    }
    TabsNavigationEvent::DropdownRequested { tab_id, anchor, .. } => {
        host.open_tab_dropdown(tab_id, anchor, cx);
    }
    _ => {}
});
```

## Migration Plan

### Phase 0: Stabilize Current Controls Dropdown

Fix current user-facing dropdown bugs before the deeper SDK migration.

Steps:

1. Close the picker when switching away from Controls.
2. Keep tab open visual state synchronized with actual popup render state.
3. Make reactivation/toggle behavior deterministic.
4. Remove or minimize focus/dismiss guard races.
5. Verify click-away, Escape, tab switching, item selection, and focus restore.

Acceptance criteria:

- no stale open chevron when the picker is not rendered
- returning to Controls does not accidentally close a picker that should open
- click-away and Escape reliably close the picker
- switching top-level tabs closes the picker
- item selection closes the picker and updates the exposition

### Phase 1: Clarify Group Semantics

Document and verify these terms in `control_group` and `tabs_navigation`:

- selected item: committed selection
- current/roving item: keyboard navigation target
- activated item: item invocation
- reactivated item: invocation of the already selected item

Avoid using `active` to mean both selected and current.

### Phase 2: Controlled Toggle Item Path

Add a reusable controlled-toggle item path for groups.

Options:

1. Host actual `Toggle` entities in `ControlGroup`.
2. Add a `ToggleLikeGroupItem` adapter that uses toggle/button-family rendering without spawning child entities.

Either is acceptable if:

- `ControlGroup` remains the source of truth for selection
- toggle visuals come from SDK toggle/button-family logic
- item activation flows through group events
- keyboard behavior is inherited from `ControlGroup`

### Phase 3: TabsNavigation Item Model Expansion

Extend `TabsNavigationItem` with:

- leading accessory
- trailing accessory
- trigger kind
- controlled open/disclosure state

Preserve minimal API compatibility for existing callers.

### Phase 4: TabsNavigation On Controlled Toggle Items

Rebuild default `TabsNavigation` item rendering on the controlled toggle item path.

Preserve:

- `TabsNavigationWidthMode::Intrinsic`
- `TabsNavigationWidthMode::Uniform`
- existing theme output where no accessories are configured
- current keyboard navigation behavior unless intentionally corrected and documented
- existing `TabsNavigationEvent::Change` behavior

Add:

- reactivation semantics
- dropdown trigger event
- item anchor reporting for trigger events

### Phase 5: DropdownPanel / Popover Primitive

Add a generic anchored panel primitive, or generalize `PopupMenu` into a sibling that supports arbitrary content.

Keep responsibilities separate:

- `TabsNavigation`: selected tab, activation, dropdown trigger intent, anchor data, open visual state
- `DropdownPanel`: open lifecycle, placement, focus/dismiss behavior, content host
- app: domain decision about which picker/content to show

### Phase 6: Luma Studio Cleanup

Rewrite the Controls tab picker integration using the new SDK surface.

Remove:

- `ControlsTabChrome`
- custom Controls-tab chevron rendering
- copied uniform-width chevron math
- app-owned trigger bounds capture
- bespoke popup focus/dismiss lifecycle, if `DropdownPanel` exists

## Testing Plan

Add or update focused tests/examples for:

- plain tabs remain visually unchanged
- uniform tabs include accessory width correctly
- selected tab state maps from group selection
- already selected tab activation emits reactivation or `was_selected`
- dropdown tab activation emits an anchor-bearing event
- controlled open state flips disclosure icon without changing selection
- keyboard navigation still works for tab groups
- Escape/click-away behavior works through `DropdownPanel` once introduced

Run at minimum:

```sh
cargo check -p gpui-luma-gallery
cargo check -p luma-studio
cargo clippy -p gpui-luma-gallery -- -D warnings
```

## Non-Goals

- `TabsNavigation` should not own arbitrary popup content.
- The Controls catalog picker should not become a special SDK case.
- This should not force every tab item to be a dropdown-capable item.
- This should not introduce a second selection owner beside `ControlGroup`.

## Design Principle

Tabs, segmented controls, toolbar toggles, and dropdown-capable tab triggers should share the same SDK button/toggle foundation. Specialized controls should be presets over common group and item primitives, not separate islands of bespoke interactive chrome.
