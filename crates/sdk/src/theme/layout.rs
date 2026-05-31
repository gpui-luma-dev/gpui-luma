use crate::theme::{ControlSize, MetricTokens};

/// Rounds a logical length to the nearest physical pixel at the given display scale.
pub fn snap_to_pixel(value: f32, scale_factor: f32) -> f32 {
    (value * scale_factor).round() / scale_factor
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StandardBoxScale {
    pub height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub radius: f32,
    pub track_width: f32,
    pub track_height: f32,
    pub track_padding: f32,
    pub thumb_size: f32,
    pub track_radius: f32,
    pub label_baseline_shift: f32,
}

impl StandardBoxScale {
    pub fn compute(size: ControlSize, metrics: &MetricTokens, scale_factor: f32) -> Self {
        let control_height = metrics.control_height(size);

        Self {
            height: snap_to_pixel(control_height, scale_factor),
            padding_x: snap_to_pixel(metrics.padding_x(size), scale_factor),
            padding_y: snap_to_pixel(metrics.padding_y(size), scale_factor),
            gap: snap_to_pixel(metrics.gap(size), scale_factor),
            radius: metrics.radius(size),
            track_width: snap_to_pixel(control_height * (42.0 / 36.0), scale_factor),
            track_height: snap_to_pixel(control_height * (22.0 / 36.0), scale_factor),
            track_padding: snap_to_pixel((control_height * (2.0 / 36.0)).max(1.0), scale_factor),
            thumb_size: snap_to_pixel(control_height * 0.5, scale_factor),
            track_radius: metrics.radius.pill,
            label_baseline_shift: label_baseline_shift(size),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphIndicatorScale {
    pub control_radius: f32,
    pub control_padding_x: f32,
    pub control_padding_y: f32,
    pub indicator_size: f32,
    pub indicator_radius: f32,
    pub height: f32,
    pub gap: f32,
    pub label_baseline_shift: f32,
    pub glyph_size: f32,
    pub dot_size: f32,
    pub icon_inset: f32,
    pub icon_stroke_width: f32,
}

impl GlyphIndicatorScale {
    pub fn compute(size: ControlSize, metrics: &MetricTokens, scale_factor: f32) -> Self {
        let control_height = metrics.control_height(size);
        let (indicator_ratio, indicator_radius, icon_stroke_width, icon_inset) = match size {
            ControlSize::Sm => (0.45, 2.0, 1.0, 2.0),
            ControlSize::Md => (0.50, 4.0, 1.5, 3.0),
            ControlSize::Lg => (0.55, 6.0, 2.0, 4.0),
        };

        let indicator_size = snap_to_pixel(control_height * indicator_ratio, scale_factor);
        let icon_inset = snap_to_pixel(icon_inset, scale_factor);

        Self {
            control_radius: metrics.radius(size),
            control_padding_x: 0.0,
            control_padding_y: 0.0,
            indicator_size,
            indicator_radius: snap_to_pixel(indicator_radius, scale_factor),
            height: snap_to_pixel(control_height, scale_factor),
            gap: snap_to_pixel(metrics.gap(size), scale_factor),
            label_baseline_shift: label_baseline_shift(size),
            glyph_size: snap_to_pixel((indicator_size - icon_inset * 2.0).max(0.0), scale_factor),
            dot_size: snap_to_pixel(control_height * 0.24, scale_factor),
            icon_inset,
            icon_stroke_width: snap_to_pixel(icon_stroke_width, scale_factor),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ListRowScale {
    pub min_height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub radius: f32,
    pub gap: f32,
    pub label_baseline_shift: f32,
}

impl ListRowScale {
    pub fn compute(size: ControlSize, metrics: &MetricTokens, scale_factor: f32) -> Self {
        Self {
            min_height: snap_to_pixel(metrics.control_height(size), scale_factor),
            padding_x: snap_to_pixel(metrics.padding_x(size), scale_factor),
            padding_y: snap_to_pixel(metrics.padding_y(size), scale_factor),
            radius: metrics.radius(size),
            gap: snap_to_pixel(metrics.gap(size), scale_factor),
            label_baseline_shift: label_baseline_shift(size),
        }
    }
}

fn label_baseline_shift(size: ControlSize) -> f32 {
    match size {
        ControlSize::Sm => 0.5,
        ControlSize::Md => 1.0,
        ControlSize::Lg => 1.5,
    }
}

#[cfg(test)]
mod tests {
    use super::snap_to_pixel;

    #[test]
    fn snap_to_pixel_snaps_at_1x() {
        assert_eq!(snap_to_pixel(10.4, 1.0), 10.0);
        assert_eq!(snap_to_pixel(10.6, 1.0), 11.0);
    }

    #[test]
    fn snap_to_pixel_snaps_at_1_5x() {
        assert_eq!(snap_to_pixel(10.2, 1.5), 10.0);
        assert!((snap_to_pixel(10.4, 1.5) - (16.0 / 1.5)).abs() < 0.000_001);
    }

    #[test]
    fn snap_to_pixel_snaps_at_2x() {
        assert_eq!(snap_to_pixel(10.2, 2.0), 10.0);
        assert_eq!(snap_to_pixel(10.3, 2.0), 10.5);
    }
}
