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
- [ ] Create a color controls demonstration pane inside the Gallery.

## Acceptance Criteria
- Color field, slider, ring, and arc render and respond correctly inside the Gallery application.
- The workspace compiles cleanly without clippy warnings or test regressions.
