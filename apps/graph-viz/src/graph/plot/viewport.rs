use gpui::{Bounds, Pixels, Point, point, px, size};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChartMargins {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl ChartMargins {
    pub const DEFAULT: Self = Self { left: 44.0, right: 12.0, top: 10.0, bottom: 24.0 };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport2D {
    pub bounds: Bounds<Pixels>,
}

impl Viewport2D {
    pub fn new(bounds: Bounds<Pixels>) -> Self {
        Self { bounds }
    }

    pub fn with_margins(outer: Bounds<Pixels>, margins: ChartMargins) -> Self {
        Self {
            bounds: Bounds {
                origin: point(outer.left() + px(margins.left), outer.top() + px(margins.top)),
                size: size(
                    (outer.size.width - px(margins.left + margins.right)).max(px(1.0)),
                    (outer.size.height - px(margins.top + margins.bottom)).max(px(1.0)),
                ),
            },
        }
    }

    #[allow(dead_code)]
    pub fn inset(outer: Bounds<Pixels>, inset: f32) -> Self {
        Self::with_margins(outer, ChartMargins { left: inset, right: inset, top: inset, bottom: inset })
    }

    /// Maps normalized UV to pixel coordinates. UV Y = 0.0 is the bottom of the viewport.
    pub fn uv_to_pixels(&self, uv: Point<f32>) -> Point<Pixels> {
        let x = self.bounds.origin.x + px(uv.x * self.bounds.size.width.as_f32());
        let y = self.bounds.origin.y + px((1.0 - uv.y) * self.bounds.size.height.as_f32());
        point(x, y)
    }

    pub fn data_to_pixels(&self, domain: &super::PlotDomain2D, x: f32, y: f32) -> Point<Pixels> {
        self.uv_to_pixels(domain.to_uv(x, y))
    }

    pub fn fraction_to_x(&self, fraction: f32) -> Pixels {
        self.bounds.origin.x + self.bounds.size.width * fraction.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::plot::PlotDomain2D;

    #[test]
    fn uv_to_pixels_inverts_y_for_gpui_space() {
        let viewport = Viewport2D::new(Bounds { origin: point(px(0.0), px(0.0)), size: size(px(100.0), px(100.0)) });
        let bottom = viewport.uv_to_pixels(gpui::point(0.0, 0.0));
        let top = viewport.uv_to_pixels(gpui::point(0.0, 1.0));
        assert!(top.y < bottom.y);
    }

    #[test]
    fn data_to_pixels_places_higher_values_toward_top() {
        let domain = PlotDomain2D::new(0.0..=10.0, 0.0..=100.0);
        let viewport = Viewport2D::new(Bounds { origin: point(px(0.0), px(0.0)), size: size(px(100.0), px(100.0)) });
        let low = viewport.data_to_pixels(&domain, 0.0, 0.0);
        let high = viewport.data_to_pixels(&domain, 0.0, 100.0);
        assert!(high.y < low.y);
    }

    #[test]
    fn with_margins_reserves_label_space() {
        let outer = Bounds { origin: point(px(0.0), px(0.0)), size: size(px(200.0), px(100.0)) };
        let viewport = Viewport2D::with_margins(outer, ChartMargins::DEFAULT);
        assert_eq!(
            viewport.bounds.size.width.as_f32(),
            200.0 - ChartMargins::DEFAULT.left - ChartMargins::DEFAULT.right
        );
        assert_eq!(
            viewport.bounds.size.height.as_f32(),
            100.0 - ChartMargins::DEFAULT.top - ChartMargins::DEFAULT.bottom
        );
    }
}
