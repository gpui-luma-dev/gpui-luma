use gpui::{Pixels, Point, point, px};

pub(super) struct CubicSegment {
    pub(super) end: Point<Pixels>,
    pub(super) control_a: Point<Pixels>,
    pub(super) control_b: Point<Pixels>,
}

/// PCHIP weighted harmonic derivatives and one-sided endpoint derivatives.
/// See https://docs.scipy.org/doc/scipy/reference/generated/scipy.interpolate.PchipInterpolator.html
/// Work in f64 so small pixel intervals and large slope ratios remain finite.
pub(super) fn monotone(points: &[Point<Pixels>]) -> Option<Vec<CubicSegment>> {
    if points.len() < 3
        || points.iter().any(|p| !f32::from(p.x).is_finite() || !f32::from(p.y).is_finite())
        || points.windows(2).any(|pair| pair[1].x <= pair[0].x)
    {
        return None;
    }
    let x = |i: usize| f64::from(f32::from(points[i].x));
    let y = |i: usize| f64::from(f32::from(points[i].y));
    let intervals: Vec<_> = (0..points.len() - 1).map(|i| x(i + 1) - x(i)).collect();
    let secants: Vec<_> = intervals.iter().enumerate().map(|(i, h)| (y(i + 1) - y(i)) / h).collect();
    let mut derivatives = vec![0.0; points.len()];
    derivatives[0] = endpoint(intervals[0], intervals[1], secants[0], secants[1]);
    for i in 1..points.len() - 1 {
        let left = secants[i - 1];
        let right = secants[i];
        if left != 0.0 && right != 0.0 && left.signum() == right.signum() {
            let w1 = 2.0 * intervals[i] + intervals[i - 1];
            let w2 = intervals[i] + 2.0 * intervals[i - 1];
            derivatives[i] = (w1 + w2) / (w1 / left + w2 / right);
        }
    }
    let last = secants.len() - 1;
    derivatives[points.len() - 1] = endpoint(intervals[last], intervals[last - 1], secants[last], secants[last - 1]);
    Some(
        intervals
            .iter()
            .enumerate()
            .map(|(i, h)| {
                let third = h / 3.0;
                let min = y(i).min(y(i + 1));
                let max = y(i).max(y(i + 1));
                CubicSegment {
                    end: points[i + 1],
                    control_a: point(
                        px((x(i) + third) as f32),
                        px((y(i) + third * derivatives[i]).clamp(min, max) as f32),
                    ),
                    control_b: point(
                        px((x(i + 1) - third) as f32),
                        px((y(i + 1) - third * derivatives[i + 1]).clamp(min, max) as f32),
                    ),
                }
            })
            .collect(),
    )
}

fn endpoint(h0: f64, h1: f64, d0: f64, d1: f64) -> f64 {
    let derivative = ((2.0 * h0 + h1) * d0 - h0 * d1) / (h0 + h1);
    if derivative.signum() != d0.signum() {
        0.0
    } else if d0.signum() != d1.signum() && derivative.abs() > 3.0 * d0.abs() {
        3.0 * d0
    } else {
        derivative
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sample;

    fn points(values: &[(f32, f32)]) -> Vec<Point<Pixels>> {
        values.iter().map(|(x, y)| point(px(*x), px(*y))).collect()
    }

    fn evaluate(start: Point<Pixels>, segment: &CubicSegment, t: f32) -> Point<Pixels> {
        let s = 1.0 - t;
        start * (s * s * s)
            + segment.control_a * (3.0 * s * s * t)
            + segment.control_b * (3.0 * s * t * t)
            + segment.end * (t * t * t)
    }

    #[test]
    fn irregular_intervals_preserve_knots_plateaus_and_neighbor_bounds() {
        let points = points(&[(0.0, 0.0), (0.2, 3.0), (1.0, 1.0), (4.0, 5.0), (4.1, 5.0), (8.0, -2.0)]);
        let segments = monotone(&points).expect("strictly increasing x");
        assert_eq!(segments.len(), points.len() - 1);
        for (index, segment) in segments.iter().enumerate() {
            assert_eq!(evaluate(points[index], segment, 0.0), points[index]);
            assert_eq!(evaluate(points[index], segment, 1.0), points[index + 1]);
            let min = points[index].y.min(segment.end.y);
            let max = points[index].y.max(segment.end.y);
            for step in 0..=100 {
                let value = evaluate(points[index], segment, step as f32 / 100.0);
                assert!(value.y >= min - px(1e-5) && value.y <= max + px(1e-5));
                assert!(value.x >= points[index].x - px(1e-5) && value.x <= segment.end.x + px(1e-5));
            }
        }
        assert_eq!(segments[3].control_a.y, px(5.0));
        assert_eq!(segments[3].control_b.y, px(5.0));
    }

    #[test]
    fn derivatives_are_continuous_at_internal_knots() {
        let points = points(&[(0.0, 0.0), (1.0, 1.0), (2.0, 3.0), (5.0, 4.0)]);
        let segments = monotone(&points).expect("increasing x");
        for index in 0..segments.len() - 1 {
            let knot = segments[index].end;
            let left =
                f32::from(knot.y - segments[index].control_b.y) / f32::from(knot.x - segments[index].control_b.x);
            let right = f32::from(segments[index + 1].control_a.y - knot.y)
                / f32::from(segments[index + 1].control_a.x - knot.x);
            assert!((left - right).abs() < 1e-5);
        }
    }

    #[test]
    fn unsuitable_runs_request_straight_fallback() {
        for values in [
            vec![],
            vec![(0.0, 1.0)],
            vec![(0.0, 1.0), (1.0, 2.0)],
            vec![(0.0, 1.0), (0.0, 2.0), (1.0, 3.0)],
            vec![(0.0, 1.0), (2.0, 2.0), (1.0, 3.0)],
            vec![(0.0, 1.0), (1.0, f32::NAN), (2.0, 3.0)],
        ] {
            assert!(monotone(&points(&values)).is_none());
        }
    }

    #[test]
    fn viewport_context_preserves_the_full_curve_at_visible_edges() {
        let points =
            points(&[(0.0, 0.0), (1.0, 3.0), (2.0, 2.0), (3.0, 4.0), (4.0, 6.0), (5.0, 5.0), (6.0, 8.0), (7.0, 4.0)]);
        let samples: Vec<_> =
            points.iter().map(|p| Sample::new(f64::from(f32::from(p.x)), f64::from(f32::from(p.y)))).collect();
        let mut visible = Vec::new();
        super::super::viewport::select(&samples, 2.5, 4.5, 1, |run| {
            visible.extend(run.iter().map(|p| point(px(p.x as f32), px(p.y as f32))));
        });
        let full = monotone(&points).expect("curve");
        let cropped = monotone(&visible).expect("curve");
        assert_eq!(visible.first(), Some(&points[1]));
        assert_eq!(visible.last(), Some(&points[6]));
        for index in 2..=4 {
            assert_eq!(full[index].control_a, cropped[index - 1].control_a);
            assert_eq!(full[index].control_b, cropped[index - 1].control_b);
            assert_eq!(full[index].end, cropped[index - 1].end);
        }
    }
}
