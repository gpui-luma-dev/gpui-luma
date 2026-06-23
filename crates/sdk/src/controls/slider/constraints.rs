use std::ops::RangeInclusive;

use crate::controls::value::{ControlRange, normalized_step};

const EPSILON: f32 = 0.0001;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RangeInterval {
    pub start: f32,
    pub end: f32,
}

impl RangeInterval {
    fn new(start: f32, end: f32) -> Option<Self> {
        if !start.is_finite() || !end.is_finite() {
            return None;
        }

        let (start, end) = if start <= end { (start, end) } else { (end, start) };
        Some(Self { start, end })
    }

    fn clamp_to(self, range: ControlRange) -> Option<Self> {
        let start = self.start.clamp(range.start, range.end);
        let end = self.end.clamp(range.start, range.end);
        if end + EPSILON < range.start || start - EPSILON > range.end {
            return None;
        }
        Some(Self { start, end })
    }

    fn into_range_inclusive(self) -> RangeInclusive<f32> {
        self.start..=self.end
    }
}

impl From<&RangeInclusive<f32>> for RangeInterval {
    fn from(value: &RangeInclusive<f32>) -> Self {
        let start = *value.start();
        let end = *value.end();
        Self { start: start.min(end), end: start.max(end) }
    }
}

pub trait SliderConstraintStrategy: Send + Sync {
    fn clamp_and_snap(
        &self,
        value: f32,
        allowed_intervals: &[RangeInclusive<f32>],
        range: ControlRange,
        step: Option<f32>,
    ) -> f32;

    fn step_value(
        &self,
        current: f32,
        step_delta: f32,
        allowed_intervals: &[RangeInclusive<f32>],
        range: ControlRange,
        step: Option<f32>,
    ) -> f32;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ContinuousConstraintStrategy;

#[derive(Clone, Copy, Debug, Default)]
pub struct IntervalConstraintStrategy;

pub fn normalize_intervals(intervals: &[RangeInclusive<f32>], range: ControlRange) -> Vec<RangeInclusive<f32>> {
    let mut normalized = intervals
        .iter()
        .filter_map(|interval| RangeInterval::new(*interval.start(), *interval.end()))
        .filter_map(|interval| interval.clamp_to(range))
        .collect::<Vec<_>>();

    normalized.sort_by(|left, right| left.start.total_cmp(&right.start));

    let mut merged: Vec<RangeInterval> = Vec::with_capacity(normalized.len());
    for interval in normalized {
        if let Some(last) = merged.last_mut()
            && interval.start <= last.end + EPSILON
        {
            last.end = last.end.max(interval.end);
            continue;
        }

        merged.push(interval);
    }

    merged.into_iter().map(RangeInterval::into_range_inclusive).collect()
}

pub fn clamp_and_snap_value(
    value: f32,
    allowed_intervals: &[RangeInclusive<f32>],
    range: ControlRange,
    step: f32,
) -> f32 {
    if allowed_intervals.is_empty() {
        return ContinuousConstraintStrategy.clamp_and_snap(value, allowed_intervals, range, Some(step));
    }

    IntervalConstraintStrategy.clamp_and_snap(value, allowed_intervals, range, Some(step))
}

pub fn step_allowed_value(
    current: f32,
    step_delta: f32,
    allowed_intervals: &[RangeInclusive<f32>],
    range: ControlRange,
    step: f32,
) -> f32 {
    if allowed_intervals.is_empty() {
        return ContinuousConstraintStrategy.step_value(current, step_delta, allowed_intervals, range, Some(step));
    }

    IntervalConstraintStrategy.step_value(current, step_delta, allowed_intervals, range, Some(step))
}

impl SliderConstraintStrategy for ContinuousConstraintStrategy {
    fn clamp_and_snap(
        &self,
        value: f32,
        _allowed_intervals: &[RangeInclusive<f32>],
        range: ControlRange,
        step: Option<f32>,
    ) -> f32 {
        match step {
            Some(step) => range.snap(value, step),
            None => range.clamp(value),
        }
    }

    fn step_value(
        &self,
        current: f32,
        step_delta: f32,
        _allowed_intervals: &[RangeInclusive<f32>],
        range: ControlRange,
        step: Option<f32>,
    ) -> f32 {
        let step = normalized_step(step.unwrap_or_else(|| step_delta.abs().max(1.0)));
        range.snap(current + step_delta, step)
    }
}

impl SliderConstraintStrategy for IntervalConstraintStrategy {
    fn clamp_and_snap(
        &self,
        value: f32,
        allowed_intervals: &[RangeInclusive<f32>],
        range: ControlRange,
        step: Option<f32>,
    ) -> f32 {
        let intervals = normalize_intervals(allowed_intervals, range);
        if intervals.is_empty() {
            return range.start;
        }

        let value = range.clamp(value);
        if let Some(interval) = interval_containing(&intervals, value) {
            return match step {
                Some(step) => nearest_candidate_in_interval(value, interval, range, step),
                None => value.clamp(*interval.start(), *interval.end()),
            };
        }

        nearest_boundary(value, &intervals)
    }

    fn step_value(
        &self,
        current: f32,
        step_delta: f32,
        allowed_intervals: &[RangeInclusive<f32>],
        range: ControlRange,
        step: Option<f32>,
    ) -> f32 {
        let intervals = normalize_intervals(allowed_intervals, range);
        if intervals.is_empty() {
            return range.start;
        }

        if step_delta.abs() <= EPSILON {
            return self.clamp_and_snap(current, allowed_intervals, range, step);
        }

        let base_step = normalized_step(step.unwrap_or_else(|| step_delta.abs().max(1.0)));
        let steps = ((step_delta.abs() / base_step).round() as usize).max(1);
        let direction = if step_delta.is_sign_negative() { -1.0 } else { 1.0 };
        let mut value = self.clamp_and_snap(current, allowed_intervals, range, Some(base_step));

        for _ in 0..steps {
            let next = step_once(value, direction, &intervals, range, base_step);
            if (next - value).abs() <= EPSILON {
                break;
            }
            value = next;
        }

        value
    }
}

fn interval_containing(intervals: &[RangeInclusive<f32>], value: f32) -> Option<&RangeInclusive<f32>> {
    intervals.iter().find(|interval| contains(interval, value))
}

fn contains(interval: &RangeInclusive<f32>, value: f32) -> bool {
    value >= *interval.start() - EPSILON && value <= *interval.end() + EPSILON
}

fn nearest_boundary(value: f32, intervals: &[RangeInclusive<f32>]) -> f32 {
    let mut best = *intervals[0].start();
    let mut best_distance = (value - best).abs();

    for interval in intervals {
        for candidate in [*interval.start(), *interval.end()] {
            let distance = (value - candidate).abs();
            if distance < best_distance - EPSILON || ((distance - best_distance).abs() <= EPSILON && candidate < best) {
                best = candidate;
                best_distance = distance;
            }
        }
    }

    best
}

fn nearest_candidate_in_interval(value: f32, interval: &RangeInclusive<f32>, range: ControlRange, step: f32) -> f32 {
    let start = *interval.start();
    let end = *interval.end();
    let mut candidates = vec![start, end];

    if let Some(candidate) = grid_at_or_before(value, range, step)
        && candidate >= start - EPSILON
        && candidate <= end + EPSILON
    {
        candidates.push(candidate.clamp(start, end));
    }

    if let Some(candidate) = grid_at_or_after(value, range, step)
        && candidate >= start - EPSILON
        && candidate <= end + EPSILON
    {
        candidates.push(candidate.clamp(start, end));
    }

    choose_nearest(value, &candidates)
}

fn choose_nearest(value: f32, candidates: &[f32]) -> f32 {
    let mut best = candidates[0];
    let mut best_distance = (value - best).abs();

    for &candidate in candidates.iter().skip(1) {
        let distance = (value - candidate).abs();
        if distance < best_distance - EPSILON || ((distance - best_distance).abs() <= EPSILON && candidate < best) {
            best = candidate;
            best_distance = distance;
        }
    }

    best
}

fn step_once(current: f32, direction: f32, intervals: &[RangeInclusive<f32>], range: ControlRange, step: f32) -> f32 {
    let index = intervals.iter().position(|interval| contains(interval, current)).unwrap_or_else(|| {
        intervals
            .iter()
            .position(|interval| current < *interval.start())
            .unwrap_or(intervals.len().saturating_sub(1))
    });

    if direction.is_sign_positive() {
        if let Some(next) = next_candidate_in_interval(current, &intervals[index], range, step) {
            return next;
        }

        return intervals.get(index + 1).map(|interval| *interval.start()).unwrap_or(*intervals[index].end());
    }

    if let Some(previous) = previous_candidate_in_interval(current, &intervals[index], range, step) {
        return previous;
    }

    if index > 0 {
        *intervals[index - 1].end()
    } else {
        *intervals[index].start()
    }
}

fn next_candidate_in_interval(
    current: f32,
    interval: &RangeInclusive<f32>,
    range: ControlRange,
    step: f32,
) -> Option<f32> {
    let start = *interval.start();
    let end = *interval.end();
    let mut candidates = Vec::new();

    if let Some(candidate) = grid_strictly_after(current, range, step)
        && candidate >= start - EPSILON
        && candidate <= end + EPSILON
    {
        candidates.push(candidate.clamp(start, end));
    }

    if end > current + EPSILON {
        candidates.push(end);
    }

    candidates.into_iter().min_by(|left, right| left.total_cmp(right))
}

fn previous_candidate_in_interval(
    current: f32,
    interval: &RangeInclusive<f32>,
    range: ControlRange,
    step: f32,
) -> Option<f32> {
    let start = *interval.start();
    let end = *interval.end();
    let mut candidates = Vec::new();

    if let Some(candidate) = grid_strictly_before(current, range, step)
        && candidate >= start - EPSILON
        && candidate <= end + EPSILON
    {
        candidates.push(candidate.clamp(start, end));
    }

    if start < current - EPSILON {
        candidates.push(start);
    }

    if end < current - EPSILON {
        candidates.push(end);
    }

    candidates.into_iter().max_by(|left, right| left.total_cmp(right))
}

fn grid_at_or_before(value: f32, range: ControlRange, step: f32) -> Option<f32> {
    let step = normalized_step(step);
    let offset = ((value - range.start) / step).floor();
    if !offset.is_finite() {
        return None;
    }
    Some((range.start + offset * step).clamp(range.start, range.end))
}

fn grid_at_or_after(value: f32, range: ControlRange, step: f32) -> Option<f32> {
    let step = normalized_step(step);
    let offset = ((value - range.start) / step).ceil();
    if !offset.is_finite() {
        return None;
    }
    Some((range.start + offset * step).clamp(range.start, range.end))
}

fn grid_strictly_after(value: f32, range: ControlRange, step: f32) -> Option<f32> {
    let candidate = grid_at_or_after(value + EPSILON, range, step)?;
    (candidate > value + EPSILON).then_some(candidate)
}

fn grid_strictly_before(value: f32, range: ControlRange, step: f32) -> Option<f32> {
    let candidate = grid_at_or_before(value - EPSILON, range, step)?;
    (candidate < value - EPSILON).then_some(candidate)
}

#[cfg(test)]
mod tests {
    use super::{IntervalConstraintStrategy, SliderConstraintStrategy, normalize_intervals};
    use crate::controls::value::ControlRange;

    #[test]
    fn normalize_intervals_sorts_clamps_and_merges() {
        let range = ControlRange::new(0.0, 100.0);
        let intervals = normalize_intervals(&[90.0..=120.0, 10.0..=20.0, 18.0..=40.0, -10.0..=0.0], range);

        assert_eq!(intervals, vec![0.0..=0.0, 10.0..=40.0, 90.0..=100.0]);
    }

    #[test]
    fn clamp_and_snap_snaps_gap_value_to_nearest_boundary() {
        let range = ControlRange::new(0.0, 360.0);
        let strategy = IntervalConstraintStrategy;
        let value = strategy.clamp_and_snap(150.0, &[0.0..=120.0, 180.0..=240.0, 300.0..=360.0], range, Some(10.0));

        assert_eq!(value, 120.0);
    }

    #[test]
    fn clamp_and_snap_respects_step_inside_interval() {
        let range = ControlRange::new(0.0, 360.0);
        let strategy = IntervalConstraintStrategy;
        let value = strategy.clamp_and_snap(187.0, &[0.0..=120.0, 180.0..=240.0, 300.0..=360.0], range, Some(10.0));

        assert_eq!(value, 190.0);
    }

    #[test]
    fn step_value_skips_forward_over_gap() {
        let range = ControlRange::new(0.0, 360.0);
        let strategy = IntervalConstraintStrategy;
        let value = strategy.step_value(120.0, 10.0, &[0.0..=120.0, 180.0..=240.0, 300.0..=360.0], range, Some(10.0));

        assert_eq!(value, 180.0);
    }

    #[test]
    fn step_value_skips_backward_over_gap() {
        let range = ControlRange::new(0.0, 360.0);
        let strategy = IntervalConstraintStrategy;
        let value = strategy.step_value(180.0, -10.0, &[0.0..=120.0, 180.0..=240.0, 300.0..=360.0], range, Some(10.0));

        assert_eq!(value, 120.0);
    }

    #[test]
    fn step_value_repeats_for_large_delta() {
        let range = ControlRange::new(0.0, 360.0);
        let strategy = IntervalConstraintStrategy;
        let value = strategy.step_value(100.0, 30.0, &[0.0..=120.0, 180.0..=240.0, 300.0..=360.0], range, Some(10.0));

        assert_eq!(value, 180.0);
    }

    #[test]
    fn empty_intervals_are_left_for_continuous_strategy() {
        let intervals = normalize_intervals(&[], ControlRange::new(0.0, 100.0));
        assert!(intervals.is_empty());
    }
}
