# Issue #4: Add Theme Selector to the Gallery Application

## Description
The Theme Studio app allows users to switch between different Shadcn stylesheets using a theme selector dropdown in the sidebar. The Gallery app currently lacks this capability (it only supports a command-line flag or fallback default), making it difficult to test multiple theme sheets side-by-side.

## Proposed Solution
Add a dropdown selector to the Gallery title bar or sidebar that lists available tweakcn themes (retrieved dynamically via file/theme discovery). Selecting a theme updates the active `ShadcnLook` context and notifies the workspace elements.

## Tasks
- [ ] Implement `sync_theme` in [GalleryApp](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/control.rs) to handle look swaps.
- [ ] Build a dropdown trigger inside [template.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/template.rs) (using `Selector` or a custom menu trigger).
- [ ] Bind selected theme updates to the state of the app shell, triggering a re-render.

## Acceptance Criteria
- A dropdown/selector is visible in the Gallery UI.
- Swapping the theme immediately re-paints all controls inside the gallery viewport with the selected look.
