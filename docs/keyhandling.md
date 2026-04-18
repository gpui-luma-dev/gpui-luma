# Control Key Handling Design

## 1. Purpose

This document describes the intended keyboard command model for GPUI-Luma
controls. It complements `docs/focus-handling.md`.

The goal is to avoid one-off `on_key_down` handlers as the SDK grows. New
controls should have a predictable place to define:

- which physical keys activate or change the control,
- which typed commands the control handles,
- when a command is consumed,
- when a command propagates to an ancestor,
- when raw key events are still appropriate.

Focus traversal is not redefined here. `Tab`, `Shift-Tab`, and ordinary
`Escape` fallback behavior belong to `gpui_luma::focus` and the nearest
`.luma_focus_scope(...)`.

## 2. Design Principles

### 2.1 Physical Keys Bind To Typed Commands

Shared keyboard behavior should use GPUI actions rather than duplicated
keystroke string matching.

Preferred flow:

```text
physical key -> GPUI KeyBinding -> typed SDK action -> control handler
```

Avoid this shape for shared behavior:

```text
physical key -> per-control string match -> direct mutation
```

Raw key handlers are still valid when a control genuinely needs raw keystroke
details, text input, character insertion, platform-specific composition, or a
control-specific key that has not yet earned shared SDK treatment.

### 2.2 Focus Is The Outer Keyboard Layer

Focus traversal and Escape fallback are handled by `gpui_luma::focus`.

Controls should not bind or handle `Tab` by default. Controls that legitimately
own Tab behavior, such as a future text editor or grid, should handle
`NextFocus` and `PreviousFocus` only for their internal mode and call
`cx.propagate()` at boundaries.

Simple controls should not handle `EscapeFocus`. Composite controls should
handle it only when they have transient state to dismiss, then propagate when
there is nothing to dismiss.

### 2.3 Control Commands Are Semantic

Action names should describe user intent, not the key that caused it.

Good examples:

```rust
ActivateControl
OpenControl
CommitSelection
SelectNextItem
IncreaseValue
MoveToEnd
```

Avoid:

```rust
SpacePressed
EnterPressed
ArrowDownPressed
```

The same command can be bound differently by host applications, and the same
physical key can mean different commands in different control contexts.

### 2.4 Contexts Are Control-Family Specific

Different control families need different meanings for the same keys.

Examples:

- `down` opens a closed dropdown menu,
- `down` decreases or increases a vertical slider depending orientation,
- `down` selects the next radio item,
- `down` navigates an open menu.

The SDK should use control-family key contexts instead of one global
`LumaControl` context for every key.

Each focusable control attaches its control-family context to the same element
that tracks its control `FocusHandle`. The focus scope remains an ancestor with
`LUMA_FOCUS_CONTEXT`, so focus traversal actions can still bubble through the
same dispatch path.

### 2.5 Composite Controls Own Internal Navigation

Composite controls should usually expose one tab stop and manage internal
selection or active-item state themselves.

Examples:

- `RadioGroup` has one focus handle and a selected item.
- `DropdownMenu` has one trigger focus handle and an active menu path.
- `ContextMenu` has one target focus handle and an active menu path.
- future `Tabs` should likely have one focus handle and an active tab.

Do not create per-item tab stops unless each item is meant to be independently
reachable through application tab navigation.

### 2.6 Disabled Controls Do Not Mutate

Disabled controls should not be keyboard tab stops. If a disabled control is
focused programmatically, its control-specific key handlers must not mutate
state or emit semantic events.

It is acceptable for disabled controls to consume activation-like commands as a
no-op so an accidental programmatic focus does not trigger an ancestor command.
Focus actions remain governed by `gpui_luma::focus`.

## 3. Public API Shape

The SDK should add a small public module for control keyboard commands:

```rust
pub mod keyhandling {
    use gpui::{actions, App, KeyBinding};

    actions!(
        luma_controls,
        [
            ActivateControl,
            OpenControl,
            OpenFirstItem,
            OpenLastItem,
            CommitSelection,
            SelectNextItem,
            SelectPreviousItem,
            SelectFirstItem,
            SelectLastItem,
            OpenSubmenu,
            CloseSubmenu,
            IncreaseValue,
            DecreaseValue,
            IncreaseValueLarge,
            DecreaseValueLarge,
            MoveToStart,
            MoveToEnd,
            OpenContextMenu
        ]
    );

    pub const LUMA_COMMAND_CONTEXT: &str = "LumaCommandControl";
    pub const LUMA_CHOICE_CONTEXT: &str = "LumaChoiceControl";
    pub const LUMA_RADIO_GROUP_CONTEXT: &str = "LumaRadioGroup";
    pub const LUMA_SLIDER_CONTEXT: &str = "LumaSlider";
    pub const LUMA_MENU_BUTTON_CONTEXT: &str = "LumaMenuButton";
    pub const LUMA_MENU_CONTEXT: &str = "LumaMenu";
    pub const LUMA_CONTEXT_MENU_TARGET_CONTEXT: &str = "LumaContextMenuTarget";

    pub fn bind_default_control_keys(cx: &mut App) {
        cx.bind_keys([
            KeyBinding::new("enter", ActivateControl, Some(LUMA_COMMAND_CONTEXT)),
            KeyBinding::new("space", ActivateControl, Some(LUMA_COMMAND_CONTEXT)),
            KeyBinding::new("space", ActivateControl, Some(LUMA_CHOICE_CONTEXT)),
            KeyBinding::new("left", SelectPreviousItem, Some(LUMA_RADIO_GROUP_CONTEXT)),
            KeyBinding::new("up", SelectPreviousItem, Some(LUMA_RADIO_GROUP_CONTEXT)),
            KeyBinding::new("right", SelectNextItem, Some(LUMA_RADIO_GROUP_CONTEXT)),
            KeyBinding::new("down", SelectNextItem, Some(LUMA_RADIO_GROUP_CONTEXT)),
            KeyBinding::new("home", SelectFirstItem, Some(LUMA_RADIO_GROUP_CONTEXT)),
            KeyBinding::new("end", SelectLastItem, Some(LUMA_RADIO_GROUP_CONTEXT)),
            KeyBinding::new("left", DecreaseValue, Some(LUMA_SLIDER_CONTEXT)),
            KeyBinding::new("down", DecreaseValue, Some(LUMA_SLIDER_CONTEXT)),
            KeyBinding::new("right", IncreaseValue, Some(LUMA_SLIDER_CONTEXT)),
            KeyBinding::new("up", IncreaseValue, Some(LUMA_SLIDER_CONTEXT)),
            KeyBinding::new("pagedown", DecreaseValueLarge, Some(LUMA_SLIDER_CONTEXT)),
            KeyBinding::new("pageup", IncreaseValueLarge, Some(LUMA_SLIDER_CONTEXT)),
            KeyBinding::new("home", MoveToStart, Some(LUMA_SLIDER_CONTEXT)),
            KeyBinding::new("end", MoveToEnd, Some(LUMA_SLIDER_CONTEXT)),
            KeyBinding::new("down", OpenFirstItem, Some(LUMA_MENU_BUTTON_CONTEXT)),
            KeyBinding::new("up", OpenLastItem, Some(LUMA_MENU_BUTTON_CONTEXT)),
            KeyBinding::new("enter", OpenFirstItem, Some(LUMA_MENU_BUTTON_CONTEXT)),
            KeyBinding::new("space", OpenFirstItem, Some(LUMA_MENU_BUTTON_CONTEXT)),
            KeyBinding::new("down", SelectNextItem, Some(LUMA_MENU_CONTEXT)),
            KeyBinding::new("up", SelectPreviousItem, Some(LUMA_MENU_CONTEXT)),
            KeyBinding::new("home", SelectFirstItem, Some(LUMA_MENU_CONTEXT)),
            KeyBinding::new("end", SelectLastItem, Some(LUMA_MENU_CONTEXT)),
            KeyBinding::new("right", OpenSubmenu, Some(LUMA_MENU_CONTEXT)),
            KeyBinding::new("left", CloseSubmenu, Some(LUMA_MENU_CONTEXT)),
            KeyBinding::new("enter", CommitSelection, Some(LUMA_MENU_CONTEXT)),
            KeyBinding::new("space", CommitSelection, Some(LUMA_MENU_CONTEXT)),
            KeyBinding::new("shift-f10", OpenContextMenu, Some(LUMA_CONTEXT_MENU_TARGET_CONTEXT)),
            KeyBinding::new("menu", OpenContextMenu, Some(LUMA_CONTEXT_MENU_TARGET_CONTEXT)),
        ]);
    }
}
```

The exact module name can change before implementation, but the important
shape is:

- public typed actions,
- named key contexts,
- opt-in default key bindings,
- per-control action handlers.

`bind_default_control_keys(cx)` should be separate from
`bind_default_focus_keys(cx)`. Host applications may want SDK focus traversal
but custom control bindings, or custom traversal but default control bindings.

## 4. Control-Family Policy

### 4.1 Command Controls

Controls:

- `Button`
- `IconButton`
- command-like future controls

Default context:

```rust
LUMA_COMMAND_CONTEXT
```

Default bindings:

- `Enter` -> `ActivateControl`
- `Space` -> `ActivateControl`

Behavior:

- enabled controls emit their click event,
- disabled controls do nothing,
- activation consumes the action.

### 4.2 Toggle Command Controls

Controls:

- `ToggleButton`

Default context:

```rust
LUMA_COMMAND_CONTEXT
```

Default bindings:

- `Enter` -> `ActivateControl`
- `Space` -> `ActivateControl`

Behavior:

- enabled controls toggle selected state and emit change,
- disabled controls do nothing,
- activation consumes the action.

Toggle buttons behave like buttons with pressed or selected state, so they can
share the command-control context unless future behavior needs a separate
toggle context.

### 4.3 Choice Controls

Controls:

- `Checkbox`
- `Switch`
- future standalone choice controls

Default context:

```rust
LUMA_CHOICE_CONTEXT
```

Default bindings:

- `Space` -> `ActivateControl`

Behavior:

- enabled controls toggle value and emit change,
- disabled controls do nothing,
- activation consumes the action.

Do not bind `Enter` by default for checkbox-like controls unless the SDK
intentionally chooses a broader app convention. Space is the expected default
toggle key.

### 4.4 Radio Groups

Controls:

- `RadioGroup`
- future segmented single-select groups

Default context:

```rust
LUMA_RADIO_GROUP_CONTEXT
```

Default bindings:

- `Left` / `Up` -> `SelectPreviousItem`
- `Right` / `Down` -> `SelectNextItem`
- `Home` -> `SelectFirstItem`
- `End` -> `SelectLastItem`

Behavior:

- one tab stop enters the group,
- arrow keys move to the next enabled item and select it,
- disabled items are skipped,
- actions consume when the group is enabled,
- disabled groups do nothing.

Tab should leave the group through `NextFocus` or `PreviousFocus`. It should
not move between radio items.

### 4.5 Sliders And Range Inputs

Controls:

- `Slider`
- future `Scrollbar` or range-like input controls

Default context:

```rust
LUMA_SLIDER_CONTEXT
```

Default bindings:

- `Left` / `Down` -> `DecreaseValue`
- `Right` / `Up` -> `IncreaseValue`
- `PageDown` -> `DecreaseValueLarge`
- `PageUp` -> `IncreaseValueLarge`
- `Home` -> `MoveToStart`
- `End` -> `MoveToEnd`

Behavior:

- enabled controls snap values through the existing range and step logic,
- large increments use the control's large-step policy,
- disabled controls do nothing,
- actions consume when recognized.

Orientation-specific controls may choose different arrow bindings by using a
different context or by exposing an orientation mode before binding defaults.
The default horizontal slider policy should not become a hidden assumption for
all future range controls.

### 4.6 Dropdown Menus

Controls:

- `DropdownMenu`
- future select-like menu buttons

Default contexts:

```rust
LUMA_MENU_BUTTON_CONTEXT
LUMA_MENU_CONTEXT
```

Closed trigger bindings:

- `Down` -> `OpenFirstItem`
- `Up` -> `OpenLastItem`
- `Enter` -> `OpenFirstItem`
- `Space` -> `OpenFirstItem`

Open menu bindings:

- `Down` -> `SelectNextItem`
- `Up` -> `SelectPreviousItem`
- `Home` -> `SelectFirstItem`
- `End` -> `SelectLastItem`
- `Right` -> `OpenSubmenu`
- `Left` -> `CloseSubmenu`
- `Enter` -> `CommitSelection`
- `Space` -> `CommitSelection`
- `Escape` -> `EscapeFocus`

Behavior:

- closed menus open with an active enabled item,
- open menus keep focus on the trigger-owned focus handle,
- active item state is internal control state, not GPUI focus,
- committing a leaf item closes the menu and emits select,
- committing a parent item opens its submenu,
- `EscapeFocus` closes an open menu and consumes,
- `EscapeFocus` propagates when the menu is already closed.

`LUMA_MENU_CONTEXT` can be attached when the menu is open. If GPUI element
structure makes dynamic contexts awkward, the trigger can keep
`LUMA_MENU_BUTTON_CONTEXT` and the control can conditionally interpret menu
actions while open, but the conceptual distinction should remain in the design.

### 4.7 Context Menus

Controls:

- `ContextMenu`
- future context-menu targets

Default contexts:

```rust
LUMA_CONTEXT_MENU_TARGET_CONTEXT
LUMA_MENU_CONTEXT
```

Closed target bindings:

- `Shift-F10` -> `OpenContextMenu`
- platform menu key -> `OpenContextMenu`

Open menu bindings:

- same menu navigation bindings as `DropdownMenu`,
- `Escape` -> `EscapeFocus`.

Behavior:

- keyboard-opened context menus open at the target anchor,
- pointer-opened context menus open at the pointer position,
- active item state is internal control state,
- `EscapeFocus` closes an open menu and consumes,
- `EscapeFocus` propagates when the menu is already closed.

### 4.8 Text-Like Controls

Controls:

- future text input,
- future text area,
- future code/editor-like controls.

Text-like controls are the main exception to the shared-action default.

They may need raw key handling for:

- character insertion,
- IME or composition behavior,
- selection modification,
- repeat behavior,
- platform-specific editing shortcuts.

They should still participate in SDK focus actions:

- consume `NextFocus` when Tab inserts content or indents,
- propagate `NextFocus` when Tab should leave the field,
- consume `PreviousFocus` when Shift-Tab outdents,
- propagate `PreviousFocus` when Shift-Tab should leave the field,
- handle `EscapeFocus` only for transient state such as completions or inline
  popovers, then propagate.

Expose an explicit mode flag when both Tab insertion and focus traversal are
reasonable for the same control.

### 4.9 Data And Composite Navigation Controls

Controls:

- future tabs,
- future lists,
- future trees,
- future tables,
- future grids.

These controls need a per-control decision:

- does focus stay on the composite root with an active descendant model,
- does each item own a real GPUI focus handle,
- does Tab leave the composite or move inside it,
- do arrow keys select, move active item, or scroll.

Default rule:

- use one tab stop for the composite,
- use arrow keys for internal active-item movement,
- use `Tab` and `Shift-Tab` for leaving the composite,
- propagate focus actions at boundaries only when the composite intentionally
  owns Tab.

## 5. Handler Policy

### 5.1 Action Handlers

Action handlers consume by default in GPUI's bubble phase.

When a control handles an action:

```rust
fn activate(&mut self, _: &ActivateControl, _window: &mut Window, cx: &mut Context<Self>) {
    if !self.model.enabled {
        return;
    }

    self.activate_internal(cx);
}
```

When a control intentionally declines an action that an ancestor should handle:

```rust
fn handle_escape_focus(
    &mut self,
    _: &EscapeFocus,
    _window: &mut Window,
    cx: &mut Context<Self>,
) {
    if self.open {
        self.close_menu();
        cx.notify();
    } else {
        cx.propagate();
    }
}
```

Use `cx.propagate()` deliberately. Do not add it just to be defensive.

### 5.2 Raw Key Handlers

Raw `on_key_down` handlers are allowed when:

- no shared semantic action exists yet,
- the key is truly private to the control,
- raw keystroke data matters,
- text input or composition is involved,
- a temporary implementation is being used before extracting shared behavior.

When adding a raw handler, leave the code easy to promote into a shared action
later:

- keep key mapping in a small helper,
- return an enum or semantic intent,
- keep mutation separate from string matching,
- add tests for the mapping if the key matrix is non-trivial.

The existing `MenuKey` helper is the right transitional shape: it converts raw
keys into semantic menu intents before mutating menu state.

### 5.3 Pointer And Keyboard Activation Share Internals

Pointer clicks and keyboard activation should call the same internal behavior.

Recommended shape:

```rust
fn activate(&mut self, cx: &mut Context<Self>) -> bool {
    if !self.model.enabled {
        return false;
    }

    cx.emit(ButtonEvent::Click);
    true
}

fn handle_click(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
    self.activate(cx);
}

fn handle_activate_control(
    &mut self,
    _: &ActivateControl,
    _: &mut Window,
    cx: &mut Context<Self>,
) {
    self.activate(cx);
}
```

This avoids separate keyboard-only behavior drifting away from pointer behavior.

## 6. Template Policy

Templates should not own key behavior.

Templates can choose where the focusable element appears and must return an
element that the control can decorate with:

- `.track_focus(...)`,
- `.key_context(...)`,
- `.on_action(...)`,
- `.on_key_down(...)` only when still needed.

If a template emits multiple interactive regions, the control should pass
control-owned handlers into the template through a typed handlers struct. The
template places those handlers; it does not decide what they mean.

## 7. Implementation Plan

1. Add public `gpui_luma::keyhandling` actions, contexts, and
   `bind_default_control_keys(cx)`.
2. Update the gallery to call `bind_default_control_keys(cx)` next to
   `bind_default_focus_keys(cx)`.
3. Convert command controls to `ActivateControl`:
   `Button`, `IconButton`, `ToggleButton`.
4. Convert choice controls to `ActivateControl`:
   `Checkbox`, `Switch`.
5. Convert `RadioGroup` arrow/home/end handling to shared item-selection
   actions.
6. Convert `Slider` arrow/page/home/end handling to shared value actions.
7. Convert dropdown and context menu non-Escape navigation to shared menu
   actions.
8. Keep `EscapeFocus` in `gpui_luma::focus`; do not add a second Escape action
   for controls.
9. Keep raw handlers only where the design explicitly allows them.

## 8. Verification

Verify keyboard behavior in the gallery after each migration step:

- `Tab` and `Shift-Tab` still traverse enabled controls through the focus
  scope.
- disabled controls are skipped by tab traversal.
- `Escape` on a simple focused control returns focus to the scope.
- `Tab` still works after `Escape`.
- `Enter` and `Space` activate command controls.
- `Space` toggles checkbox-like controls.
- radio-group arrows select the next or previous enabled item.
- slider arrows, page keys, home, and end update value through range snapping.
- dropdown keyboard open, navigation, submenu, commit, and Escape behavior work.
- context-menu keyboard open, navigation, submenu, commit, and Escape behavior
  work.
- pointer and keyboard activation emit the same semantic events.

## 9. Design Boundary

This document is intentionally about control key handling, not full
accessibility semantics.

The same choices should later inform accessibility roles, active-descendant
semantics, labels, and screen-reader behavior, but those concerns should be
documented separately once the SDK has an accessibility foundation module.
