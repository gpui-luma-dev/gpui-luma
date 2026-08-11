# Control Group Focus Strategy

## Context

`control_group` is becoming the shared foundation for grouped SDK controls such as radio groups, listbox, tabs navigation, and toolbar-like composites.

The current implementation has one implicit focus model:

- the group owns a single `FocusHandle`
- `Tab` enters/leaves the group as one composite control
- arrow keys move the group's current/active item
- templates receive `CompositeItemState` and paint active/focus-visible item visuals

That model works well for controls whose items are rendered directly by the group template, such as tabs, radio groups, listbox rows, and toggle groups.

It is not sufficient for composites that host real SDK controls as children. A toolbar may contain a `Button`, `Selector`, `PopupMenu`, `TextField`, color picker, or another future control. Those hosted controls may have their own `FocusHandle`, key context, activation behavior, popup state, and arrow-key needs.

The SDK should not solve this by defining a broad up-front `Control` trait or by making toolbar enumerate concrete item kinds. The reusable problem is narrower:

`control_group` needs an explicit focus strategy layer.

## Problem

`control_group` currently mixes these concerns inside one behavior path:

- group focus ownership
- current/active item tracking
- keyboard navigation
- focus-visible render state
- pointer state
- selection state
- activation events

This works for active-descendant style controls where items are visual descendants.

It breaks down when a group item hosts an independently focusable control:

- moving `active_id` does not focus the hosted child
- pressing `Enter` or `Space` activates the group item, not necessarily the child control
- hosted controls may stop pointer event propagation before the group wrapper sees it
- popup/selector controls may need arrow keys while open
- text inputs need left/right arrows for caret movement when focused

This is a control-group focus architecture issue, not only a toolbar issue.

## Design Direction

Extract an explicit focus strategy for `control_group`.

The first two strategies should be:

### Active Descendant

This is the current behavior.

- the group root owns focus
- item focus is represented by current/active item state
- templates paint focus-visible and active visuals
- arrow keys move current/active item
- item elements do not need their own focus handles

Use for:

- radio groups
- tabs navigation
- listbox
- toggle groups
- selection controls whose items are rendered directly by the template

### Roving Item Focus

This is needed for composites that host independently focusable controls.

- the group still owns item order and navigation
- each item may optionally provide a focus target
- arrow keys move current/active item through `control_group`
- when the current item changes, the focus strategy may move actual GPUI focus to the item's focus target
- children that need to own arrow keys while focused must be able to opt into that behavior

Use for:

- toolbar
- ribbon-like control groups
- property rows with embedded controls
- mixed command/input composites

## Proposed API Shape

Do not define a broad SDK `Control` trait yet.

Instead, define small focus-specific types that can be attached to items or supplied by builders/templates.

Possible model:

```rust
pub enum ControlGroupFocusStrategy {
    ActiveDescendant,
    RovingItemFocus,
}

pub enum ControlGroupArrowPolicy {
    GroupOwns,
    ChildOwnsWhenFocused,
    ChildOwnsAlways,
}

pub struct ControlGroupFocusTarget {
    pub focus_handle: gpui::FocusHandle,
    pub arrow_policy: ControlGroupArrowPolicy,
}
```

`ControlGroupItemLike` should remain the minimal item identity/enabled contract:

```rust
fn id(&self) -> &SharedString;
fn label(&self) -> &SharedString;
fn is_enabled(&self) -> bool;
```

Focus target information should be additive and optional. It may be supplied through:

- an optional item method if the trait is expanded carefully
- a separate `ControlGroupFocusTargetProvider<T>`
- a builder method on `ControlGroupBuilder<T>`
- a template/adapter layer for controls like toolbar

Avoid making every SDK control implement a large common trait just to solve this.

## Control Group Changes

The control group should separate these responsibilities:

- selection model: selected IDs and selection modes
- current item model: active/current item and movement
- focus strategy: how current item maps to GPUI focus
- keyboard routing: when group handles arrows vs when focused child may own them
- activation: whether activation emits group events, delegates to a hosted child, or both

Likely changes:

- Add a focus strategy setting to `ControlGroupBuilder`.
- Preserve current behavior as the default `ActiveDescendant` strategy.
- Add optional item focus target plumbing for `RovingItemFocus`.
- Emit or internally handle current-item changes in one place.
- Keep pointer state and selection behavior unchanged unless the strategy requires focus transfer.
- Keep disabled items and separators out of navigation.

## Toolbar Implications

Toolbar should be revisited after this work.

The toolbar should either:

- use `ControlGroupControl<ToolbarItem>` with `RovingItemFocus`, or
- be a thin toolbar-specific wrapper that delegates navigation/current-item state to the same focus strategy machinery.

Toolbar should not:

- reimplement arrow traversal
- maintain a separate manual active-index loop
- predefine concrete child control kinds such as button/menu/selector/textfield
- force a broad SDK `Control` trait before the focus problem is understood

Toolbar items may still need toolbar-local styling specialization, but that is separate from focus strategy.

## Tasks

- [ ] Audit current `control_group` focus, active/current item, keyboard, pointer, and selection responsibilities.
- [ ] Name and document the existing behavior as `ActiveDescendant`.
- [ ] Design a `RovingItemFocus` strategy for items with optional GPUI focus targets.
- [ ] Decide how item focus targets are supplied without requiring a broad SDK `Control` trait.
- [ ] Define arrow-key ownership policy for hosted controls.
- [ ] Ensure `Tab` entry/exit remains predictable for both strategies.
- [ ] Preserve radio group, listbox, tabs, and current control-group behavior under the default strategy.
- [ ] Add tests or gallery verification for current-item movement, disabled-item skipping, separators, and focus-visible state.
- [ ] Revisit toolbar after the focus strategy exists.

## Acceptance Criteria

- Existing `control_group` consumers keep their current behavior by default.
- The current focus model is explicit and documented as active-descendant behavior.
- A second focus strategy exists or is fully designed for roving focus across hosted controls.
- The strategy can move focus to an item's focus target without the composite reimplementing navigation.
- Hosted controls can declare or be configured with arrow-key ownership behavior.
- The design does not require a broad universal SDK `Control` trait.
- Toolbar design can be resumed with a clear decision: use `control_group` directly with the new strategy or wrap the reusable strategy in toolbar-specific code.
