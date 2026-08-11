# Issue #12: Reduce Luma Studio Card Shadow Gradient by 50%

## Description
The background box shadows rendered on Card layouts in the Luma Studio are visually too prominent/large. We need to reduce the box shadow gradient specifications by approximately 50%.

## Proposed Solution
Modify `panel_box_shadow()` in the Luma Studio shared panel module to scale down the opacity, offset, and blur radius properties of the shadow.

## Tasks
- [ ] Locate `panel_box_shadow` in [common.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/luma-studio/src/studio/panels/common.rs).
- [ ] Reduce the blur radius from `px(24.0)` to `px(12.0)`.
- [ ] Reduce the vertical offset from `px(8.0)` to `px(4.0)`.
- [ ] Reduce the color alpha opacity from `0.35` to `0.18`.

## Acceptance Criteria
- Cards in the luma studio panels render with a softer, less prominent shadow that is 50% smaller in blur radius and offset, and roughly half the opacity.
