# Issue #9: Add Modal Dialog/Popup & PopupNotice Controls

## Description
Applications frequently need to prompt users with warning alerts or prompt dialogs centered over the screen. We need a generic Modal Dialog popup shell that dims the screen viewport behind it and intercepts pointer interactions, plus a standardized `PopupNotice` alert component that presents title, warning icon, message, and Action buttons.

## Proposed Solution
Create a new `dialog` module. It should render an absolute overlay wrapper `inset_0()` with a dimming backdrop `hsla(0, 0, 0, 0.4)` and a center container. Implement `PopupNotice` as a built-in pre-designed component utilizing the `dialog` overlay wrapper.

## Tasks
- [ ] Create `crates/sdk/src/controls/dialog/` module directory.
- [ ] Implement `Dialog` component to render the absolute center-aligned dim overlay shell.
- [ ] Implement `PopupNotice` utility helper building warnings, confirmations, and info messages.
- [ ] Style dialog shadows and elevation borders in the look-shadcn templates.
- [ ] Add interactive dialog triggers inside the Gallery app.

## Acceptance Criteria
- Activating a dialog displays a modal card centered in the viewport.
- Clicks outside the dialog boundaries close the modal (if configured).
- Standard Notice popups render cleanly with the warning/info icons and styled action button triggers.
