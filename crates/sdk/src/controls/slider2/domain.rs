use gpui::{
    Bounds, Hsla, Pixels, Window, hsla, linear_color_stop, linear_gradient, point, px, size, transparent_black,
    BorderStyle, Edges, PaintQuad,
};

use super::layout::{apply_outer_edge_corner_radii, track_corner_radii};
use super::model::Slider2Orientation;

pub trait DomainTrackRenderer: Send + Sync {
    fn paint(&self, bounds: Bounds<Pixels>, orientation: Slider2Orientation, reversed: bool, window: &mut Window);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct HueDomainTrack;

impl DomainTrackRenderer for HueDomainTrack {
    fn paint(&self, bounds: Bounds<Pixels>, orientation: Slider2Orientation, reversed: bool, window: &mut Window) {
        let is_vertical = orientation == Slider2Orientation::Vertical;
        let total_size = if is_vertical {
            bounds.size.height.as_f32()
        } else {
            bounds.size.width.as_f32()
        };
        let band_count = 6;
        let band_size = total_size / band_count as f32;
        let default_radius = if is_vertical {
            bounds.size.width.as_f32()
        } else {
            bounds.size.height.as_f32()
        } * 0.5;
        let corner_radius = px(default_radius);

        let mut bands = [
            (hsla(0.0, 1.0, 0.5, 1.0), hsla(60.0 / 360.0, 1.0, 0.5, 1.0)),
            (hsla(60.0 / 360.0, 1.0, 0.5, 1.0), hsla(120.0 / 360.0, 1.0, 0.5, 1.0)),
            (hsla(120.0 / 360.0, 1.0, 0.5, 1.0), hsla(180.0 / 360.0, 1.0, 0.5, 1.0)),
            (hsla(180.0 / 360.0, 1.0, 0.5, 1.0), hsla(240.0 / 360.0, 1.0, 0.5, 1.0)),
            (hsla(240.0 / 360.0, 1.0, 0.5, 1.0), hsla(300.0 / 360.0, 1.0, 0.5, 1.0)),
            (hsla(300.0 / 360.0, 1.0, 0.5, 1.0), hsla(1.0, 1.0, 0.5, 1.0)),
        ];

        if reversed {
            bands.reverse();
            for band in bands.iter_mut() {
                let (start, end) = *band;
                *band = (end, start);
            }
        }

        for (index, (start_color, end_color)) in bands.iter().enumerate() {
            let start_offset = band_size * index as f32;
            let band_bounds = if is_vertical {
                Bounds {
                    origin: point(bounds.origin.x, bounds.origin.y + px(start_offset)),
                    size: size(bounds.size.width, px(band_size)),
                }
            } else {
                Bounds {
                    origin: point(bounds.origin.x + px(start_offset), bounds.origin.y),
                    size: size(px(band_size), bounds.size.height),
                }
            };

            let mut corner_radii = track_corner_radii(orientation, px(0.0));
            apply_outer_edge_corner_radii(
                &mut corner_radii,
                orientation,
                corner_radius,
                index == 0,
                index == band_count - 1,
            );

            let angle = if is_vertical { 180.0 } else { 90.0 };
            window.paint_quad(PaintQuad {
                bounds: band_bounds,
                corner_radii,
                background: linear_gradient(
                    angle,
                    linear_color_stop(*start_color, 0.0),
                    linear_color_stop(*end_color, 1.0),
                ),
                border_widths: Edges::default(),
                border_color: transparent_black(),
                border_style: BorderStyle::default(),
            });
        }
    }
}

#[derive(Clone, Debug)]
pub struct GradientDomainTrack {
    pub colors: Vec<Hsla>,
}

impl GradientDomainTrack {
    pub fn new(colors: Vec<Hsla>) -> Self {
        Self { colors }
    }
}

impl DomainTrackRenderer for GradientDomainTrack {
    fn paint(&self, bounds: Bounds<Pixels>, orientation: Slider2Orientation, reversed: bool, window: &mut Window) {
        if self.colors.is_empty() {
            return;
        }

        let mut colors = self.colors.clone();
        if reversed {
            colors.reverse();
        }

        if colors.len() == 1 {
            window.paint_quad(PaintQuad {
                bounds,
                corner_radii: track_corner_radii(orientation, px(999.0)),
                background: colors[0].into(),
                border_widths: Edges::default(),
                border_color: transparent_black(),
                border_style: BorderStyle::default(),
            });
            return;
        }

        let is_vertical = orientation == Slider2Orientation::Vertical;
        let total_size = if is_vertical {
            bounds.size.height.as_f32()
        } else {
            bounds.size.width.as_f32()
        };
        let segment_count = colors.len().saturating_sub(1);
        let segment_size = total_size / segment_count as f32;
        let default_radius = if is_vertical {
            bounds.size.width.as_f32()
        } else {
            bounds.size.height.as_f32()
        } * 0.5;
        let corner_radius = px(default_radius);
        let angle = if is_vertical { 180.0 } else { 90.0 };

        for index in 0..segment_count {
            let start_color = colors[index];
            let end_color = colors[index + 1];
            let start_offset = segment_size * index as f32;
            let segment_bounds = if is_vertical {
                Bounds {
                    origin: point(bounds.origin.x, bounds.origin.y + px(start_offset)),
                    size: size(bounds.size.width, px(segment_size)),
                }
            } else {
                Bounds {
                    origin: point(bounds.origin.x + px(start_offset), bounds.origin.y),
                    size: size(px(segment_size), bounds.size.height),
                }
            };

            let mut corner_radii = track_corner_radii(orientation, px(0.0));
            apply_outer_edge_corner_radii(
                &mut corner_radii,
                orientation,
                corner_radius,
                index == 0,
                index == segment_count - 1,
            );

            window.paint_quad(PaintQuad {
                bounds: segment_bounds,
                corner_radii,
                background: linear_gradient(
                    angle,
                    linear_color_stop(start_color, 0.0),
                    linear_color_stop(end_color, 1.0),
                ),
                border_widths: Edges::default(),
                border_color: transparent_black(),
                border_style: BorderStyle::default(),
            });
        }
    }
}
