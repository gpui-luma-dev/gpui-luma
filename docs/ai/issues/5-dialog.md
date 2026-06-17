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
      pub header_template: Option<DialogHeaderTemplate>,
      pub footer_template: Option<DialogFooterTemplate>,
      pub state: T, // Domain state/data hosted inside the dialog
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

We will define a `DialogTemplate` trait to allow layout and style customizability:
```rust
pub trait DialogTemplate<T>: Send + Sync {
    fn render(
        &self,
        model: &DialogRenderModel<'_, T>,
        header: Option<AnyElement>,
        body: AnyElement,
        footer: Option<AnyElement>,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}
```

* **`DialogAppearance`**:
  Contains theme-aware visual rules for shadows, background, and dimming backdrops:
  ```rust
  pub struct DialogAppearance {
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

We will provide standard, pre-built templates to easily instantiate common variations:

* **Message Dialog (Alert / Information)**:
  - Renders a clean card containing an status icon (Info, Success, Warning, Error), a short title, a descriptive paragraph, and a single dismiss button (e.g., "OK").
* **Confirmation Dialog**:
  - Renders title, descriptive body, and a structured footer containing two action buttons: a primary action button (e.g., "Confirm", "Delete") and a cancel button. Supports destructive styling (e.g., red background for "Delete").
* **Modeless Panel**:
  - Modeless dialog template that skips dim backdrops, uses a transparent touch-through boundary, renders a close `(X)` icon button in the header, and is designed to float relative to other elements based on `DialogPosition` and `ModelessDismissPolicy`.

---

### 3. Focus Lifecycle, Overlay Stacking & Modeless Draggability

* **Focus Restoration**:
  When a dialog is spawned (especially modal), focus is programmatically directed to the dialog's root or first interactive field. To prevent focus from being lost when the dialog is dismissed, the control must capture the currently active `FocusHandle` at the moment of opening and restore focus to it upon dismissal.
* **Modal Focus Scope**:
  When configured in `DialogMode::Modal`, the control traps focus navigation (Tab / Shift-Tab) using an active `luma_focus_scope` to ensure focus cycles exclusively within the dialog.
* **Escape Handling**:
  The dialog intercepts `EscapeFocus` to dismiss itself (if `dismissible: true` is set) and consumes the action rather than propagating it to the host view.
* **Nesting & Stacking**:
  If multiple modal dialogs are stacked (e.g. a Confirmation alert over an Options dialog), the system should manage backdrops and layers so that only the topmost dialog is interactive and receives events, and dismissing it restores focus to the underlying dialog.
* **Modeless Draggability**:
  For floating panels (such as color pickers), we will support a draggable header zone. Clicking and dragging the header moves the dialog, updating `DialogPosition::Absolute` in the state.

---

## Tasks

### Phase 1: SDK Dialog Core (`crates/sdk`)
- [ ] Create `crates/sdk/src/controls/dialog/mod.rs` to export the dialog types and macros.
- [ ] Implement the `DialogModel`, `DialogMode`, `DialogPosition`, and `DialogBuilder` in `model.rs`.
- [ ] Implement the runtime control, event handlers, positioning layouts (flex layout bounds for semantic positions and absolute offsets for `Absolute`), and keyboard/focus trap in `control.rs`.
- [ ] Implement **Focus Restoration** tracking (saving and restoring the previously focused handle) in `control.rs`.
- [ ] Implement **Draggability mouse event handlers** for modeless dialog headers in `control.rs`.
- [ ] Implement the baseline appearance metrics and palette rules in `theme.rs` (including `backdrop_background` and `backdrop_blur`).
- [ ] Implement `DialogTemplate` and the pre-built templates (`MessageDialogTemplate`, `ConfirmationDialogTemplate`, `ModelessPanelTemplate`) in `template.rs`.
- [ ] Register `dialog` in `crates/sdk/src/controls/mod.rs`.

### Phase 2: Downstream Look-Shadcn Integration (`crates/look-shadcn`)
- [ ] Define default stylesheet rule sets in `crates/look-shadcn/assets/style.toml` (handling borders, padding scales, shadows, and backdrop colors).
- [ ] Add config models for `dialog` in `crates/look-shadcn/src/stylesheet/config.rs` and matching rules in `src/look.rs`.

### Phase 3: Gallery Showcase (`apps/gallery`)
- [ ] Add a new "Dialog" demo pane in the Gallery page registry (`apps/gallery/src/gallery/panes/registry.rs`).
- [ ] Implement interactive showcase triggers for:
  - A standard modal Warning Alert dialog.
  - A modal Confirmation dialog.
  - A modeless floating information panel that overlays in a corner (e.g. `DialogPosition::TopRight`) without interrupting main view clicks.
  - An absolute-positioned modeless dialog (e.g., spawning where the click occurred).
  - A stacked modal demo (triggering a confirmation alert from within a modal form dialog).
  - A draggable modeless color picker or layout panel.

---

## Verification Plan

### Automated Tests
- [ ] Add unit tests in `crates/sdk/src/controls/dialog/control.rs` to verify:
  - Escape key dismisses when `dismissible` is true.
  - Click on backdrop dismisses when modal and dismissible.
  - Focus is successfully returned to the previously focused handle when closed.
  - Modeless configurations do not intercept click events outside bounds.
  - Semantic and concrete positioning layouts generate expected layout properties.

### Manual Verification
- Run `just gallery` and navigate to the **Dialog Showcase**:
  - Trigger each dialog variation and check positioning (Center, TopRight, and Absolute coordinates).
  - Verify focus returns to the previously focused button after closing a dialog.
  - Verify that dragging the header of a modeless panel repositions it smoothly.
  - Verify that nested modal stacking works (underlying dialog remains open, topmost modal takes focus, closing the topmost modal returns focus to the underlying dialog).
  - Validate look/mode swaps (light/dark mode) apply correct background dims and border shadows.
