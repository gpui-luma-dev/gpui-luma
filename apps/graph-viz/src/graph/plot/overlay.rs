use gpui::Window;

use super::axes::{AxisLabelStyle, RightAxisLabelsLayer, YAxisLabelSide, measure_max_label_width};
use super::domain::PlotDomain2D;
use super::ticks::nice_value_ticks;
use super::viewport::ChartMargins;
use super::x_axis::fit_x_axis_margins;
use super::{
    AreaFillLayer, AxisLabelsLayer, DualAxisOverlayStack, GridLinesLayer, InteractionCursorLayer, LinePathLayer,
    minmax_decimate,
};
use crate::graph::activity::{remap_time_samples_to_x, ride_distance_display_extent, scrub_x_fraction};
use crate::graph::metrics::TelemetryMetric;
use crate::graph::units::MetricDisplay;

const TARGET_Y_TICKS: usize = 6;
const MIN_LEFT_MARGIN: f32 = 40.0;
const MIN_RIGHT_MARGIN: f32 = 16.0;
const LABEL_GAP: f32 = 10.0;
const TOP_MARGIN: f32 = 8.0;
const BOTTOM_MARGIN: f32 = 22.0;

pub fn build_speed_hr_overlay_stack(
    speed_samples: &[(f32, f32)],
    hr_samples: &[(f32, f32)],
    duration_seconds: f32,
    _total_distance_meters: f32,
    distance_profile: &[(f32, f32)],
    canvas_width_px: f32,
    units: MetricDisplay,
    background_color: gpui::Hsla,
    grid_color: gpui::Hsla,
    cursor_color: gpui::Hsla,
    speed_fill_color: gpui::Hsla,
    speed_stroke_color: gpui::Hsla,
    hr_stroke_color: gpui::Hsla,
    time_scrub_fraction: f32,
    axis_style: &AxisLabelStyle,
    window: &mut Window,
) -> Option<DualAxisOverlayStack> {
    let speed_plot_samples = remap_time_samples_to_x(speed_samples, distance_profile, units.x_axis, units.speed_unit);
    let hr_plot_samples = remap_time_samples_to_x(hr_samples, distance_profile, units.x_axis, units.speed_unit);
    if speed_plot_samples.len() < 2 || hr_plot_samples.len() < 2 {
        return None;
    }

    let speed_metric = TelemetryMetric::Speed;
    let hr_metric = TelemetryMetric::HeartRate;

    let speed_display_min = speed_plot_samples
        .iter()
        .map(|(_, y)| speed_metric.to_display(*y, units))
        .fold(f32::INFINITY, f32::min);
    let speed_display_max = speed_plot_samples
        .iter()
        .map(|(_, y)| speed_metric.to_display(*y, units))
        .fold(f32::NEG_INFINITY, f32::max);
    let speed_y_display_scale = nice_value_ticks(
        speed_display_min,
        speed_display_max,
        speed_metric.include_zero_on_axis(),
        TARGET_Y_TICKS,
        speed_metric.preferred_y_display_step(units),
    );
    let speed_y_range = speed_metric.from_display(*speed_y_display_scale.range.start(), units)
        ..=speed_metric.from_display(*speed_y_display_scale.range.end(), units);
    let speed_y_ticks: Vec<f32> =
        speed_y_display_scale.ticks.iter().map(|tick| speed_metric.from_display(*tick, units)).collect();
    let speed_y_labels: Vec<String> = speed_y_display_scale
        .ticks
        .iter()
        .map(|value| speed_metric.format_display_tick(*value, speed_y_display_scale.step))
        .collect();
    let left_margin = (measure_max_label_width(window, axis_style, &speed_y_labels) + LABEL_GAP).max(MIN_LEFT_MARGIN);

    let hr_display_min = hr_plot_samples.iter().map(|(_, y)| *y).fold(f32::INFINITY, f32::min);
    let hr_display_max = hr_plot_samples.iter().map(|(_, y)| *y).fold(f32::NEG_INFINITY, f32::max);
    let hr_y_display_scale = nice_value_ticks(
        hr_display_min,
        hr_display_max,
        hr_metric.include_zero_on_axis(),
        TARGET_Y_TICKS,
        hr_metric.preferred_y_display_step(units),
    );
    let hr_y_range = *hr_y_display_scale.range.start()..=*hr_y_display_scale.range.end();
    let hr_y_ticks: Vec<f32> = hr_y_display_scale.ticks.clone();
    let hr_y_labels: Vec<String> = hr_y_display_scale
        .ticks
        .iter()
        .map(|value| hr_metric.format_display_tick(*value, hr_y_display_scale.step))
        .collect();
    let hr_label_width = measure_max_label_width(window, axis_style, &hr_y_labels);

    let distance_extent = ride_distance_display_extent(distance_profile, duration_seconds, units.speed_unit);
    let (mut right_margin, x_layout) = fit_x_axis_margins(
        units,
        duration_seconds,
        distance_extent,
        canvas_width_px,
        left_margin,
        MIN_RIGHT_MARGIN,
        LABEL_GAP,
        axis_style,
        window,
    );
    right_margin = right_margin.max(hr_label_width + LABEL_GAP);

    let margins = ChartMargins { left: left_margin, right: right_margin, top: TOP_MARGIN, bottom: BOTTOM_MARGIN };
    let plot_width = (canvas_width_px - margins.left - margins.right).max(1.0);
    let x_range = x_layout.x_min..=x_layout.x_max;
    let speed_decimated = minmax_decimate(&speed_plot_samples, plot_width as usize);
    let hr_decimated = minmax_decimate(&hr_plot_samples, plot_width as usize);
    let cursor_x_fraction = scrub_x_fraction(
        time_scrub_fraction,
        duration_seconds,
        distance_profile,
        x_layout.x_min,
        x_layout.x_max,
        units.x_axis,
        units.speed_unit,
    );

    Some(DualAxisOverlayStack {
        background_color,
        x_domain: PlotDomain2D::new(x_range.clone(), speed_y_range.clone()),
        primary_y_domain: PlotDomain2D::new(x_range.clone(), speed_y_range.clone()),
        overlay_y_domain: PlotDomain2D::new(x_range, hr_y_range),
        margins,
        grid: GridLinesLayer {
            x_tick_values: x_layout.ticks.clone(),
            y_tick_values: speed_y_ticks.clone(),
            grid_color,
        },
        left_axes: AxisLabelsLayer {
            x_tick_values: x_layout.ticks,
            y_tick_values: speed_y_ticks,
            x_tick_step: x_layout.tick_step,
            y_tick_step_display: speed_y_display_scale.step,
            metric: speed_metric,
            units,
            y_inverted: false,
            y_label_side: YAxisLabelSide::Left,
            style: axis_style.clone(),
        },
        right_axes: RightAxisLabelsLayer {
            y_tick_values: hr_y_ticks,
            y_tick_step_display: hr_y_display_scale.step,
            metric: hr_metric,
            units,
            y_inverted: false,
            style: axis_style.clone(),
        },
        area_fill: AreaFillLayer { points: speed_decimated.clone(), fill_color: speed_fill_color },
        base_line: LinePathLayer { points: speed_decimated, stroke_width: gpui::px(1.5), color: speed_stroke_color },
        overlay_line: LinePathLayer { points: hr_decimated, stroke_width: gpui::px(2.0), color: hr_stroke_color },
        cursor: InteractionCursorLayer {
            active_fraction: cursor_x_fraction,
            cursor_color,
            marker_color: speed_fill_color,
            marker_samples: speed_plot_samples,
        },
    })
}
