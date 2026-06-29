use std::ops::RangeInclusive;

const TIME_STEP_SECONDS: [f32; 11] = [15.0, 30.0, 60.0, 120.0, 300.0, 600.0, 900.0, 1800.0, 3600.0, 7200.0, 10800.0];
const DISTANCE_STEP_MILES: [f32; 6] = [0.5, 1.0, 2.0, 5.0, 10.0, 20.0];
const DISTANCE_STEP_KM: [f32; 6] = [0.5, 1.0, 2.0, 5.0, 10.0, 25.0];

use crate::graph::units::SpeedUnit;

#[derive(Clone, Debug, PartialEq)]
pub struct AxisScale {
    pub range: RangeInclusive<f32>,
    pub ticks: Vec<f32>,
    pub step: f32,
}

/// Picks a "nice" step size (1, 2, 5, 10 × 10^n) for the given rough interval.
pub fn nice_step(rough_step: f32) -> f32 {
    if !rough_step.is_finite() || rough_step <= 0.0 {
        return 1.0;
    }

    let magnitude = 10_f32.powf(rough_step.log10().floor());
    let normalized = rough_step / magnitude;
    let nice = if normalized <= 1.0 {
        1.0
    } else if normalized <= 2.0 {
        2.0
    } else if normalized <= 5.0 {
        5.0
    } else {
        10.0
    };
    nice * magnitude
}

fn nice_time_step(rough_step: f32) -> f32 {
    TIME_STEP_SECONDS
        .iter()
        .copied()
        .find(|step| *step >= rough_step)
        .unwrap_or_else(|| nice_step(rough_step))
}

fn nice_distance_step(rough_step: f32, speed_unit: SpeedUnit) -> f32 {
    let steps = match speed_unit {
        SpeedUnit::Mph => &DISTANCE_STEP_MILES,
        SpeedUnit::Kmh => &DISTANCE_STEP_KM,
    };
    steps.iter().copied().find(|step| *step >= rough_step).unwrap_or_else(|| nice_step(rough_step))
}

/// Generates evenly spaced ticks with nice step and range that covers `[data_min, data_max]`.
pub fn nice_value_ticks(
    data_min: f32,
    data_max: f32,
    include_zero: bool,
    target_count: usize,
    step_override: Option<f32>,
) -> AxisScale {
    let target = target_count.max(2);
    let mut lo = data_min.min(data_max);
    let mut hi = data_min.max(data_max);

    if (hi - lo).abs() < f32::EPSILON {
        let padding = if lo.abs() < f32::EPSILON { 1.0 } else { lo.abs() * 0.1 };
        lo -= padding;
        hi += padding;
    }

    if include_zero {
        lo = lo.min(0.0);
    }

    let span = (hi - lo).max(f32::EPSILON);
    let step = step_override
        .filter(|step| *step > 0.0 && step.is_finite())
        .unwrap_or_else(|| nice_step(span / (target - 1) as f32));
    let tick_min = if include_zero { 0.0 } else { (lo / step).floor() * step };
    let tick_max = (hi / step).ceil() * step;
    let ticks = generate_ticks(tick_min, tick_max, step);

    AxisScale { range: tick_min..=tick_max, ticks, step }
}

/// Time axis with calendar-friendly steps, sized to `plot_width_px`.
pub fn nice_time_ticks(duration_seconds: f32, plot_width_px: f32, min_label_px: f32) -> AxisScale {
    let duration = duration_seconds.max(f32::EPSILON);
    let max_ticks = ((plot_width_px / min_label_px.max(1.0)).floor() as usize).clamp(3, 8);
    let rough_step = duration / (max_ticks - 1).max(1) as f32;
    let step = nice_time_step(rough_step);

    let mut ticks = generate_ticks(0.0, duration, step);
    while ticks.last().is_some_and(|tick| *tick > duration + step * 0.01) {
        ticks.pop();
    }

    if ticks.is_empty() {
        ticks.push(0.0);
    }

    let last = *ticks.last().unwrap_or(&0.0);
    if duration - last > step * 0.15 {
        ticks.push(sanitize_tick(duration));
    }

    AxisScale { range: 0.0..=duration, ticks, step }
}

/// Distance axis with ride-friendly steps. Domain extent matches `distance_display` exactly,
/// like the time axis uses exact duration rather than a padded nice range.
pub fn nice_distance_ticks(
    distance_display: f32,
    plot_width_px: f32,
    min_label_px: f32,
    speed_unit: SpeedUnit,
) -> AxisScale {
    let distance = distance_display.max(f32::EPSILON);
    let max_ticks = ((plot_width_px / min_label_px.max(1.0)).floor() as usize).clamp(3, 8);
    let rough_step = distance / (max_ticks - 1).max(1) as f32;
    let step = nice_distance_step(rough_step, speed_unit);

    let mut ticks = generate_ticks(0.0, distance, step);
    while ticks.last().is_some_and(|tick| *tick > distance + step * 0.01) {
        ticks.pop();
    }

    if ticks.is_empty() {
        ticks.push(0.0);
    }

    let last = *ticks.last().unwrap_or(&0.0);
    if distance - last > step * 0.15 {
        ticks.push(sanitize_tick(distance));
    }

    AxisScale { range: 0.0..=distance, ticks, step }
}

pub fn fixed_value_ticks(range: std::ops::RangeInclusive<f32>, step: f32) -> AxisScale {
    let min = *range.start();
    let max = *range.end();
    let ticks = generate_ticks(min, max, step);
    AxisScale { range: min..=max, ticks, step }
}

pub fn cull_ticks_by_label_width(
    ticks: &[f32],
    label_widths: &[f32],
    x_min: f32,
    x_max: f32,
    plot_width_px: f32,
    min_gap_px: f32,
) -> Vec<f32> {
    if ticks.len() <= 2 {
        return ticks.to_vec();
    }

    let span = (x_max - x_min).max(f32::EPSILON);
    let plot_width = plot_width_px.max(f32::EPSILON);
    let mut kept = vec![ticks[0]];
    let mut kept_widths = vec![label_widths.first().copied().unwrap_or(0.0)];

    for index in 1..ticks.len() - 1 {
        let x = (ticks[index] - x_min) / span * plot_width;
        let half = label_widths.get(index).copied().unwrap_or(0.0) / 2.0;
        let prev_x = (*kept.last().unwrap_or(&x_min) - x_min) / span * plot_width;
        let prev_half = *kept_widths.last().unwrap_or(&0.0) / 2.0;

        if x - half >= prev_x + prev_half + min_gap_px {
            kept.push(ticks[index]);
            kept_widths.push(label_widths.get(index).copied().unwrap_or(0.0));
        }
    }

    kept.push(ticks[ticks.len() - 1]);
    kept
}

fn generate_ticks(min: f32, max: f32, step: f32) -> Vec<f32> {
    if step <= 0.0 || !step.is_finite() {
        return vec![min, max];
    }

    let mut ticks = Vec::new();
    let mut value = min;
    let limit = max + step * 0.001;
    while value <= limit {
        ticks.push(sanitize_tick(value));
        value += step;
    }
    if ticks.is_empty() {
        ticks.push(min);
    }
    ticks
}

fn sanitize_tick(value: f32) -> f32 {
    if value.abs() < 1e-6 {
        0.0
    } else {
        (value * 1_000.0).round() / 1_000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::units::SpeedUnit;

    #[test]
    fn fixed_value_ticks_cover_pco_axis() {
        let scale = fixed_value_ticks(-30.0..=30.0, 15.0);
        assert_eq!(scale.ticks, vec![-30.0, -15.0, 0.0, 15.0, 30.0]);
    }

    #[test]
    fn nice_step_picks_1_2_5_10_multiples() {
        assert_eq!(nice_step(0.23), 0.5);
        assert_eq!(nice_step(0.8), 1.0);
        assert_eq!(nice_step(3.4), 5.0);
        assert_eq!(nice_step(7.1), 10.0);
    }

    #[test]
    fn nice_value_ticks_expands_to_round_numbers() {
        let scale = nice_value_ticks(9.2, 47.8, false, 5, None);
        assert!(scale.ticks.first().copied().unwrap() <= 9.2);
        assert!(scale.ticks.last().copied().unwrap() >= 47.8);
        assert!(scale.step >= 5.0);
    }

    #[test]
    fn nice_time_ticks_use_minute_friendly_steps() {
        let scale = nice_time_ticks(4_166.0, 640.0, 64.0);
        assert_eq!(scale.ticks.first(), Some(&0.0));
        assert_eq!(scale.ticks.last(), Some(&4_166.0));
        assert!(scale.ticks.len() >= 4);
        assert!(scale.step >= 60.0);
    }

    #[test]
    fn nice_distance_ticks_end_at_ride_extent() {
        let scale = nice_distance_ticks(20.025, 640.0, 64.0, SpeedUnit::Mph);
        assert_eq!(*scale.range.end(), 20.025);
        assert!(scale.ticks.first() == Some(&0.0));
        assert!(scale.ticks.last().copied().unwrap() <= 20.025);
        assert!(scale.step <= 5.0);
    }

    #[test]
    fn speed_mph_range_uses_five_mph_steps() {
        let scale = nice_value_ticks(0.0, 47.0, true, 6, Some(5.0));
        assert_eq!(scale.step, 5.0);
        assert_eq!(scale.ticks, vec![0.0, 5.0, 10.0, 15.0, 20.0, 25.0, 30.0, 35.0, 40.0, 45.0, 50.0]);
    }
}
