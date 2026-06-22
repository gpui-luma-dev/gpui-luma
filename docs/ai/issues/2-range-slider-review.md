# Range Slider Review: Rounded Edges & GPUI Clipping Limitation

This document contains a deep technical review of the `RangeSlider` control regarding its rounded corner rendering limitations in GPUI.

---

## 1. Root Cause: GPUI `overflow_hidden` Limitation
As documented in [rounded_shell.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/rounded_shell.rs#L1-L6):
* **Rectangular Clipping Only**: In GPUI, calling `.overflow_hidden()` on a parent element only masks its child descendants using a **rectangular boundary**. It does not clip children to the parent's rounded corner paths (`corner_radii`).
* **Overlap / Corner Coverage**: When a child element with a background color spans to the edges of a rounded parent container, its square corners overlap and obscure the parent's rounded ends.

---

## 2. How This Affects RangeSlider

### [RangeSlider](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/range_slider/mod.rs#L26)
In [ThemedRangeSliderTemplate::render](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/range_slider/template.rs#L114-L130), gaps are overlayed over the active fill track:
```rust
.child(div().absolute().inset_0().bg(look.fill_background).rounded(track_radius))
.children(
    model.track_segments.iter().filter(|segment| segment.kind == RangeSliderSegmentKind::Gap).map(
        |segment| {
            div()
                .absolute()
                .left(relative(start))
                .w(relative(width))
                .bg(look.track_background) // Square corners
        },
    ),
)
```
* **The Visual Bug**: If a gap segment is positioned at the start ($0.0$) or end ($1.0$) of the track, its square overlay `div` obscures the rounded corners of the parent track, resulting in square/flat slider bar ends.

---

## 3. Established Solution Patterns in Luma
The codebase already implements a robust pattern to bypass this GPUI limitation in the [ColorSlider](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/color/color_slider/surface.rs#L17) delegates:

* **Outer-Edge-Only Rounding**: In [HueDelegate](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/color/color_slider/delegates.rs#L154-L157), the track is split into non-overlapping bands. Dynamic corner rounding is applied *only* to the outermost edges of the first and last segments:
```rust
let mut corner_radii = Corners::default();
apply_edge_corner_radii(
    &mut corner_radii, 
    axis, 
    corner_radius, 
    i == 0,             // Round start of track
    i == bands.len() - 1 // Round end of track
);
```

---

## 4. Recommended Fixes

### For `RangeSliderTemplate`:
Apply dynamic corner radii to the track segments based on their boundary percentages:
* If a segment starts at `0.0`, round its left corners (`top_left`/`bottom_left`).
* If a segment ends at `1.0`, round its right corners (`top_right`/`bottom_right`).
* If a segment spans the entire track (`0.0` to `1.0`), round all corners.
* Keep all middle segments completely square.
