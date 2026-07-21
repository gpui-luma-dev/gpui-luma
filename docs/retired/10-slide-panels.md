# Issue #10: Add Slide Panels (Drawers/Sheets)

## Description
Many applications use slide-out navigation sheets or detail drawers that transition smoothly from one of the screen edges (Left, Right, Top, or Bottom). We need a generic `SlidePanel` control that anchors absolutely to the window boundary, slides in upon activation, and provides an easy way to close or hide the panel (such as a close button, backdrop click, or keyboard actions).

## Proposed Solution
Create a new `slide_panel` prototype control and pane under `apps/gallery/src/gallery/panes/prototypes/slide_panel/` (similar to how the `shadow_button` prototype is structured). Position the panel container absolutely based on the selected edge (`Left`, `Right`, `Top`, `Bottom`), and support an overlay backdrop. Incorporate a standard close/hide button (e.g., an "X" icon button) within the panel header or top corner.
Support standard keyboard accessibility:
1. Dismiss/close the active panel on `Escape`.
2. Trap focus within the panel container (focus trapping) while it is open.
3. Restore focus to the trigger control that opened the panel upon closing.

## Constraints
> [!IMPORTANT]
> There should be no changes to the `luma-studio` app for this work.
> The prototype control and pane must be fully self-contained within the Gallery application under `apps/gallery/src/gallery/panes/prototypes/slide_panel/` to ensure zero impact outside the gallery.

## Tasks
- [ ] Create `apps/gallery/src/gallery/panes/prototypes/slide_panel/` module folder.
- [ ] Implement `SlidePanel` control builders/logic inside the prototype folder (e.g., `control.rs` or `mod.rs`).
- [ ] Map position anchors (`top_0().left_0().h_full()` for Left, etc.).
- [ ] Hook up transition states to slide in.
- [ ] Add a close/hide button to the panel design (along with optional backdrop click-to-close behavior).
- [ ] Bind `Escape` to close/dismiss the active panel.
- [ ] Implement focus trapping (keeping `Tab`/`Shift-Tab` within the panel).
- [ ] Restore focus to the previously active element upon closing the panel.

- [ ] Add a SlidePanel gallery pane to the Gallery app with buttons to trigger slide panels from each edge (Left, Right, Top, Bottom) to demonstrate and test the visual and interaction states.

## Acceptance Criteria
- Slide panels slide out smoothly from their designated window edge.
- Background backdrops overlay the rest of the application interface while the panel is active.
- The panel contains a functional close/hide button that triggers the slide-out transition and closes the panel.
- Pressing `Escape` while the panel is active closes the panel and restores focus to the trigger element.
- Keyboard focus is trapped inside the panel while it is active (cycling via `Tab`/`Shift-Tab` stays within the panel).
- The Gallery app contains a dedicated SlidePanel pane with buttons to trigger and preview each panel direction.



