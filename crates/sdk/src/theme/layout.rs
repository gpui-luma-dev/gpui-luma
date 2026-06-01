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

pub(crate) fn label_baseline_shift(size: ControlSize) -> f32 {
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
