# Issue #5: Dialog Control (Modal & Modeless) with Themed Variations

## Description

Applications need a clean way to prompt users for actions, show alerts, display confirmations, or present floating non-blocking tools. Currently, the workspace lacks a first-class, theme-aware Dialog control.

We need a flexible `Dialog` component in the SDK that supports:
1. **Modal Mode**: Draws a dimming backdrop overlay, traps keyboard focus, and blocks interactions with background elements until dismissed.
2. **Modeless Mode**: Acts as a floating, non-blocking window or overlay panel. Background elements remain fully interactive.
3. **Template-Driven Variations**: Supports distinct layout variants (e.g., standard "Message Dialog", "Confirmation Alert", or custom "Form Dialog") through a customizable template architecture.

---

## Proposed Solution

We will create a new SDK control under `crates/sdk/src/controls/dialog/` with the standard SDK control structure:

### 1. Dialog Configuration & State Models (`model.rs` & `control.rs`)

* **`DialogMode`**:
  ```rust
  #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
  pub enum DialogMode {
      #[default]
      Modal,
      Modeless,
  }
  ```

* **`ModelessDismissPolicy`**:
  Configures the dismissal policy for modeless dialogs when focus changes or click-away interactions occur:
  ```rust
  #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
  pub enum ModelessDismissPolicy {
      /// Stays open until explicitly closed (e.g., clicking close button or pressing Escape). Perfect for floating color pickers or toolbox bars.
      #[default]
      KeepOpen,
      /// Automatically closes when the user clicks anywhere outside the dialog bounds.
      CloseOnClickAway,
      /// Automatically closes when focus shifts away from the dialog.
      CloseOnFocusLoss,
  }
  ```

* **`DialogPosition`**:
  Defines how the dialog is positioned within the parent container or viewport:
  ```rust
  #[derive(Clone, Copy, Debug, Default, PartialEq)]
  pub enum DialogPosition {
      #[default]
      Center,
      Top,
      Bottom,
      Left,
      Right,
      TopLeft,
      TopRight,
      BottomLeft,
      BottomRight,
      /// Concrete coordinates relative to the parent context (e.g. at cursor or target element)
      Absolute(gpui::Point<gpui::Pixels>),
  }
  ```

* **`DialogModel<T>`**:
  Stores state, layout size, modal mode, positioning, dismissal policy, and custom child slots.
  ```rust
  pub struct DialogModel<T> {
      pub id: SharedString,
      pub mode: DialogMode,
      pub position: DialogPosition,
      pub dismiss_policy: ModelessDismissPolicy,
      pub dismissible: bool, // Allows closing via Escape key or backdrop clicks
      pub size: ControlSize,
      pub template: Arc<dyn DialogTemplate<T>>,
      pub state: T, // Domain state/data hosted inside the dialog
  }
  ```

* **`DialogRenderModel<'a, T>`**:
  Read-only snapshot passed to the renderer/template.
  ```rust
  pub struct DialogRenderModel<'a, T> {
      pub id: &'a SharedString,
      pub mode: DialogMode,
      pub position: DialogPosition,
      pub dismissible: bool,
      pub size: ControlSize,
      pub state: &'a T,
      pub focused: bool,
  }
  ```

* **`DialogEvent`**:
  Allows parents/hosts to hook into dialog lifecycles:
  ```rust
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub enum DialogEvent {
      Opened,
      Dismissed,
      Confirmed,
  }
  ```

---

### 2. Dialog Template Architecture (`template.rs` & `theme.rs`)

We will define a single `DialogTemplate` trait to allow layout and style customizability:
```rust
pub trait DialogTemplate<T>: Send + Sync {
    fn render(
        &self,
        model: &DialogRenderModel<'_, T>,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}
```

Because templating is so flexible (and can contain modifiers), having variations of dialog boxes with individual templates is cleaner. For initial dev work we want an 'x' icon to close the dialog and OK and cancel buttons and that's it:

1.  **Header**: Renders the standard close `(X)` icon button (if `dismissible` is true). Clicking this close button triggers dismissal (`DialogEvent::Dismissed`).
2.  **Body**: Renders the layout defined by the concrete template utilizing `model.state`.
3.  **Footer**: Renders two standard action buttons:
    *   **Cancel Button**: Outline button that triggers dismissal (`DialogEvent::Dismissed`).
    *   **OK Button**: Filled button that triggers confirmation (`DialogEvent::Confirmed`).

* **`DialogLook`**:
  Contains theme-aware visual rules for shadows, background, and dimming backdrops:
  ```rust
  pub struct DialogLook {
      pub background: Hsla,
      pub border: Hsla,
      pub radius: f32,
      pub shadow: Shadow,
      /// Theme-controlled dim color overlay for Modal dialogs (e.g. `hsla(0, 0, 0, 0.4)`)
      pub backdrop_background: Hsla,
      /// Optional blur backdrop filter for glassmorphic dim overlay
      pub backdrop_blur: Option<gpui::Pixels>,
  }
  ```

---

### 3. Focus Lifecycle, Overlay Stacking & Modeless Draggability

* **Focus Restoration**:
  When a dialog is spawned (especially modal), focus is programmatically directed to the dialog's root or first interactive field. To prevent focus from being lost when the dialog is dismissed, the control must capture the currently active `FocusHandle` at the moment of opening and restore focus to it upon dismissal.
* **Modal Focus Scope & Trap**:
  When configured in `DialogMode::Modal`, the control traps focus navigation (Tab / Shift-Tab) using `luma_focus_scope`. Because background elements still exist in the element tree, if `window.focus_next` or `window.focus_prev` moves focus to a handle not contained within the dialog (detected via `FocusHandle::contains`), the control must intercept this transition and programmatically wrap focus back to the dialog's first or last focusable element.
* **Escape Handling**:
  The dialog intercepts `EscapeFocus` to dismiss itself (if `dismissible: true` is set) and consumes the action rather than propagating it to the host view.
* **Nesting & Stacking**:
  If multiple modal dialogs are stacked, the system manages backdrops and layers so that only the topmost dialog is interactive and receives events, and dismissing it restores focus to the underlying dialog.
* **Modeless Draggability**:
  For floating panels (such as color pickers), we will support a draggable header zone. Clicking and dragging the header moves the dialog, updating `DialogPosition::Absolute` in the state using GPUI's `.on_drag` and `.on_drag_move` APIs.

---

## Tasks

### Phase 1: SDK Dialog Core (`crates/sdk`)
- [ ] Create `crates/sdk/src/controls/dialog/mod.rs` to export the dialog types and macros.
- [ ] Implement the `DialogModel`, `DialogMode`, `DialogPosition`, `DialogRenderModel`, and `DialogBuilder` in `model.rs`.
- [ ] Implement the runtime control `DialogControl<T>` (aliased to `Dialog<T>`), event handlers, positioning layouts (flex layout bounds for semantic positions and absolute offsets for `Absolute`), and keyboard/focus trap in `control.rs`.
- [ ] Implement **Focus Restoration** tracking (saving and restoring the previously focused handle) in `control.rs`.
- [ ] Implement **Draggability mouse event handlers** (`DialogDrag` payload) for modeless dialog headers in `control.rs`.
- [ ] Implement the baseline look metrics and palette rules in `theme.rs` (including `backdrop_background` and `backdrop_blur`).
- [ ] Implement a default concrete `DefaultDialogTemplate` in `template.rs` rendering the header with `(X)` close button, state-defined body content, and OK/Cancel buttons.
- [ ] Register `dialog` in `crates/sdk/src/controls/mod.rs`.

### Phase 2: Downstream Look-Shadcn Integration (`crates/look-shadcn`)
- [ ] Define default stylesheet rule sets in `crates/look-shadcn/assets/style.toml` (handling borders, padding scales, shadows, and backdrop colors).
- [ ] Add config models for `dialog` in `crates/look-shadcn/src/stylesheet/config.rs`.
- [ ] Implement resolution functions `resolve_dialog_color_rule` in `crates/look-shadcn/src/stylesheet/resolve.rs` and export them in `crates/look-shadcn/src/stylesheet/mod.rs`.
- [ ] Implement builder factory extension methods `dialog` in `crates/look-shadcn/src/look.rs` and `crates/look-shadcn/src/controls/ext.rs`.
- [ ] Declare Radix/Shadcn-themed template and theme adapters in `crates/look-shadcn/src/controls/templates.rs` or `crates/look-shadcn/src/controls/dialog.rs`.

### Phase 3: Gallery Showcase (`apps/gallery`)
- [ ] Add a new "Dialog" demo pane in the Gallery page registry (`apps/gallery/src/gallery/panes/registry.rs`).
- [ ] Implement interactive showcase triggers using the standard template:
  - A standard modal warning / alert dialog.
  - A modeless floating information panel that overlays in a corner (e.g. `DialogPosition::TopRight`) without interrupting main view clicks.
  - An absolute-positioned modeless dialog (e.g., spawning where the click occurred).
  - A draggable modeless dialog panel.

---

## Verification Plan

### Automated Tests
- [ ] Add unit tests in `crates/sdk/src/controls/dialog/control.rs` to verify:
  - Escape key dismisses when `dismissible` is true.
  - Close `(X)` button click dismisses.
  - Click on backdrop dismisses when modal and dismissible.
  - OK button triggers `DialogEvent::Confirmed` and closes.
  - Cancel button triggers `DialogEvent::Dismissed` and closes.
  - Focus is successfully returned to the previously focused handle when closed.
  - Modeless configurations do not intercept click events outside bounds.
  - Semantic and concrete positioning layouts generate expected layout properties.

### Manual Verification
- Run `just gallery` and navigate to the **Dialog Showcase**:
  - Trigger each dialog variation and check positioning (Center, TopRight, and Absolute coordinates).
  - Verify close button, OK button, and Cancel button dismiss dialogs correctly.
  - Verify focus returns to the previously focused button after closing a dialog.
  - Verify that dragging the header of a modeless panel repositions it smoothly.
  - Verify that nested modal stacking works (underlying dialog remains open, topmost modal takes focus, closing the topmost modal returns focus to the underlying dialog).
  - Validate look/mode swaps (light/dark mode) apply correct background dims and border shadows.
