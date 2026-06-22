# Basic Slider Review: Rounded Edges & GPUI Clipping Limitation

This document contains a deep technical review of the standard `Slider` control regarding its rounded corner rendering limitations in GPUI.

---

## 1. Root Cause: GPUI `overflow_hidden` Limitation
As documented in [rounded_shell.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/rounded_shell.rs#L1-L6):
* **Rectangular Clipping Only**: In GPUI, calling `.overflow_hidden()` on a parent element only masks its child descendants using a **rectangular boundary**. It does not clip children to the parent's rounded corner paths (`corner_radii`).
* **Overlap / Corner Coverage**: When a child element with a background color spans to the edges of a rounded parent container, its square corners overlap and obscure the parent's rounded ends.

---

## 2. How This Affects the Basic Slider

### Standard [Slider](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/slider/mod.rs#L21)
In [ThemedSliderTemplate::render](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/slider/template.rs#L112-L121), the active fill is rendered as an overlapping child `div`:
```rust
let track = div()
    .rounded(track_radius)
    .overflow_hidden()
    .child(
        div()
            .h_full()
            .w(relative(percentage))
            .bg(look.fill_background)
            .rounded(track_radius), // Double-rounded issue
    )
```
* **The Problem**: Because `overflow_hidden` cannot clip the child, the fill child must apply `.rounded(track_radius)` to avoid poking out of the track's left end.
* **The Visual Bug**: This causes the right end of the active fill (where it meets the unfilled track background) to be incorrectly rounded, rather than terminating in a flat, vertical edge.

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

### For standard `SliderTemplate`:
Avoid overlapping child elements entirely. Split the track into two side-by-side elements:
1. **Active Fill**: Spans `0.0..=percentage`. Apply rounding ONLY to the left corners: `.rounded_tl(track_radius).rounded_bl(track_radius)`.
2. **Unfilled Track**: Spans `percentage..=1.0`. Apply rounding ONLY to the right corners: `.rounded_tr(track_radius).rounded_br(track_radius)`.
3. *Note*: If `percentage == 1.0` or `0.0`, fall back to rounding all corners of the single visible segment.
