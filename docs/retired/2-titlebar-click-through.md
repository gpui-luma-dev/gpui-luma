# Issue #2: Fix TitleBar Switch Button Click-Through

## Description
When clicking the light/dark mode switch button in the title bar of the Gallery or Luma Studio, the mouse-down events bubble up to the parent title bar layout element. If double-clicked (or clicked rapidly), this triggers the title bar's double-click zoom/maximize behavior (`zoom_window()`), unexpectedly making the application zoom or enter full-screen mode.

## Proposed Solution
Stop mouse event propagation at the theme switch button itself in both applications. By adding an `on_mouse_down` handler that calls `cx.stop_propagation()`, we ensure that the mouse-down clicks do not reach the parent title bar's zoom listener.

## Tasks
- [ ] Add `.on_mouse_down(MouseButton::Left, |_, cx| cx.stop_propagation())` to `gallery-titlebar-theme-toggle` in [template.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/template.rs).
- [ ] Add `.on_mouse_down(MouseButton::Left, |_, cx| cx.stop_propagation())` to `luma-studio-mode-toggle` in [app.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/luma-studio/src/studio/app.rs).

## Acceptance Criteria
- Rapidly clicking or double-clicking the theme mode button in the Gallery changes the theme correctly without maximizing the window.
- Rapidly clicking or double-clicking the mode switch in Luma Studio changes the theme correctly without maximizing the window.
