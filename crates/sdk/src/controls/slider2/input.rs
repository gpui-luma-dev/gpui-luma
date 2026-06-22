use std::f32::consts::PI;

use gpui::{Bounds, Pixels, Point, px};

use super::layout::position_from_pointer;
use super::model::Slider2Orientation;
use crate::controls::value::{ControlRange, normalized_step};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Slider2InputStrategy {
    #[default]
    Horizontal,
    Vertical,
    Angular {
        min_angle: f32,
        max_angle: f32,
    },
}

impl Slider2InputStrategy {
    pub fn orientation(self) -> Slider2Orientation {
        match self {
            Self::Horizontal | Self::Angular { .. } => Slider2Orientation::Horizontal,
            Self::Vertical => Slider2Orientation::Vertical,
        }
    }

    pub fn is_angular(self) -> bool {
        matches!(self, Self::Angular { .. })
    }
}

impl From<Slider2Orientation> for Slider2InputStrategy {
    fn from(value: Slider2Orientation) -> Self {
        match value {
            Slider2Orientation::Horizontal => Self::Horizontal,
            Slider2Orientation::Vertical => Self::Vertical,
        }
    }
}

pub fn percentage_from_position(
    strategy: Slider2InputStrategy,
    reversed: bool,
    bounds: Bounds<Pixels>,
    position: Point<Pixels>,
) -> Option<f32> {
    let raw = match strategy {
        Slider2InputStrategy::Horizontal => {
            if bounds.size.width <= px(0.0) {
                return None;
            }

            ((position.x - bounds.left()) / bounds.size.width).clamp(0.0, 1.0)
        }
        Slider2InputStrategy::Vertical => {
            if bounds.size.height <= px(0.0) {
                return None;
            }

            (1.0 - ((position.y - bounds.top()) / bounds.size.height)).clamp(0.0, 1.0)
        }
        Slider2InputStrategy::Angular { min_angle, max_angle } => {
            let angle = angle_from_position(bounds, position)?;
            let span = max_angle - min_angle;
            if span.abs() <= f32::EPSILON {
                return None;
            }

            let normalized_angle = normalize_angle_for_range(angle, min_angle, max_angle);
            ((normalized_angle.clamp(min_angle, max_angle) - min_angle) / span).clamp(0.0, 1.0)
        }
    };

    Some(position_from_pointer(raw, reversed))
}

pub fn angle_for_percentage(min_angle: f32, max_angle: f32, percentage: f32) -> f32 {
    min_angle + (max_angle - min_angle) * percentage.clamp(0.0, 1.0)
}

pub fn angle_from_position(bounds: Bounds<Pixels>, position: Point<Pixels>) -> Option<f32> {
    let center = bounds.center();
    let dy = (position.y - center.y).as_f32();
    let dx = (position.x - center.x).as_f32();

    if dx.abs() <= f32::EPSILON && dy.abs() <= f32::EPSILON {
        return None;
    }

    Some(dy.atan2(dx))
}

pub fn unwrap_angle_near(reference: f32, angle: f32) -> f32 {
    let mut angle = angle;

    while angle - reference > PI {
        angle -= 2.0 * PI;
    }
    while angle - reference < -PI {
        angle += 2.0 * PI;
    }

    angle
}

pub fn normalize_angle_for_range(angle: f32, min_angle: f32, max_angle: f32) -> f32 {
    let mut normalized_angle = angle;

    while normalized_angle > max_angle {
        normalized_angle -= 2.0 * PI;
    }
    while normalized_angle < min_angle {
        normalized_angle += 2.0 * PI;
    }

    normalized_angle
}

pub fn angular_drag_value(
    pointer_angle: f32,
    offset: f32,
    min_angle: f32,
    max_angle: f32,
    range: ControlRange,
    wrapping: bool,
    step: f32,
) -> Option<f32> {
    let span = max_angle - min_angle;
    if span.abs() <= f32::EPSILON {
        return None;
    }

    let target_angle = pointer_angle - offset;
    if wrapping {
        let raw_value = range.start + ((target_angle - min_angle) / span) * range.span();
        Some(wrap_and_snap(raw_value, range, step))
    } else {
        let target_angle = target_angle.clamp(min_angle, max_angle);
        let percentage = ((target_angle - min_angle) / span).clamp(0.0, 1.0);
        Some(range.value_at(percentage))
    }
}

pub fn wrap_value(value: f32, range: ControlRange) -> f32 {
    let span = range.span();
    if span <= f32::EPSILON {
        return range.start;
    }

    let mut wrapped = (value - range.start) % span;
    if wrapped < 0.0 {
        wrapped += span;
    }

    range.start + wrapped
}

pub fn wrap_and_snap(value: f32, range: ControlRange, step: f32) -> f32 {
    range.snap(wrap_value(value, range), normalized_step(step))
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use gpui::{Bounds, point, px, size};

    use super::*;

    #[test]
    fn angular_percentage_maps_clamped_arc() {
        let strategy = Slider2InputStrategy::Angular { min_angle: -1.25 * PI, max_angle: 0.25 * PI };
        let bounds = Bounds { origin: point(px(0.0), px(0.0)), size: size(px(200.0), px(200.0)) };
        let percentage = percentage_from_position(strategy, false, bounds, point(px(100.0), px(0.0)));

        assert_eq!(percentage, Some(0.5));
    }

    #[test]
    fn unwrap_angle_keeps_pointer_continuous_across_atan_seam() {
        let reference = -3.9;
        let raw_angle = 2.9;
        let unwrapped = unwrap_angle_near(reference, raw_angle);

        assert!(unwrapped < -3.0, "expected seam-crossing angle near the reference, got {unwrapped}");
    }

    #[test]
    fn wrap_value_loops_within_range() {
        let range = ControlRange::new(0.0, 360.0);
        assert_eq!(wrap_value(370.0, range), 10.0);
        assert_eq!(wrap_value(-10.0, range), 350.0);
    }

    #[test]
    fn angular_drag_value_wraps_when_enabled() {
        let range = ControlRange::new(0.0, 360.0);
        let min_angle = 0.0;
        let max_angle = 2.0 * PI;
        let span = max_angle - min_angle;
        let offset = 0.0;
        let pointer_past_end = min_angle + span * 1.05;

        let value = angular_drag_value(pointer_past_end, offset, min_angle, max_angle, range, true, 1.0);

        assert_eq!(value, Some(18.0));
    }

    #[test]
    fn angular_drag_value_clamps_when_not_wrapping() {
        let range = ControlRange::new(0.0, 360.0);
        let min_angle = -1.25 * PI;
        let max_angle = 0.25 * PI;
        let offset = 0.0;
        let pointer_past_end = max_angle + 0.5;

        let value = angular_drag_value(pointer_past_end, offset, min_angle, max_angle, range, false, 1.0);

        assert_eq!(value, Some(360.0));
    }
}
