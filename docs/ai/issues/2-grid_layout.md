# Issue #2-GL: WPF-Style Grid Layout Panel

## Description
In visual design interfaces (such as `theme-studio` control sidebars), aligning labels, sliders, text inputs, and unit suffixes across separate stacked rows using basic nested flexboxes (`hstack!`/`vstack!`) is fragile and visually inconsistent. Because different labels have varying text lengths and some sliders lack unit suffixes, the controls become unaligned. Hardcoding layout widths (e.g. `.w(px(60.0))` on labels) acts as a temporary workaround but fails under localization or layout size updates.

To solve this systematically, we will implement a WPF-inspired **`grid_layout!`** panel. This layout control will support track definitions (`RowDefinitions` and `ColumnDefinitions`) with Star and Pixel sizing, allowing child elements to be explicitly positioned at coordinates `(row, col)` with optional spanning (`colspan`). 

This document defines the selected architecture: **Strategy A (Flexbox Emulation)**. Strategy B (Custom Layout Elements) was evaluated and rejected due to GPUI hit-testing and event dispatch mechanics.

---

## The Rejection of Strategy B & Sizing Model Decisions
* **Strategy B (Custom Element Layout) Rejection**: GPUI delegates mouse hovers, clicks, drags, and focus tracking using layout coordinates computed and cached in the layout tree during the `request_layout` phase. Overriding cell coordinates manually during `paint` breaks this hitbox mapping, making standard interactive controls (like sliders and textfields) completely non-responsive to user input.
* **Banning Direct `GridTrack::Auto`**: A row-first flex grid cannot dynamically align auto-sized columns across different row groups in a single flexbox pass. Therefore, we **do not support `GridTrack::Auto`** in `GridTrack`. Columns requiring alignment must use explicit `Px` or `Star` sizes.

### The Solution: Static Text Pre-Measurement
To achieve the exact behavior of an "Auto" label column without breaking layout alignment, the caller pre-measures the text labels statically using GPUI's text layout system and passes the computed maximum width to `GridTrack::Px`.

---

## Features
* **Track-Based Sizing**: Define columns using `GridTrack` sizing constraints (`Px` for fixed sizes, `Star` for proportional fill).
* **Coordinate-Based Child Placement**: Position child elements at absolute coordinate indexes `[row, col]` with support for horizontal cell spanning (`colspan`).
* **Text Pre-Measurement Integration**: Emulates dynamic auto-sizing by accepting computed pixel measurements directly into `GridTrack::Px`.
* **Interactive Alignment**: Maintains GPUI's built-in event-routing system, ensuring text inputs, sliders, and buttons remain fully responsive and focus-ring compatible.
* **Declarative Macro Syntax**: A clean, self-documenting syntax mimicking WPF/XAML grid definitions.

---

## Technical Design & API

### 1. Sizing Model (`GridTrack`)
We will define an enum representing column sizing rules in [layout.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/layout.rs):

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GridTrack {
    /// Fixed width in pixels (WPF: Px)
    Px(f32),
    /// Proportional share of the remaining space (WPF: Star / *)
    Star(f32),
}
```

### 2. Emulating "Auto" columns (Pre-Measurement Helper)
For text label columns, the caller defines a helper function to compute the column's maximum width:

```rust
fn shadow_grid_label_width(sidebar: &ThemeSidebar, window: &mut Window) -> f32 {
    let text_system = window.text_system();
    let theme = &sidebar.vm.look;
    let typography = theme.typography_scale(ShadcnTextSize::Xs);
    let font_size = px(typography.size);
    let font = gpui::font(typography.font_family.clone());
    
    let labels = ["Opacity", "Blur", "Spread", "Offset X", "Offset Y"];
    let mut max_width = 0.0;
    
    for label in labels {
        let run = gpui::TextRun {
            len: label.len(),
            font: {
                let mut f = font.clone();
                f.weight = gpui::FontWeight::NORMAL;
                f
            },
            color: gpui::black(),
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        if let Ok(shaped) = text_system.shape_line(label.into(), font_size, &[run], None) {
            max_width = max_width.max(shaped.width.as_f32());
        }
    }
    
    max_width
}
```

### 3. Builder Pattern & Macro Usage
The grid layout builder compiles down to nested GPUI `div()` containers.

```rust
fn shadow_slider_grid(sidebar: &ThemeSidebar, window: &mut Window) -> AnyElement {
    grid_layout! {
        rows: 5,
        columns: [
            GridTrack::Px(shadow_grid_label_width(sidebar, window)),
            GridTrack::Star(1.0),
            GridTrack::Px(METRIC_FIELD_WIDTH),
            GridTrack::Px(SHADOW_GRID_UNIT_WIDTH),
        ],
        gap_x: SLIDER_FIELD_GRID_GAP_X,
        gap_y: SHADOW_SECTION_GAP;
        [0, 0] => slider_field_grid_label(sidebar, "Opacity"),
        [0, 1] => sidebar.vm.shadow_opacity_slider.clone(),
        [0, 2] => sidebar.vm.shadow_opacity_field.clone(),
        [0, 3] => slider_field_grid_unit(sidebar, ""),
        [1, 0] => slider_field_grid_label(sidebar, "Blur"),
        [1, 1] => sidebar.vm.shadow_blur_slider.clone(),
        [1, 2] => sidebar.vm.shadow_blur_field.clone(),
        [1, 3] => slider_field_grid_unit(sidebar, "px"),
        [2, 0] => slider_field_grid_label(sidebar, "Spread"),
        [2, 1] => sidebar.vm.shadow_spread_slider.clone(),
        [2, 2] => sidebar.vm.shadow_spread_field.clone(),
        [2, 3] => slider_field_grid_unit(sidebar, "px"),
        [3, 0] => slider_field_grid_label(sidebar, "Offset X"),
        [3, 1] => sidebar.vm.shadow_offset_x_slider.clone(),
        [3, 2] => sidebar.vm.shadow_offset_x_field.clone(),
        [3, 3] => slider_field_grid_unit(sidebar, "px"),
        [4, 0] => slider_field_grid_label(sidebar, "Offset Y"),
        [4, 1] => sidebar.vm.shadow_offset_y_slider.clone(),
        [4, 2] => sidebar.vm.shadow_offset_y_field.clone(),
        [4, 3] => slider_field_grid_unit(sidebar, "px"),
    }
    .into_any_element()
}
```

---

## Grid Compiler Layout Logic (`IntoElement`)

When building the container, `GridLayout` compiles down to nested GPUI `div()` containers. To render:

1. **Sort & Group Elements**:
   Group `children` by row index `0..row_count`.
2. **Build Vertically**:
   Instantiate an outer container `div().flex().flex_col().gap(px(gap)).size_full()`.
3. **Build Horizontal Rows**:
   For each row `r`:
   * Instantiate a row wrapper: `div().flex().w_full().gap(px(gap))` (or rely on column gaps).
   * Loop through each column `c` in `0..columns.len()`:
     * Check if a child starts at `(r, c)`.
     * **If a child exists**:
       * Check its `col_span`. Compute the layout rule for the span:
         * Sum up widths for fixed columns: `width = sum(Px(w))` across the spanned columns.
         * Sum up weights for star columns: `weight = sum(Star(w))` across the spanned columns.
       * Wrap the child element in a cell `div()` with the computed size:
         * **Fixed Columns**: `.w(px(width)).flex_shrink_0()`
         * **Star Columns**: `.flex_grow(weight).min_w(px(0.0))` (setting min width protects elements from shrinking under flex constraints).
       * Append cell to row wrapper. Skip the next `col_span - 1` columns in row loop.
     * **If no child exists (Empty Cell/Spacer)**:
       * Render an empty placeholder cell `div()` with the column's defined width/weight constraints. This acts as a structural column spacer, guaranteeing that adjacent cells in subsequent rows remain aligned.
   * Append row wrapper to outer vertical container.

---

## Tasks

### Phase 1: SDK Grid Layout Model (`crates/sdk`)
- [ ] Define `GridTrack`, `GridChild`, and `GridLayout` inside `crates/sdk/src/layout.rs`.
- [ ] Implement the `gpui::IntoElement` trait for `GridLayout` using the vertical/horizontal flex generation loop.
- [ ] Verify handling of cell spans (`colspan`) and ensure structural empty spacer cells are inserted correctly.

### Phase 2: Macro DSL Implementation (`crates/sdk`)
- [ ] Add the `grid_layout!` macro to `crates/sdk/src/macros.rs` with parsing support.
- [ ] Export `grid_layout!` and `GridLayout` via the SDK prelude.

### Phase 3: Integration & Alignment Tuning
- [ ] Replace the nested `hstack!` slider rows in [other.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/theme-studio/src/studio/theme_sidebar/panels/other.rs) with the new `grid_layout!` implementation.
- [ ] Implement text measurements for label fields to drive the dynamic label column `GridTrack::Px` calculation.
- [ ] Run verification tests ensuring slider thumbs, text values, and empty suffix fields align vertically across all category blocks.
- [ ] Confirm code formatting (`cargo fmt`), lint warnings (`cargo clippy`), and existing tests pass cleanly.

---

## Acceptance Criteria
- Controls of varying text and suffix lengths placed inside a `grid_layout!` align horizontally and vertically.
- Track-sizing bounds (`Px`, `Star`) scale dynamically when the sidebar or parent card is resized.
- Columns remain perfectly aligned across different row records, even when middle cells are skipped (empty spacer cells function correctly).
- Hit-testing (dragging sliders, clicking text fields) resolves accurately, maintaining standard interactivity.
- The project compiles successfully without any warnings or layout computation panics.
