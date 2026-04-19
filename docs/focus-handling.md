# Focus Handling

## 1. Purpose

This document describes the keyboard focus navigation model implemented for
GPUI-Luma controls. It records how the SDK uses GPUI focus handles, key
contexts, and action dispatch after the focus-handling implementation pass.

The goal is to make focus behavior:

- idiomatic to GPUI,
- shared across controls,
- predictable for host applications,
- compatible with text inputs and composite controls,
- stable across renders.

## 2. Implemented State

SDK controls own focus locally. Most interactive controls create a
`FocusHandle` through `ControlInteraction`, attach it with `.track_focus(...)`,
and expose it through `Focusable`.

That part is intentional. A control needs a stable focus identity, and enabled
controls are GPUI tab stops.

Focus traversal is centralized through the public `gpui_luma::focus` module:

- `NextFocus`,
- `PreviousFocus`,
- `EscapeFocus`,
- `bind_default_focus_keys(cx)`,
- `LumaFocusScopeExt`.

There is no shared `blur_on_escape` helper and the controls no longer use raw
`.on_key_down(...)` callbacks for the SDK's default focus policy. `Tab`,
`Shift-Tab`, and `Escape` are modeled as typed actions.

Control-specific keyboard behavior is separate and lives in
`gpui_luma::keyhandling`. That module owns actions such as `ActivateControl`,
`SelectNextItem`, `IncreaseValue`, and `OpenContextMenu`, plus control key
profiles that individual controls attach to their focus-tracked elements.

## 3. GPUI Model

GPUI's keyboard command path is action based:

- define typed actions with `actions!(...)`,
- bind physical keys with `cx.bind_keys(...)`,
- attach key contexts with `.key_context(...)`,
- track focus with `.track_focus(...)`,
- handle commands with `.on_action(...)`,
- move focus with `window.focus_next(cx)` and `window.focus_prev(cx)`.

Action dispatch follows the focused node's element path. A focused child can
handle an action first. If it calls `cx.propagate()`, an ancestor can handle the
same action. Bubble-phase action handlers stop propagation by default, so
fallback behavior must be explicit.

This is the right fit for focus traversal: common traversal lives on a focus
scope, while deeper controls can override or consume the same actions when they
own more specific keyboard behavior.

## 4. Focus Actions

The SDK exposes focus actions directly from the public `focus` module.

```rust
pub mod focus {
    use gpui::{actions, App, FocusHandle, InteractiveElement, KeyBinding, Window};

    actions!(luma_focus, [NextFocus, PreviousFocus, EscapeFocus]);

    const FOCUS_CONTEXT: &str = "LumaFocus";

    pub fn bind_default_focus_keys(cx: &mut App) {
        cx.bind_keys([
            KeyBinding::new("tab", NextFocus, Some(FOCUS_CONTEXT)),
            KeyBinding::new("shift-tab", PreviousFocus, Some(FOCUS_CONTEXT)),
            KeyBinding::new("escape", EscapeFocus, Some(FOCUS_CONTEXT)),
        ]);
    }
}
```

The action structs are declared in `pub mod focus`, so consumers import them as:

```rust
use gpui_luma::focus::{EscapeFocus, NextFocus, PreviousFocus};
```

Do not write `pub use actions::{...}`. GPUI's `actions!` macro declares action
structs in the current module; it does not create an `actions` module.

## 5. Focus Scope

A focus scope is a stable, view-owned `FocusHandle` attached to an existing root
element. It is not an extra wrapper element.

The SDK provides a non-wrapper element extension that decorates the caller's
existing root element:

```rust
pub trait LumaFocusScopeExt: InteractiveElement + Sized {
    fn luma_focus_scope(self, scope: &FocusHandle) -> Self;
}

impl<E> LumaFocusScopeExt for E
where
    E: InteractiveElement + Sized,
{
    fn luma_focus_scope(self, scope: &FocusHandle) -> Self {
        let scope = scope.clone();

        self.track_focus(&scope)
            .key_context(FOCUS_CONTEXT)
            .on_action(|_: &NextFocus, window: &mut Window, cx: &mut App| {
                window.focus_next(cx);
            })
            .on_action(|_: &PreviousFocus, window: &mut Window, cx: &mut App| {
                window.focus_prev(cx);
            })
            .on_action(move |_: &EscapeFocus, window: &mut Window, cx: &mut App| {
                window.focus(&scope, cx);
            })
    }
}
```

Host code uses it on the root element it already renders:

```rust
struct MyView {
    focus_scope: FocusHandle,
    input_field: Entity<TextInput>,
}

impl Render for MyView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .luma_focus_scope(&self.focus_scope)
            .child(self.input_field.clone())
    }
}
```

This avoids an artificial layout node. GPUI does not provide `display:
contents`, so a generic wrapper `div()` can affect flex layout, hit testing,
occlusion, and style inheritance.

## 6. Scope Focus Lifecycle

Context-bound key bindings only work when the matching key context is in the
focused node's dispatch path. Because `Tab`, `Shift-Tab`, and `Escape` are bound
to the private focus key context, the application must keep focus inside a
rendered focus scope.

Each top-level surface that wants SDK focus navigation should:

- own a stable scope `FocusHandle`,
- call `.luma_focus_scope(&self.focus_scope)` on its existing root element,
- focus the scope handle when the surface opens,
- return focus to the scope handle when inert background areas are clicked.

Startup example:

```rust
struct MyView {
    focus_scope: FocusHandle,
}

impl MyView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_scope = cx.focus_handle();
        window.focus(&focus_scope, cx);

        Self { focus_scope }
    }
}
```

Background-click example:

```rust
let scope = self.focus_scope.clone();

div()
    .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
        window.focus(&scope, cx);
        cx.stop_propagation();
    });
```

Do not use `window.blur()` for ordinary inert-background clicks inside a focus
surface. A full blur removes the focused node from the dispatch path, so the
next context-bound `Tab` may not dispatch `NextFocus`.

The gallery implements this shape:

- `apps/gallery/src/main.rs` calls `bind_default_focus_keys(cx)` during startup.
- `GalleryApp` owns `focus_scope: FocusHandle`.
- `GalleryApp::new` focuses the scope when the gallery opens.
- `GalleryApp::render` calls `.luma_focus_scope(&self.focus_scope)` on the root
  element.
- The inert background handles left mouse down by focusing the scope and
  stopping propagation.

## 7. Control Focus Ownership

Controls own their own stable `FocusHandle`s.

Enabled interactive controls should be tab stops:

```rust
cx.focus_handle().tab_stop(true)
```

Disabled controls should remain focusable only by programmatic choice and
should not be keyboard tab stops:

```rust
focus_handle = focus_handle.clone().tab_stop(false);
```

The focus scope itself should be focusable but normally should not be a tab
stop. It exists as a stable action-dispatch anchor and escape target, not as a
user-visible item in the tab order.

`ControlInteraction` implements the common single-surface case:

- `new(enabled, cx)` creates `cx.focus_handle().tab_stop(enabled)`,
- `set_enabled(enabled)` updates the tab-stop state,
- disabling clears hover and pressed state,
- `handle_mouse_down` focuses the control when enabled,
- `render_state(enabled, window)` projects focus into `InteractionState`.

Composite controls can own focus directly. `RadioGroup` has one group
`FocusHandle`, tracks focus on the rendered group, and projects active item
state through `CompositeItemState`. Menus project root focus through
`ControlFocusState`, whose `focus_visible` flag is true only when GPUI says the
last input modality was keyboard.

## 8. Escape Semantics

`EscapeFocus` is layered.

The default scope behavior is:

```rust
window.focus(&scope, cx);
```

That clears the active control while keeping focus inside the surface, which
keeps the next `Tab` reliable.

Simple controls should not handle `EscapeFocus`. They should let the focus scope
receive the action.

Composite controls should handle `EscapeFocus` only when they have transient
state to dismiss:

- an open `DropdownMenu` closes the menu and consumes `EscapeFocus`,
- a closed `DropdownMenu` calls `cx.propagate()`,
- an open `ContextMenu` closes the menu and consumes `EscapeFocus`,
- a closed `ContextMenu` calls `cx.propagate()`.

Example:

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

Because GPUI action handlers stop propagation by default during the bubble
phase, controls must call `cx.propagate()` when they intentionally decline to
consume `EscapeFocus`.

## 9. Tab Override Policy

`NextFocus` and `PreviousFocus` are default focus traversal actions, not an
absolute ownership claim over `Tab`.

Most controls should not handle these actions. They should let the nearest focus
scope move focus.

No current SDK control overrides `NextFocus` or `PreviousFocus`. Future controls
that legitimately own Tab behavior may override it with a deeper key context and
local action handlers. Examples include:

- text inputs that insert a tab character,
- editors that indent or outdent,
- grids that move within cells before leaving the grid,
- composite widgets with an internal roving focus model.

The policy is:

- consume `NextFocus` or `PreviousFocus` when Tab changes internal control
  state or inserts content,
- call `cx.propagate()` when the control is at its internal boundary and wants
  the outer scope to continue traversal,
- expose an opt-out or mode flag for controls where both behaviors are useful.

Example for a future text-like control:

```rust
fn handle_next_focus(&mut self, _: &NextFocus, _window: &mut Window, cx: &mut Context<Self>) {
    if self.accepts_tab_input() {
        self.insert_tab(cx);
    } else {
        cx.propagate();
    }
}
```

This preserves normal application focus traversal while giving specialized
controls a clear escape hatch.

## 10. API Shape

The SDK exposes a small public focus module:

```rust
pub mod focus {
    pub fn bind_default_focus_keys(cx: &mut App);

    pub trait LumaFocusScopeExt: InteractiveElement + Sized {
        fn luma_focus_scope(self, scope: &FocusHandle) -> Self;
    }
}
```

`bind_default_focus_keys(cx)` is opt-in. Host applications may already own
`Tab`, `Shift-Tab`, or `Escape` bindings. The gallery calls the helper
explicitly.

## 11. Implementation Status

Implemented:

- `gpui_luma::focus` defines `NextFocus`, `PreviousFocus`, `EscapeFocus`,
  `bind_default_focus_keys(cx)`, and `LumaFocusScopeExt`.
- The gallery calls `bind_default_focus_keys(cx)` during startup.
- The gallery root owns a stable focus-scope handle and focuses it when the
  gallery opens.
- The gallery inert background focuses the gallery scope instead of blurring the
  window.
- Simple controls do not handle `EscapeFocus`; they let the scope consume it.
- Dropdown and context menus handle `EscapeFocus`, close when open, and
  propagate when closed.
- Arrow-key, home/end, menu, context-menu, slider, and scrollbar navigation
  behavior stays inside the owning controls through `ControlKeyProfile`.

## 12. Verification

Verify the implemented behavior in the gallery with these checks:

- `Tab` moves through enabled controls in render/tab order.
- `Shift-Tab` moves backward through enabled controls.
- disabled controls are skipped.
- background clicks return focus to the gallery scope rather than blurring the
  window.
- `Escape` on a simple focused control returns focus to the scope root.
- `Tab` still works after `Escape`.
- `Escape` closes an open dropdown or context menu without leaving the scope.
- a second `Escape` after closing a menu returns focus to the scope root.
- control-specific keys still work, such as slider arrows and radio-group
  arrows.
- future text-like controls can consume or propagate `NextFocus` according to
  their mode.
