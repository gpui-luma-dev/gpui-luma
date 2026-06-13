# Issue #1: VStack/HStack Layout Macro Demos & App Refactoring

## Description
The SDK exposes powerful layout macros (`vstack!`, `hstack!`, `wrappanel!`, and `flow!`) to handle flex layout styling cleanly. However, these are under-documented, and the gallery application doesn't showcase them. Additionally, several views under `apps/` are still hand-rolling `.flex().flex_col().gap(...)` instead of leveraging these standard macros.

## Proposed Solution
1. Create a new gallery pane displaying layout macro examples with interactive toggles for gap, alignment, and justification.
2. Refactor existing views in `apps/` to use the layout helper macros.

## Tasks
- [ ] Register `GalleryPageKind::LayoutMacros` in [registry.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/registry.rs).
- [ ] Implement layout macros showcase pane in `apps/gallery/src/gallery/panes/layout_macros/pane.rs` demonstrating `vstack!`, `hstack!`, and `wrappanel!`.
- [ ] Replace hand-rolled flex column code with `vstack!` inside [palette/pane.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/palette/pane.rs).
- [ ] Replace layout wrappers in [inspector_detail.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/button/inspector_detail.rs) with `vstack!`.
- [ ] Replace detail header layouts in [detail.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/shared/inspector/detail.rs) with `vstack!`.

## Acceptance Criteria
- A "Stack Layouts" page is accessible under the Layout section in the gallery sidebar.
- Layout parameters (gap, alignment, justification) can be edited interactively.
- Refactored files build successfully and visual layouts remain unchanged.
