# GPUI-Luma Focus & Keyboard Handling Guide

This guide details the focus navigation model, traversal lifecycles, and action-based keyboard command systems within the GPUI-Luma SDK.

---

## 1. Focus Traversal & Focus Scopes

Luma uses GPUI's action-based dispatch path to manage focus traversal instead of ad-hoc key listeners.

### Focus Actions
Core focus traversal commands are declared in `luma::focus` as typed actions:
*   `NextFocus` (tab)
*   `PreviousFocus` (shift-tab)
*   `EscapeFocus` (escape)

### Focus Scope (`luma_focus_scope`)
A focus scope is a stable `FocusHandle` registered at the root of a view. It is not an extra layout wrapper.
Instead, it is attached to the existing root element using the `LumaFocusScopeExt` trait modifier:
```rust
div().luma_focus_scope(&self.focus_scope)
```
The scope intercepts traversal actions:
*   `NextFocus` -> calls `window.focus_next(cx)`
*   `PreviousFocus` -> calls `window.focus_prev(cx)`
*   `EscapeFocus` -> focuses the scope handle (returning focus to the root container rather than blurring the window).

### Focus Lifecycle Rules
*   Every interactive control must own a stable `FocusHandle` (usually wrapped via `ControlInteraction`).
*   **Enabled controls** must be tab stops (`tab_stop(true)`).
*   **Disabled controls** must clear focus and tab-stop eligibility (`tab_stop(false)`).
*   **Inert background clicks** should focus the parent scope handle (preventing window blurs, which break subsequent tab traversal actions).

---

## 2. Keyboard Key Handling Design

Luma maps physical key strokes to semantic typed actions using GPUI's key context system. Action names represent user intent, not physical keys.

### Core Key Profiles (`ControlKeyProfile`)
Controls bind default key profile actions to their focus-tracked elements. Below are the standard profiles:

| Profile | Target Controls | Action Commands | Key Bindings |
| :--- | :--- | :--- | :--- |
| **Command** | `Button`, `IconButton`, `ToggleButton` | `ActivateControl` | `Enter`, `Space` |
| **Choice** | `Checkbox`, `Switch` | `ActivateControl` | `Space` |
| **TabList** | `ControlGroup` composites, `TabsNavigation` | `SelectNextItem`, `SelectPreviousItem`, `SelectFirstItem`, `SelectLastItem` | Arrow keys (`Right/Down`, `Left/Up`), `Home`, `End` |
| **RangeValue** | `Slider` | `IncreaseValue`, `DecreaseValue`, `IncreaseValueLarge`, `DecreaseValueLarge`, `MoveToStart`, `MoveToEnd` | Arrow keys (`Right/Up`, `Left/Down`), `PageUp`, `PageDown`, `Home`, `End` |
| **ScrollOffset** | `Scrollbar` | `IncreaseValue`, `DecreaseValue`, `IncreaseValueLarge`, `DecreaseValueLarge`, `MoveToStart`, `MoveToEnd` | Arrow keys, `PageUp`, `PageDown`, `Home`, `End` |
| **Menu** | `PopupMenu` | `SelectNextItem`, `SelectPreviousItem`, `OpenSubmenu`, `CloseSubmenu`, `ActivateControl`, `EscapeFocus` | Arrow keys, `Enter`, `Space`, `Escape` |
| **ContextMenu**| `ContextMenu` | `OpenContextMenu` + Menu navigation | `Shift-F10`, Menu key |

---

## 3. Propagation & Override Policy

### 3.1 Escape Key Bubble Path
*   **Simple controls** (like buttons or switches) should not intercept `EscapeFocus`. They let it bubble up to the nearest focus scope.
*   **Composite controls with transient states** (like open menus) must handle `EscapeFocus` to close themselves, and then call `cx.propagate()` when closed so that outer ancestors or scopes can catch the action.

```rust
fn handle_escape_focus(&mut self, _: &EscapeFocus, _window: &mut Window, cx: &mut Context<Self>) {
    if self.open {
        self.close_menu();
        cx.notify();
    } else {
        cx.propagate(); // Let ancestor scopes handle the Escape
    }
}
```

### 3.2 Tab Key Override Path
*   Controls do not bind `Tab` directly.
*   Advanced text editors or grid sheets that want to use `Tab` internally (e.g. for indentation or cell movement) intercept `NextFocus` / `PreviousFocus` and call `cx.propagate()` only when hitting internal borders.

---

## 4. Initialization & Binding Registration

Default keys are registered explicitly at startup in `main.rs`:
```rust
luma::focus::bind_default_focus_keys(cx);
luma::key_handling::bind_default_control_keys(cx);
```
Binding focus and control keys are separate steps so applications can define when to load default SDK bindings or use custom ones.
