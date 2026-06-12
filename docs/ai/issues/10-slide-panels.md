# Issue #10: Add Slide Panels (Drawers/Sheets)

## Description
Many applications use slide-out navigation sheets or detail drawers that transition smoothly from one of the screen edges (Left, Right, Top, or Bottom). We need a generic `SlidePanel` control that anchors absolutely to the window boundary and slides in upon activation.

## Proposed Solution
Create a new `slide_panel` component in `crates/sdk/src/controls/slide_panel`. Position the panel container absolutely based on the selected edge (`Left`, `Right`, `Top`, `Bottom`), and support an overlay backdrop.

## Tasks
- [ ] Create `crates/sdk/src/controls/slide_panel/` module folder.
- [ ] Implement `SlidePanel` builders with edge selection.
- [ ] Map position anchors (`top_0().left_0().h_full()` for Left, etc.).
- [ ] Hook up transition states to slide in.
- [ ] Add slide-out examples to the Gallery app.

## Acceptance Criteria
- Slide panels slide out smoothly from their designated window edge.
- Background backdrops overlay the rest of the application interface while the panel is active.
