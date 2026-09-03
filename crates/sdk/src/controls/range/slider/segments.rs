use std::ops::RangeInclusive;

use crate::infra::value::ControlRange;

use super::model::{TrackPresentation, TrackSegment, TrackSegmentKind};

pub fn build_track_segments(
    presentation: TrackPresentation,
    thumb_position: f32,
    allowed_intervals: &[RangeInclusive<f32>],
    range: ControlRange,
) -> Vec<TrackSegment> {
    match presentation {
        TrackPresentation::Fill => build_fill_segments(thumb_position, allowed_intervals, range),
        TrackPresentation::Domain => build_domain_segments(allowed_intervals, range),
    }
}

fn build_fill_segments(
    thumb_position: f32,
    allowed_intervals: &[RangeInclusive<f32>],
    range: ControlRange,
) -> Vec<TrackSegment> {
    if allowed_intervals.is_empty() {
        let thumb = thumb_position.clamp(0.0, 1.0);
        return vec![
            TrackSegment { start: 0.0, end: thumb, kind: TrackSegmentKind::Active },
            TrackSegment { start: thumb, end: 1.0, kind: TrackSegmentKind::Inactive },
        ];
    }

    build_interval_segments(allowed_intervals, range, TrackSegmentKind::Active, TrackSegmentKind::Blocked)
}

fn build_domain_segments(allowed_intervals: &[RangeInclusive<f32>], range: ControlRange) -> Vec<TrackSegment> {
    if allowed_intervals.is_empty() {
        return vec![TrackSegment { start: 0.0, end: 1.0, kind: TrackSegmentKind::Domain }];
    }

    build_interval_segments(allowed_intervals, range, TrackSegmentKind::Domain, TrackSegmentKind::Blocked)
}

fn build_interval_segments(
    allowed_intervals: &[RangeInclusive<f32>],
    range: ControlRange,
    allowed_kind: TrackSegmentKind,
    gap_kind: TrackSegmentKind,
) -> Vec<TrackSegment> {
    let normalized = super::constraints::normalize_intervals(allowed_intervals, range);
    if normalized.is_empty() {
        return vec![TrackSegment { start: 0.0, end: 1.0, kind: allowed_kind }];
    }

    let mut segments = Vec::with_capacity(normalized.len() * 2 + 1);
    let mut cursor = range.start;

    for interval in normalized {
        let start = *interval.start();
        let end = *interval.end();

        if start > cursor {
            segments.push(TrackSegment {
                start: range.percentage(cursor),
                end: range.percentage(start),
                kind: gap_kind,
            });
        }

        segments.push(TrackSegment { start: range.percentage(start), end: range.percentage(end), kind: allowed_kind });
        cursor = end;
    }

    if cursor < range.end {
        segments.push(TrackSegment {
            start: range.percentage(cursor),
            end: range.percentage(range.end),
            kind: gap_kind,
        });
    }

    if segments.is_empty() {
        segments.push(TrackSegment { start: 0.0, end: 1.0, kind: allowed_kind });
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::value::ControlRange;

    #[test]
    fn fill_segments_split_at_thumb() {
        let segments = build_fill_segments(0.25, &[], ControlRange::new(0.0, 100.0));
        assert_eq!(
            segments,
            vec![
                TrackSegment { start: 0.0, end: 0.25, kind: TrackSegmentKind::Active },
                TrackSegment { start: 0.25, end: 1.0, kind: TrackSegmentKind::Inactive },
            ]
        );
    }

    #[test]
    fn fill_segments_with_gaps() {
        let range = ControlRange::new(0.0, 360.0);
        let segments = build_fill_segments(0.5, &[0.0..=120.0, 180.0..=240.0, 300.0..=360.0], range);
        assert_eq!(segments.len(), 5);
        assert_eq!(segments[0].kind, TrackSegmentKind::Active);
        assert_eq!(segments[1].kind, TrackSegmentKind::Blocked);
    }

    #[test]
    fn domain_segments_cover_full_track() {
        let segments = build_domain_segments(&[], ControlRange::default());
        assert_eq!(segments, vec![TrackSegment { start: 0.0, end: 1.0, kind: TrackSegmentKind::Domain }]);
    }

    #[test]
    fn interval_segments_partition_track_without_gaps() {
        let range = ControlRange::new(0.0, 360.0);
        let segments = build_interval_segments(
            &[0.0..=120.0, 180.0..=240.0, 300.0..=360.0],
            range,
            TrackSegmentKind::Active,
            TrackSegmentKind::Blocked,
        );

        assert_eq!(segments.first().map(|segment| segment.start), Some(0.0));
        assert_eq!(segments.last().map(|segment| segment.end), Some(1.0));

        let covered: f32 = segments.iter().map(|segment| segment.end - segment.start).sum();
        assert!((covered - 1.0).abs() <= 0.0001);
    }
}
