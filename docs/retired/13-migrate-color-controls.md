# Issue #13: Migrate gpui-opal Color Controls to SDK

## Description
The color-picker components and color adjustment inputs developed in the `gpui-opal` project need to be migrated to the `gpui-luma` SDK to serve as first-class controls.

## Proposed Solution
Copy files from `/Users/scg/Developer/GitHub/gpui-opal/crates/sdk/src/controls/color/` into `crates/sdk/src/controls/color/` within `gpui-luma`, integrate them under the controls module, and style them appropriately.

## Tasks
- [ ] Create `crates/sdk/src/controls/color/` directory in the luma SDK.
- [ ] Migrate the files: `color_arc`, `color_field`, `color_ring`, `color_slider`, `mouse_behavior.rs`, `shape.rs`, `style.rs` (and subfolders).
- [ ] Register the new `color` module in the SDK controls entry point [mod.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/mod.rs).
- [ ] Review dependencies (e.g. `tiny-skia` or custom mathematics) and verify they compile in the SDK.
- [ ] Wire basic theme mapping configurations in `crates/look-shadcn` for styling these controls.
- [ ] Port gallery examples from the `gpui-opal` project to `gpui-luma`:
  - [ ] Port basic color control pages from `/Users/scg/Developer/GitHub/gpui-opal/apps/gallery/src/gallery/pages/color/` (specifically `arc.rs`, `field.rs`, `ring.rs`, `slider.rs`, and `slider_revealled.rs`) to `apps/gallery/src/gallery/panes/color/` in this project.
  - [ ] Port color composition pages from `/Users/scg/Developer/GitHub/gpui-opal/apps/gallery/src/gallery/pages/color_compositions/` (including the sub-components and pages like `hsv_wheel.rs`, `color_picker_photoshop.rs`, `split_ring_pixagram.rs`, `multi_mixer.rs`, and their helpers) to `apps/gallery/src/gallery/panes/color_compositions/` in this project.
  - [ ] Register the new color and color composition panes in `apps/gallery/src/gallery/panes/mod.rs` and `apps/gallery/src/gallery/panes/registry.rs`.

## Acceptance Criteria
- Basic color controls and color compositions compile and render correctly within the gallery application.
- Interactive behavior (mouse and selection) works smoothly.
- The workspace compiles cleanly without clippy warnings or test regressions.

