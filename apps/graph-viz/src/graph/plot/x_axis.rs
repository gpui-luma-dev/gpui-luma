use gpui::Window;

use super::axes::{AxisLabelStyle, measure_label_widths};
use super::ticks::{cull_ticks_by_label_width, nice_distance_ticks, nice_time_ticks};
use crate::graph::ride_summary::format_duration;
use crate::graph::units::{MetricDisplay, SpeedUnit, XAxisMode};

const MIN_X_LABEL_PX: f32 = 72.0;

#[derive(Clone, Debug, PartialEq)]
pub struct XAxisLayout {
    pub x_min: f32,
    pub x_max: f32,
    pub ticks: Vec<f32>,
    pub tick_step: f32,
}

pub fn format_x_axis_tick(value: f32, step: f32, units: MetricDisplay) -> String {
    match units.x_axis {
        XAxisMode::Time => format_duration(value),
        XAxisMode::Distance => units.speed_unit.format_distance_axis_tick(value, step),
    }
}

pub fn x_axis_span(units: MetricDisplay, duration_seconds: f32, distance_extent_display: f32) -> f32 {
    match units.x_axis {
        XAxisMode::Time => duration_seconds.max(f32::EPSILON),
        XAxisMode::Distance => distance_extent_display.max(f32::EPSILON),
    }
}

pub fn layout_x_axis(
    units: MetricDisplay,
    duration_seconds: f32,
    distance_extent_display: f32,
    plot_width_px: f32,
) -> XAxisLayout {
    let x_min = 0.0;
    match units.x_axis {
        XAxisMode::Time => {
            let duration = duration_seconds.max(f32::EPSILON);
            let scale = nice_time_ticks(duration, plot_width_px, MIN_X_LABEL_PX);
            XAxisLayout { x_min, x_max: duration, ticks: scale.ticks, tick_step: scale.step }
        }
        XAxisMode::Distance => {
            let x_max = distance_extent_display.max(f32::EPSILON);
            let scale = nice_distance_ticks(x_max, plot_width_px, MIN_X_LABEL_PX, units.speed_unit);
            XAxisLayout { x_min, x_max, ticks: scale.ticks, tick_step: scale.step }
        }
    }
}

pub fn fit_x_axis_margins(
    units: MetricDisplay,
    duration_seconds: f32,
    distance_extent_display: f32,
    canvas_width_px: f32,
    left_margin: f32,
    min_right_margin: f32,
    label_gap: f32,
    axis_style: &AxisLabelStyle,
    window: &mut Window,
) -> (f32, XAxisLayout) {
    let mut right_margin = min_right_margin;
    let mut layout = layout_x_axis(units, duration_seconds, distance_extent_display, 1.0);

    for _ in 0..2 {
        let plot_width = (canvas_width_px - left_margin - right_margin).max(1.0);
        layout = layout_x_axis(units, duration_seconds, distance_extent_display, plot_width);
        let x_labels: Vec<String> =
            layout.ticks.iter().map(|tick| format_x_axis_tick(*tick, layout.tick_step, units)).collect();
        let x_widths = measure_label_widths(window, axis_style, &x_labels);
        let culled_ticks = cull_ticks_by_label_width(
            &layout.ticks,
            &x_widths,
            layout.x_min,
            layout.x_max,
            plot_width,
            MIN_X_LABEL_PX * 0.35,
        );
        let culled_labels: Vec<String> =
            culled_ticks.iter().map(|tick| format_x_axis_tick(*tick, layout.tick_step, units)).collect();
        let culled_widths = measure_label_widths(window, axis_style, &culled_labels);
        let end_half = culled_widths.last().copied().unwrap_or(0.0) / 2.0;
        layout.ticks = culled_ticks;
        right_margin = (end_half + label_gap).max(min_right_margin);
    }

    (right_margin, layout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::units::SpeedUnit;

    #[test]
    fn distance_axis_span_uses_speed_unit() {
        let mph = MetricDisplay { speed_unit: SpeedUnit::Mph, x_axis: XAxisMode::Distance, ..MetricDisplay::default() };
        let kmh = MetricDisplay { speed_unit: SpeedUnit::Kmh, x_axis: XAxisMode::Distance, ..MetricDisplay::default() };
        assert!((x_axis_span(mph, 100.0, 10.0) - 10.0).abs() < 0.01);
        assert!((x_axis_span(kmh, 100.0, 16.093) - 16.093).abs() < 0.01);
    }

    #[test]
    fn sample_ride_distance_axis_covers_remapped_speed_extent() {
        use crate::graph::activity::{
            cumulative_distance_profile, load_sample_ride, remap_time_samples_to_x, total_distance_meters,
        };
        use crate::graph::metrics::TelemetryMetric;

        let ride = load_sample_ride().expect("sample ride");
        let profile = cumulative_distance_profile(&ride.points);
        let duration = ride.summary.total_duration_seconds;
        let distance_extent = crate::graph::activity::ride_distance_display_extent(&profile, duration, SpeedUnit::Mph);
        let speed_samples = TelemetryMetric::Speed.samples(&ride.points);
        let units =
            MetricDisplay { speed_unit: SpeedUnit::Mph, x_axis: XAxisMode::Distance, ..MetricDisplay::default() };
        let remapped = remap_time_samples_to_x(&speed_samples, &profile, units.x_axis, units.speed_unit);
        let layout = layout_x_axis(units, duration, distance_extent, 640.0);

        let max_sample_x = remapped.iter().map(|(x, _)| *x).fold(f32::NEG_INFINITY, f32::max);
        let time_layout = layout_x_axis(
            MetricDisplay { speed_unit: SpeedUnit::Mph, x_axis: XAxisMode::Time, ..MetricDisplay::default() },
            duration,
            distance_extent,
            640.0,
        );
        let max_time_x = speed_samples.iter().map(|(x, _)| *x).fold(f32::NEG_INFINITY, f32::max);

        eprintln!(
            "summary_dist={:.1}m profile_total={:.1}m duration={:.0}s extent={:.3}mi",
            ride.summary.total_distance_meters,
            total_distance_meters(&profile),
            duration,
            distance_extent
        );
        eprintln!(
            "time x_max={:.1} max_sample={:.1} fill={:.3}",
            time_layout.x_max,
            max_time_x,
            max_time_x / time_layout.x_max
        );
        eprintln!(
            "dist x_max={:.3} max_sample={:.3} fill={:.3} tick_step={:.3}",
            layout.x_max,
            max_sample_x,
            max_sample_x / layout.x_max,
            layout.tick_step
        );

        assert!(max_sample_x <= layout.x_max + 0.01, "sample x {max_sample_x} exceeds axis max {}", layout.x_max);
        assert!(
            max_sample_x / layout.x_max > 0.95,
            "distance axis should fill most of plot, got {}",
            max_sample_x / layout.x_max
        );
    }
}
