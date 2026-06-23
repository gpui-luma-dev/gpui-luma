use gpui::{Corners, Pixels};

use super::model::{Slider2Orientation, TrackSegment};

const TRACK_EDGE_EPSILON: f32 = 0.0001;

pub fn display_position(position: f32, reversed: bool) -> f32 {
    let position = position.clamp(0.0, 1.0);
    if reversed { 1.0 - position } else { position }
}

pub fn position_from_pointer(raw: f32, reversed: bool) -> f32 {
    display_position(raw, reversed)
}

pub fn oriented_step_delta(delta: f32, reversed: bool) -> f32 {
    if reversed { -delta } else { delta }
}

pub fn segment_display_span(segment: &TrackSegment, reversed: bool) -> (f32, f32) {
    let start = segment.start.clamp(0.0, 1.0);
    let end = segment.end.clamp(0.0, 1.0);
    let span = (end - start).clamp(0.0, 1.0);

    if reversed { (1.0 - end, span) } else { (start, span) }
}

pub fn segment_corner_radii(
    display_start: f32,
    display_span: f32,
    orientation: Slider2Orientation,
    radius: Pixels,
) -> Corners<Pixels> {
    let mut corner_radii = Corners::default();
    apply_outer_edge_corner_radii(
        &mut corner_radii,
        orientation,
        radius,
        touches_track_start(display_start),
        touches_track_end(display_start + display_span),
    );
    corner_radii
}

pub(crate) fn apply_outer_edge_corner_radii(
    corner_radii: &mut Corners<Pixels>,
    orientation: Slider2Orientation,
    radius: Pixels,
    round_start: bool,
    round_end: bool,
) {
    if round_start {
        match orientation {
            Slider2Orientation::Horizontal => {
                corner_radii.top_left = radius;
                corner_radii.bottom_left = radius;
            }
            Slider2Orientation::Vertical => {
                corner_radii.bottom_left = radius;
                corner_radii.bottom_right = radius;
            }
        }
    }

    if round_end {
        match orientation {
            Slider2Orientation::Horizontal => {
                corner_radii.top_right = radius;
                corner_radii.bottom_right = radius;
            }
            Slider2Orientation::Vertical => {
                corner_radii.top_left = radius;
                corner_radii.top_right = radius;
            }
        }
    }
}

pub fn touches_track_start(value: f32) -> bool {
    value <= TRACK_EDGE_EPSILON
}

pub fn touches_track_end(value: f32) -> bool {
    value >= 1.0 - TRACK_EDGE_EPSILON
}

#[cfg(test)]
mod tests {
    use gpui::px;

    use super::*;
    use crate::controls::slider::model::{TrackSegment, TrackSegmentKind};

    #[test]
    fn display_position_mirrors_when_reversed() {
        assert_eq!(display_position(0.25, false), 0.25);
        assert_eq!(display_position(0.25, true), 0.75);
    }

    #[test]
    fn oriented_step_delta_negates_when_reversed() {
        assert_eq!(oriented_step_delta(10.0, false), 10.0);
        assert_eq!(oriented_step_delta(10.0, true), -10.0);
        assert_eq!(oriented_step_delta(-10.0, true), 10.0);
    }

    #[test]
    fn pointer_mapping_round_trips_through_reversed_display() {
        let raw = 0.2;
        assert_eq!(position_from_pointer(raw, true), 0.8);
        assert!((display_position(position_from_pointer(raw, true), true) - raw).abs() <= f32::EPSILON);
    }

    #[test]
    fn segment_display_span_mirrors_horizontal_segments() {
        let segment = TrackSegment { start: 0.0, end: 0.25, kind: TrackSegmentKind::Active };
        assert_eq!(segment_display_span(&segment, false), (0.0, 0.25));
        assert_eq!(segment_display_span(&segment, true), (0.75, 0.25));
    }

    #[test]
    fn middle_segment_gets_no_corner_rounding() {
        let radii = segment_corner_radii(0.25, 0.5, Slider2Orientation::Horizontal, px(4.0));
        assert_eq!(radii.top_left, px(0.0));
        assert_eq!(radii.top_right, px(0.0));
    }

    #[test]
    fn outer_segments_round_only_outer_edges() {
        let start = segment_corner_radii(0.0, 0.25, Slider2Orientation::Horizontal, px(4.0));
        assert_eq!(start.top_left, px(4.0));
        assert_eq!(start.top_right, px(0.0));

        let end = segment_corner_radii(0.75, 0.25, Slider2Orientation::Horizontal, px(4.0));
        assert_eq!(end.top_left, px(0.0));
        assert_eq!(end.top_right, px(4.0));
    }
}
