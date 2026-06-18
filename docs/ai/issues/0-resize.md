# Issue #0: Slider Layout Sizing and Responsive Resizing

## Description
The standard horizontal [Slider](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/slider/mod.rs#L21) control in the SDK is designed with a fixed track width resolved from the active theme/stylesheet (`appearance.width` / `long_axis`). 

When placed inside layout containers that scale dynamically as the window or sidebar width changes (such as the Luma Theme Studio sidebar), the slider does not dynamically resize its track or position its thumb knob. Instead:
- The track remains locked at the theme's resolved pixel width (e.g. `260px` in [DefaultSliderTheme::resolve](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/slider/theme.rs#L44-L81)).
- If parent containers restrict the width and specify `.overflow_hidden()`, the right portion of the slider and its thumb knob are clipped.
- While the visual elements remain static, the actual hit-test/interaction bounds (determined by layout-phase canvas measurements) *do* stretch/shrink. This leads to a bug where the visual knob position is desynchronized from the actual value registered by clicking or dragging.

---

## Motivation (Specific Instance)
The primary motivation for this layout correction is the **Other** tab in the Luma Theme Studio sidebar ([theme_sidebar.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/theme-studio/src/studio/theme_sidebar.rs)):

- The sidebar contains two sliders for adjusting **Radius** and **Spacing** metrics.
- The layout row is constructed using `hstack!` inside [metric_category_content](file:///Users/scg/Developer/GitHub/gpui-luma/apps/theme-studio/src/studio/theme_sidebar.rs#L608):
  ```rust
  hstack! {
      gap=10 align=center;
      div()
          .flex_shrink_0()
          .child(label),
      div()
          .flex_1()
          .min_w(px(0.0))
          .overflow_hidden()
          .child(slider),
      div()
          .w(px(METRIC_FIELD_WIDTH))
          .flex_shrink_0()
          .child(field),
      div()
          .flex_shrink_0()
          .child("rem"),
  }
  ```
- Because the slider is wrapped in a flex child with `.overflow_hidden()`, shrinking the sidebar cuts off the right half of the slider (clipping the track and hiding the thumb knob completely at higher percentages).

---

## Proposed Solution (Fluid Layout via Relative/Percentage Sizing)

To resolve this issue cleanly and support fluid sizing, the [ThemedSliderTemplate](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/slider/template.rs#L38) should be updated to size and position its inner track and thumb relative to the actual width allocated by the layout engine (Taffy), rather than hardcoding the theme's `appearance.width`.

### 1. Relative Template Rendering
For a horizontal orientation:
* **Track**: Render the track container using `.w_full()` instead of a fixed `.w(px(track_width))`.
* **Fill**: Render the colored fill bar using relative width `.w(relative(percentage))` instead of `.w(px(track_width * percentage))`.
* **Thumb**: Position the thumb knob using `.left(relative(percentage))` combined with a negative left margin `.margin_left(px(-appearance.thumb_size / 2.0))` to center the thumb over its actual value.
* **Padding**: Add horizontal padding of `.px(px(appearance.thumb_size / 2.0))` to the slider root container to ensure the thumb does not overflow or clip when positioned at the extreme ends (`0%` and `100%`).
* **Precise Interaction Bounds**: Position the hit-testing canvas exactly over the track bounds (e.g. by making it a child of the track rather than the full root container), so that `percentage_from_position` maps inputs precisely to the active track length.

---

## Clean Layout Refactoring with `dock_panel!`

Once the slider is fluid, the sidebar row can be simplified. Instead of using verbose nested flexbox containers with custom `flex_1()`, `flex_shrink_0()`, and `min_w(px(0.0))` properties, we can utilize Luma's declarative `dock_panel!` macro to lay out the row components cleanly.

### Old Flexbox Row Layout
```rust
hstack! {
    gap=10 align=center;
    div()
        .flex_shrink_0()
        .typography_style(row_label_typography)
        .text_color(chrome.body_text)
        .child(label),
    div()
        .flex_1()
        .min_w(px(0.0))
        .overflow_hidden()
        .child(slider),
    div()
        .w(px(METRIC_FIELD_WIDTH))
        .flex_shrink_0()
        .child(field),
    div()
        .flex_shrink_0()
        .typography_style(unit_style)
        .text_color(chrome.muted_text)
        .child("rem"),
}
```

### New Declarative DockPanel Row Layout
```rust
dock_panel! {
    left: div()
        .typography_style(row_label_typography)
        .text_color(chrome.body_text)
        .child(label),
    right: hstack! {
        gap=10 align=center;
        div()
            .w(px(METRIC_FIELD_WIDTH))
            .child(field),
        div()
            .typography_style(unit_style)
            .text_color(chrome.muted_text)
            .child("rem"),
    },
    fill: slider
}
.w_full()
.h(px(root_height))
```

This layout represents a clear, robust UI pattern:
- The variable-width label is docked on the **left**.
- The fixed-width inputs and labels are docked together on the **right**.
- The responsive, fluid slider fills the remaining space (**fill**) and resizes gracefully as the parent width changes.

---

## Impact on Gallery Slider Demo Pane

The Gallery app contains a slider demo pane ([pane.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/slider/pane.rs)) that displays the slider and its state previews in a centered column. Because horizontal sliders will become fully fluid (`w_full()`) by default:

* **The Problem**: The slider inside the gallery's `items_center()` flex column will stretch to fill the entire main pane width (potentially 800px+), which is visually unappealing and makes fine-grained slider adjustments difficult.
* **The Update Needed**: The gallery pane's render function must be updated to wrap the slider inside a width-constrained container (e.g. `.w(px(260.0))` or `.max_w(px(320.0))`).


