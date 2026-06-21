//! Reusable color swatch rendering.
//!
//! This control renders a color preview with alpha transparency support.
//! It uses a single canvas so the checkerboard, color fill, and border are
//! painted in one place, which avoids the corner bleeding and sub-pixel gaps
//! caused by layered rounded `div`s in GPUI.

use gpui::{prelude::*, *};

use crate::controls::color::style::ActiveTheme;
use crate::theme::ControlSize;

/// A reusable color preview swatch.
///
/// The swatch is lookless: it resolves theme colors from the active GPUI
/// theme, but its rendering strategy is independent of any product-specific
/// palette or layout code.
///
/// Rendering model:
/// - The outer swatch is drawn in a single canvas pass.
/// - The color fill and border are painted together in one `PaintQuad` to avoid
///   haloing at the edge.
/// - When the color has alpha, the checkerboard is drawn inside an inset inner
///   bounds so the border occludes any checkerboard edge antialiasing.
#[derive(IntoElement)]
pub struct ColorSwatch {
    color: Hsla,
    size: ControlSize,
    custom_height: Option<Pixels>,
    corner_radius: Option<Pixels>,
    checkerboard: bool,
}

impl ColorSwatch {
    /// Creates a swatch for the given color.
    ///
    /// The default sizing is `ControlSize::Md`, but callers can override the
    /// height or rounded radius after construction.
    pub fn new(color: impl Into<Hsla>) -> Self {
        Self {
            color: color.into(),
            size: ControlSize::Md,
            custom_height: None,
            corner_radius: None,
            checkerboard: true,
        }
    }

    /// Sets the semantic swatch size.
    ///
    /// This picks a default height and corner radius from the control-size
    /// scale when the caller has not provided explicit values.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    /// Overrides the swatch height.
    ///
    /// This is used by gallery layouts that need a fixed preview strip while
    /// still keeping the control reusable elsewhere.
    pub fn height(mut self, height: impl Into<Pixels>) -> Self {
        self.custom_height = Some(height.into());
        self
    }

    /// Overrides the corner radius used for the outer swatch and the inner
    /// checkerboard mask.
    ///
    /// The same radius is intentionally reused across all painted layers so the
    /// swatch does not exhibit corner seams or background bleed.
    pub fn rounded(mut self, radius: impl Into<Pixels>) -> Self {
        self.corner_radius = Some(radius.into());
        self
    }

    /// Disables the checkerboard underlay.
    ///
    /// Use this for opaque previews where the alpha indicator would be visual
    /// noise, or when the caller wants a pure solid swatch fill.
    pub fn disable_checkerboard(mut self) -> Self {
        self.checkerboard = false;
        self
    }

    fn height_for_size(size: ControlSize) -> Pixels {
        match size {
            ControlSize::Sm => px(28.0),
            ControlSize::Md => px(36.0),
            ControlSize::Lg => px(44.0),
        }
    }

    fn default_radius(size: ControlSize) -> Pixels {
        match size {
            ControlSize::Sm => px(6.0),
            ControlSize::Md => px(8.0),
            ControlSize::Lg => px(12.0),
        }
    }
}

impl RenderOnce for ColorSwatch {
    /// Renders the swatch as a single canvas-backed element.
    ///
    /// The paint order is deliberate:
    /// 1. Compute the outer radius and derive a 1px inset inner radius.
    /// 2. If the color is translucent, paint an inner base fill and masked
    ///    checkerboard inside the inset bounds.
    /// 3. Paint the actual color and border together in one `PaintQuad` so the
    ///    outer edge is composed once.
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let chrome = cx.theme();
        let is_dark = chrome.is_dark();
        let height = self.custom_height.unwrap_or_else(|| Self::height_for_size(self.size));
        let radius = self.corner_radius.unwrap_or_else(|| Self::default_radius(self.size));
        let color = self.color;
        let border_width = px(1.0);
        let corner_radii = Corners::all(radius);

        div().relative().w_full().h(height).child(
            canvas(
                move |_, _, _| (),
                move |bounds, _, window, _| {
                    let (c1, c2) = if is_dark {
                        (hsla(0., 0., 0.1, 1.), hsla(0., 0., 0.13, 1.))
                    } else {
                        (hsla(0., 0., 1.0, 1.), hsla(0., 0., 0.95, 1.))
                    };

                    let inner_bounds = bounds.inset(border_width);
                    let inner_r = (radius - border_width).max(px(0.0));

                    if self.checkerboard
                        && color.a < 0.999
                        && inner_bounds.size.width.as_f32() > 0.0
                        && inner_bounds.size.height.as_f32() > 0.0
                    {
                        window.paint_quad(PaintQuad {
                            bounds: inner_bounds,
                            corner_radii: Corners::all(inner_r),
                            background: c1.into(),
                            border_widths: Edges::default(),
                            border_color: transparent_black(),
                            border_style: BorderStyle::default(),
                        });

                        paint_checkerboard(window, inner_bounds, inner_r, c2);
                    }

                    window.paint_quad(PaintQuad {
                        bounds,
                        corner_radii,
                        background: color.into(),
                        border_widths: Edges::all(border_width),
                        border_color: chrome.border,
                        border_style: BorderStyle::default(),
                    });
                },
            )
            .absolute()
            .size_full(),
        )
    }
}

/// Paints the alternating checkerboard squares inside the provided bounds.
///
/// Squares are clamped to the available bounds so the last row/column does not
/// overrun the swatch edge, and each square is skipped if any of its corners
/// would fall outside the rounded inner mask.
fn paint_checkerboard(window: &mut Window, bounds: Bounds<Pixels>, radius: Pixels, color: Hsla) {
    let square_size = px(8.0);
    let rows = (bounds.size.height / square_size).ceil() as i32;
    let cols = (bounds.size.width / square_size).ceil() as i32;

    for row in 0..rows {
        for col in 0..cols {
            if (row + col) % 2 == 0 {
                let origin = bounds.origin + point(square_size * (col as f32), square_size * (row as f32));
                let sq_w = square_size.min(bounds.size.width - square_size * col as f32);
                let sq_h = square_size.min(bounds.size.height - square_size * row as f32);
                let square_bounds = Bounds { origin, size: size(sq_w, sq_h) };

                if !is_square_outside_rounded_rect(square_bounds, bounds, radius) {
                    window.paint_quad(PaintQuad {
                        bounds: square_bounds,
                        corner_radii: Corners::default(),
                        background: color.into(),
                        border_widths: Edges::default(),
                        border_color: transparent_black(),
                        border_style: BorderStyle::default(),
                    });
                }
            }
        }
    }
}

/// Returns `true` when any corner of `square` lies outside the rounded rect.
///
/// This helper uses the four-corner test described in the swatch issue notes.
/// That conservative test intentionally drops partially intersecting squares so
/// their antialiased edges cannot leak beyond the rounded boundary.
fn is_square_outside_rounded_rect(square: Bounds<Pixels>, rect: Bounds<Pixels>, radius: Pixels) -> bool {
    let r_f32 = radius.as_f32();
    if r_f32 <= 0.0 {
        return false;
    }

    let left = rect.origin.x.as_f32();
    let top = rect.origin.y.as_f32();
    let right = (rect.origin.x + rect.size.width).as_f32();
    let bottom = (rect.origin.y + rect.size.height).as_f32();

    let sq_left = square.origin.x.as_f32();
    let sq_top = square.origin.y.as_f32();
    let sq_right = (square.origin.x + square.size.width).as_f32();
    let sq_bottom = (square.origin.y + square.size.height).as_f32();

    let corners = [(sq_left, sq_top), (sq_right, sq_top), (sq_left, sq_bottom), (sq_right, sq_bottom)];

    for &(x, y) in &corners {
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
