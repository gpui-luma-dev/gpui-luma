# Proposal: Declarative Layout System for GPUI-Luma

This proposal outlines the layout noise problems in `gpui-luma` panels and defines a cohesive, declarative suite of layout primitives designed to make code visual, clean, and self-documenting.

---

## 1. The Problem

Building a standard, interactive settings or control panel in GPUI currently introduces three main layers of code noise:

1. **Top-of-File Constant Bloat:** Sizing layout values (like card widths, spacing gaps, swatch diameters) are separated from the render tree as file-level constants. Tweaking a margin requires constant scrolling and disconnects layout logic from visual hierarchy.
2. **"How" vs. "What" (Geometry-Oriented Styling):** Elements that should be simple alignments or relationships are written in CSS-like geometry details:
   * To align a field right: `.min_w(px(0.0)).flex_1().child(control)`
   * To center a wheel on a ring: `.relative()`, then `.absolute().left_1_2().top_1_2().ml(-half_size).mt(-half_size)`
3. **Rust Iterator Boilerplate:** Rendering lists of elements (like swatch collections) forces visual code to be nested inside verbose `.into_iter().map(...).into_any_element()` chains.

---

## 2. The Solution: Declarative Primitives

We propose a core set of intent-revealing layout primitives that abstract CSS Flexbox layout mechanics into self-documenting structures.

### Structural Containers
* **`vstack! { gap: F, align: A; children... }`** (Existing): Lays out children vertically.
* **`hstack! { gap: F, align: A; children... }`** (Existing): Lays out children horizontally.
* **`zstack! { children... }`**: Layers children on top of each other (Z-ordering), where each child defaults to centering unless wrapped in an alignment modifier.
* **`scrollable! { child }`**: Wraps any layout in an auto-scrolling viewport with automatic clipping and mouse-wheel binding.

### Alignment Modifiers
* **`align_right! { child }`**: Inside an `hstack!`, pushes the child to the far right.
* **`align_left! { child }`**: Inside an `hstack!`, aligns the child to the far left.
* **`align_bottom! { child }`**: Inside a `vstack!`, aligns the child to the bottom.
* **`align_top! { child }`**: Inside a `vstack!`, aligns the child to the top.
* **`center! { child }`**: Centers a child horizontally and vertically.

### Relationship Layouts
* **`center_on! { target: A, base: B }`**: Centers element `A` (e.g. color wheel overlay) directly on top of element `B` (e.g. background lightness ring) without absolute margin math.
* **`labeled! { "Label Text": control }`**: Generates a standard form row with a text label formatted to the active look/theme on the left, and the control automatically aligned to the right.
* **`aspect_ratio! { ratio: F; child }`**: Forces a layout child to maintain a strict aspect ratio (e.g. a perfect square `1.0`) as it resizes.

---

## 3. Example Usage: `combinations_pane.rs`

Here is how the rendering and layout of the color combinations pane looks using the new system. 

* The constants are defined inline at the point of use.
* Absolute positioning and centering offsets are handled by `center_on!`.
* Alignment is handled explicitly with `align_right!`.
* Row labels and spacing are wrapped cleanly in `labeled!`.

```rust
impl gpui::Render for ColorCombinationsState {
    fn render(&mut self, window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let swatches = self.selected_combination.palette(self.color);
        let harmony_points = harmony_points_from_swatches(&swatches);

        self.look
            .card("color-combinations-card")
            .elevated(false)
            .child_render(move |_, _| {
                // Outer vertical stack grouping the entire card contents
                vstack! {
                    gap=28.0 align=center;

                    // 1. Stage Area: Center the wheel on the lightness ring.
                    // (Replaces absolute coordinates, negative margins, and relative wraps)
                    center_on! {
                        target: render_combo_wheel_layer(&self.wheel, &harmony_points),
                        base: &self.lightness_ring,
                    },

                    // 2. Color Field Row: Uses labeled! to manage row structure and spacing
                    labeled! {
                        "Color": hstack! {
                            gap=16.0 align=center;
                            
                            // Visual Swatch
                            div()
                                .size(px(48.0))
                                .rounded_full()
                                .border_1()
                                .border_color(self.look.chrome().border)
                                .bg(self.color),
                                
                            // Input text field aligned to the right of the row
                            align_right! {
                                &self.color_input
                            }
                        }
                    },

                    // 3. Combination Menu Row
                    labeled! {
                        "Combination": align_right! {
                            &self.harmony_menu
                        }
                    },

                    // 4. Results Swatches: Balanced columns of color preview cards
                    hstack! {
                        gap=16.0;
                        for swatch in swatches {
                            vstack! {
                                gap=10.0;
                                
                                // Color swatch box
                                div()
                                    .h(px(112.0))
                                    .rounded(px(14.0))
                                    .bg(swatch.color),
                                    
                                // Hex value text
                                div()
                                    .text_size(px(14.0))
                                    .line_height(px(20.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(self.look.chrome().body_text)
                                    .child(format_rgb_hex(swatch.color)),
                            }
                            .flex_1()
                        }
                    }
                    .w_full()
                }
                .into_any_element()
            })
            .render(window, cx)
            .w(px(520.0)) // Card width declared inline
            .max_w_full()
    }
}

// Renders the overlay dots for harmony combinations on top of the wheel.
fn render_combo_wheel_layer(
    wheel: &Entity<ColorFieldState>,
    harmony_points: &[CombinationSwatch],
) -> impl IntoElement {
    // Sizing values calculated locally
    let wheel_size = (300.0 - 2.0 * sizing::RING_THICKNESS_MEDIUM - 12.0).max(40.0);
    let center = wheel_size * 0.5;
    let marker_size = (((wheel_size * 0.07).max(10.0)) * 0.64).max(8.0);
    let marker_half = marker_size * 0.5;
    let marker_radius_max = (center - marker_half).max(0.0);

    div()
        .size(px(wheel_size))
        .relative()
        .child(wheel.clone())
        .children(harmony_points.iter().map(|swatch| {
            let angle = (swatch.color.h * 360.0).rem_euclid(360.0).to_radians();
            let marker_radius = marker_radius_max * swatch.color.s.clamp(0.0, 1.0);
            let left = center + marker_radius * angle.cos() - marker_half;
            let top = center - marker_radius * angle.sin() - marker_half;

            div()
                .absolute()
                .left(px(left))
                .top(px(top))
                .size(px(marker_size))
                .rounded_full()
                .border_1()
                .border_color(hsla(0.0, 0.0, 0.0, 0.95))
                .bg(hsla(0.0, 0.0, 1.0, 0.96))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .size(px((marker_size - 5.0).max(3.0)))
                        .rounded_full()
                        .bg(swatch.color)
                        .border_1()
                        .border_color(hsla(0.0, 0.0, 1.0, 1.0)),
                )
        }))
}
```

---

## 4. Implementation Considerations

Before moving this layout system into production code, there are four key technical and architectural decisions in Rust/GPUI that need to be addressed:

### A. Shared Label Columns (Aligning Controls Horizontally)
If you use `labeled! { "Label": control }` on consecutive rows, a short label like `"H"` and a long label like `"Combination"` will cause their respective sliders/selectors to start at different horizontal offsets, unless the labels share a layout column.
* **Option A: Fixed Label Widths (Simple):** The `labeled!` macro can default to a standard visual width for the label column (e.g., `50.0px` or `80.0px`), or allow a custom override.
  ```rust
  labeled! { width=40.0; "H": &self.slider_h }
  ```
* **Option B: Parent Grid Container (WPF SharedSizeGroup style):** A wrapping `form! { ... }` macro inspects its children and automatically forces all labels to match the width of the longest label. This is more complex to write in Rust macros but provides the cleanest developer experience.

### B. Handling GPUI `Entity<T>` vs. `impl IntoElement`
In `gpui-luma`, some controls are owned as `Entity<T>` (like `self.plane: Entity<ColorFieldState>`), while others are simple elements or layout builders. 
* The macro expansion must automatically check if the item implements `IntoElement` or if it's an `Entity<T>` that needs cloning/spawning.
* Using a helper conversion trait (e.g., `LumaLayoutElement`) can allow the macro to accept `&Entity<T>`, `Entity<T>`, or raw `Div` elements interchangeably:
  ```rust
  // The macro should expand to something that resolves the target type:
  luma_resolve_element(target)
  ```

### C. Rust Macro compilation and token parsing (`macro_rules!`)
To keep the macros fast to compile and easy to maintain, we should avoid complex token-tree parsing where possible.
* **Modular Modifiers:** A macro like `align_right! { child }` should expand to a self-contained element (e.g., a wrapper `div().flex_1().justify_end().child(child)`) rather than forcing the parent `hstack!` macro to parse and handle it uniquely. This keeps the stack macros simple.
* **Inline loop tokens:** Parsing a `for ... in ...` loop inside a macro list requires matching the `for` token pattern. We must structure the pattern rules in `vstack!` / `hstack!` to recognize iteration tokens without breaking standard expression matches.

### D. Layout Tree Performance (Yoga/Taffy Engine)
GPUI uses a Flexbox layout engine (Taffy) written in Rust. Introducing structural wrappers like `align_right!` or `center_on!` adds nested `div()` elements. 
* To ensure optimal rendering speeds, the macros should produce the absolute flat-est layout hierarchy possible (e.g., merging wrappers where possible rather than piling nested divs).
