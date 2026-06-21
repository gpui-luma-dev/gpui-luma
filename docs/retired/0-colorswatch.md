# Issue #0-colorswatch: Implement a Reusable Color Swatch Control

## Description
In the gallery's color mixers (and other color preview areas), we render a color "swatch" that overlays a solid/semi-transparent color on top of a checkerboard background (used to indicate alpha transparency).

### The Corner Bleeding & Gap Problems
Our previous plan failed due to two distinct rendering issues in GPUI:
1. **Background Contrast Leaks**: The existing `Checkerboard` uses a canvas-based corner occluder (`paint_corner_occluder`) that draws solid triangles using the checkerboard base color `c1`. If the swatch is placed on a page background that does not match `c1` (e.g. a pure black background), these dark grey occluded corners show up as noticeable triangular gaps.
2. **Sub-pixel Corner Gaps**: Layering multiple independent rounded `div` elements (parent container with border, checkerboard wrapper, and absolute color overlay) causes sub-pixel antialiasing gaps in the corners, letting background colors peek through.

We will solve both issues systematically by implementing the swatch using a **Single-Canvas Rendering** model.

---

## Why the Solution Works (Single-Canvas Math & Insetting)

Instead of layering multiple absolute `div` elements and trying to coordinate their rounding and clipping, we render the entire swatch (base background, checkerboard grid, color overlay, and border) inside a **single GPUI canvas**.

```text
Single Canvas Bounds (bounds):
┌──────────────────────────────────────────────┐
│  Outer Bounds (bounds, radius r):            │
│   ╭──────────────────────────────────────╮   │
│   │  Inner Bounds (inset by border,      │   │
│   │                inner_r = r - border):│   │
│   │   ╭──────────────────────────────╮   │   │
│   │   │ 1. Paint c1 base (inner)     │   │   │
│   │   │ 2. Paint c2 grid (masked)    │   │   │
│   │   ╰──────────────────────────────╯   │   │
│   │  3. Paint color + border (outer)     │   │
│   ╰──────────────────────────────────────╯   │
└──────────────────────────────────────────────┘
▲ Checkerboard and underlay edges are completely occluded under the border.
```

### 1. Gamut & Background Independence
By leaving the region outside the outer rounded path completely transparent (unpainted), the swatch fits naturally onto any background (white, black, or gradients) without needing corner occluders.

### 2. Eliminating Corner Gaps & Aliasing Halos
We resolve sub-pixel double-blending and corner/bottom aliasing halos through two key rendering optimizations:
1. **Inner-Radius Inset**: We inset the base background (`c1`) and the checkerboard grid (`c2`) by the border width (`1.0px`) and use the mathematically correct inner radius `inner_r = (r - 1.0px).max(0.0)`. This guarantees that the checkerboard grid is bounds-constrained and its antialiased edges are completely hidden underneath the opaque outer border. This also prevents checkerboard leaking at the bottom and right edges.
2. **Unified Color & Border Paint Quad**: Instead of painting the color overlay and the border in separate `paint_quad` calls (which causes edge double-blending and sub-pixel haloing due to multiple compositing passes over the page background), we combine them into a single `paint_quad` call with the background set to `color` and `border_widths` set to the border width. GPUI's quad shader renders the background and border in a single pixel shader pass, ensuring pixel-perfect coverage transition at the outer edge.

### 3. Inner Corner Masking Math
Inside the canvas paint callback, we check if each checkerboard square `(row, col)` overlaps the inner rounded corners. 
For any point `(x, y)` in the corner quadrants, we calculate its distance from the corner arc center `(cx, cy)` relative to the `inner_bounds` and `inner_r`. If:
$$\Delta x^2 + \Delta y^2 > R_{inner}^2$$
where $R_{inner}$ is the inner corner radius, the point is outside the inner rounded boundary and we skip painting it.

---

## Proposed API & Control Model

### File: `crates/sdk/src/controls/color/swatch.rs`
```rust
use gpui::{AnyElement, AppContext, Corners, Edges, Hsla, IntoElement, PaintQuad, Pixels, RenderOnce, Styled, div, prelude::*, px};
use std::f32::consts::PI;

#[derive(IntoElement)]
pub struct ColorSwatch {
    color: Hsla,
    size: ControlSize,
    custom_height: Option<Pixels>,
    corner_radius: Option<Pixels>,
}

impl ColorSwatch {
    pub fn new(color: Hsla) -> Self {
        Self {
            color,
            size: ControlSize::Md,
            custom_height: None,
            corner_radius: None,
        }
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn height(mut self, height: impl Into<Pixels>) -> Self {
        self.custom_height = Some(height.into());
        self
    }

    pub fn rounded(mut self, radius: impl Into<Pixels>) -> Self {
        self.corner_radius = Some(radius.into());
        self
    }
}

impl RenderOnce for ColorSwatch {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let chrome = cx.theme().look.chrome();
        let is_dark = cx.theme().is_dark();
        let color = self.color;

        let height = self.custom_height.unwrap_or(match self.size {
            ControlSize::Sm => px(28.0),
            ControlSize::Md => px(36.0),
            ControlSize::Lg => px(44.0),
        });

        let radius = self.corner_radius.unwrap_or(px(8.0));

        // Render the entire swatch inside a single canvas
        div()
            .h(height)
            .w_full()
            .child(
                gpui::canvas(
                    move |_, _, _| (),
                    move |bounds, _, window, _| {
                        let border_width = px(1.0);
                        let inner_bounds = bounds.inset(border_width);
                        let inner_r = (radius - border_width).max(px(0.0));

                        // 1. Draw rounded base background (c1) if alpha < 1.0
                        let has_alpha = color.a < 0.999;
                        let (c1, c2) = if is_dark {
                            (hsla(0., 0., 0.1, 1.), hsla(0., 0., 0.13, 1.))
                        } else {
                            (hsla(0., 0., 1.0, 1.), hsla(0., 0., 0.95, 1.))
                        };

                        if has_alpha {
                            window.paint_quad(PaintQuad {
                                bounds: inner_bounds,
                                corner_radii: Corners::all(inner_r),
                                background: c1.into(),
                                border_widths: Edges::default(),
                                border_color: gpui::transparent_black(),
                                border_style: gpui::BorderStyle::default(),
                            });

                            // 2. Draw checkerboard grid (c2) with inner corner clipping
                            let square_size = px(8.0);
                            let rows = (inner_bounds.size.height / square_size).ceil() as i32;
                            let cols = (inner_bounds.size.width / square_size).ceil() as i32;

                            for row in 0..rows {
                                for col in 0..cols {
                                    if (row + col) % 2 == 0 {
                                        let sq_origin = inner_bounds.origin + gpui::point(square_size * (col as f32), square_size * (row as f32));
                                        
                                        // Clamp square size to avoid drawing past the bottom/right edges of inner bounds
                                        let sq_w = square_size.min(inner_bounds.size.width - square_size * col as f32);
                                        let sq_h = square_size.min(inner_bounds.size.height - square_size * row as f32);
                                        let sq_bounds = gpui::Bounds { origin: sq_origin, size: gpui::size(sq_w, sq_h) };
                                        
                                        // Skip drawing if this square is outside the rounded corner mask of inner bounds
                                        if is_square_outside_rounded_rect(sq_bounds, inner_bounds, inner_r) {
                                            continue;
                                        }

                                        window.paint_quad(PaintQuad {
                                            bounds: sq_bounds,
                                            corner_radii: Corners::default(),
                                            background: c2.into(),
                                            border_widths: Edges::default(),
                                            border_color: gpui::transparent_black(),
                                            border_style: gpui::BorderStyle::default(),
                                        });
                                    }
                                }
                            }
                        }

                        // 3. Paint solid/semi-transparent color overlay AND border in a single call to prevent sub-pixel gaps
                        window.paint_quad(PaintQuad {
                            bounds,
                            corner_radii: Corners::all(radius),
                            background: color.into(),
                            border_widths: Edges::all(border_width),
                            border_color: chrome.border,
                            border_style: gpui::BorderStyle::default(),
                        });
                    }
                )
                .size_full()
            )
    }
}

### 3. Corner Masking & Aliasing Prevention Math
Inside the canvas paint callback, we check if each checkerboard square `(row, col)` overlaps the inner rounded corners of `inner_bounds`.

If we only check the *center* of the `8px` square, the outer corners of the square will bleed up to `4px` beyond the rounded boundary, creating jagged aliasing artifacts in the corners. To ensure perfect clipping, we check **all four corners** of the square. If **any** of the four corners is outside the rounded boundary, we skip drawing the square.

To prevent coordinate variables from leaking across branches and causing compiler warnings or bugs, each quadrant check is performed inside an isolated scope:

```rust
/// Helper function to check if a checkerboard square is outside the rounded rect boundary.
/// Returns true if any of the four corners of the square fall outside the rounded rect bounds.
fn is_square_outside_rounded_rect(sq: gpui::Bounds<Pixels>, rect: gpui::Bounds<Pixels>, r: Pixels) -> bool {
    let r_f32 = r.as_f32();
    if r_f32 <= 0.0 {
        return false;
    }

    let left = rect.origin.x.as_f32();
    let top = rect.origin.y.as_f32();
    let right = (rect.origin.x + rect.size.width).as_f32();
    let bottom = (rect.origin.y + rect.size.height).as_f32();

    // Check all 4 corners of the checkerboard square
    let sq_left = sq.origin.x.as_f32();
    let sq_top = sq.origin.y.as_f32();
    let sq_right = (sq.origin.x + sq.size.width).as_f32();
    let sq_bottom = (sq.origin.y + sq.size.height).as_f32();

    let corners = [
        (sq_left, sq_top),     // Top-Left corner of square
        (sq_right, sq_top),    // Top-Right corner of square
        (sq_left, sq_bottom),  // Bottom-Left corner of square
        (sq_right, sq_bottom), // Bottom-Right corner of square
    ];

    for &(x, y) in &corners {
        // 1. Top-Left quadrant
        {
            let cx = left + r_f32;
            let cy = top + r_f32;
            if x < cx && y < cy {
                let dx = x - cx;
                let dy = y - cy;
                if dx * dx + dy * dy > r_f32 * r_f32 {
                    return true;
                }
            }
        }

        // 2. Top-Right quadrant
        {
            let cx = right - r_f32;
            let cy = top + r_f32;
            if x > cx && y < cy {
                let dx = x - cx;
                let dy = y - cy;
                if dx * dx + dy * dy > r_f32 * r_f32 {
                    return true;
                }
            }
        }

        // 3. Bottom-Left quadrant
        {
            let cx = left + r_f32;
            let cy = bottom - r_f32;
            if x < cx && y > cy {
                let dx = x - cx;
                let dy = y - cy;
                if dx * dx + dy * dy > r_f32 * r_f32 {
                    return true;
                }
            }
        }

        // 4. Bottom-Right quadrant
        {
            let cx = right - r_f32;
            let cy = bottom - r_f32;
            if x > cx && y > cy {
                let dx = x - cx;
                let dy = y - cy;
                if dx * dx + dy * dy > r_f32 * r_f32 {
                    return true;
                }
            }
        }
    }

    false
}
```

---

## Migration Plan
1. Implement the `ColorSwatch` inside `crates/sdk/src/controls/color/swatch.rs`.
2. Refactor `apps/gallery/src/gallery/panes/color/multi_mixer_pane.rs` to render swatches using `ColorSwatch`.
3. Verify that the gaps are eliminated and corners are fully transparent.
