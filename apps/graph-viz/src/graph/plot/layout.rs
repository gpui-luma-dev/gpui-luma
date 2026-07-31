use gpui::Window;

use super::axes::{AxisLabelStyle, RightAxisLabelsLayer, measure_max_label_width};
use super::domain::PlotDomain2D;
use super::ticks::{fixed_value_ticks, nice_value_ticks};
use super::viewport::ChartMargins;
use super::x_axis::fit_x_axis_margins;
use super::{
    AreaFillLayer, AxisLabelsLayer, ChartStack, DotsLayer, GridLinesLayer, InteractionCursorLayer, LinePathLayer,
    SeriesStyle, minmax_decimate, uniform_subsample,
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
const MAX_SCATTER_POINTS: usize = 2_500;
const SCATTER_DOT_RADIUS: f32 = 2.5;

pub fn build_telemetry_chart_stack(
    metric: TelemetryMetric,
    samples: &[(f32, f32)],
    duration_seconds: f32,
    _total_distance_meters: f32,
    distance_profile: &[(f32, f32)],
    canvas_width_px: f32,
    units: MetricDisplay,
    background_color: gpui::Hsla,
    grid_color: gpui::Hsla,
    cursor_color: gpui::Hsla,
    stroke_color: gpui::Hsla,
    fill_color: Option<gpui::Hsla>,
    series_style: SeriesStyle,
    time_scrub_fraction: f32,
    overlay_dot_samples: Option<&[(f32, f32)]>,
    overlay_dot_color: Option<gpui::Hsla>,
    axis_style: &AxisLabelStyle,
    window: &mut Window,
) -> Option<ChartStack> {
    let plot_samples = remap_time_samples_to_x(samples, distance_profile, units.x_axis, units.speed_unit);
    if plot_samples.len() < 2 {
        return None;
    }

    let y_inverted = metric.invert_y_axis();
    let y_display_scale = if let Some(fixed_range) = metric.fixed_y_display_range() {
        let step = metric.preferred_y_display_step(units).unwrap_or(15.0);
        fixed_value_ticks(fixed_range, step)
    } else {
        let display_min = plot_samples.iter().map(|(_, y)| metric.to_display(*y, units)).fold(f32::INFINITY, f32::min);
        let display_max =
            plot_samples.iter().map(|(_, y)| metric.to_display(*y, units)).fold(f32::NEG_INFINITY, f32::max);
        nice_value_ticks(
            display_min,
            display_max,
            metric.include_zero_on_axis(),
            TARGET_Y_TICKS,
            metric.preferred_y_display_step(units),
        )
    };
    let y_range = metric.display_to_internal(*y_display_scale.range.start(), units)
        ..=metric.display_to_internal(*y_display_scale.range.end(), units);
    let y_ticks: Vec<f32> = y_display_scale.ticks.iter().map(|tick| metric.display_to_internal(*tick, units)).collect();
    let y_labels: Vec<String> = y_display_scale
        .ticks
        .iter()
        .map(|value| metric.format_display_tick(*value, y_display_scale.step))
        .collect();
    let has_pco_overlay = metric == TelemetryMetric::RightPlatformCenterOffset
        && overlay_dot_samples.is_some_and(|samples| samples.len() >= 2);
    let y_label_width = measure_max_label_width(window, axis_style, &y_labels) + LABEL_GAP;
    let (left_margin, min_right_margin, left_y_inverted) = if metric == TelemetryMetric::RightPlatformCenterOffset {
        if has_pco_overlay {
            (y_label_width.max(MIN_LEFT_MARGIN), y_label_width.max(MIN_RIGHT_MARGIN), true)
        } else {
            (y_label_width.max(MIN_LEFT_MARGIN), MIN_RIGHT_MARGIN, true)
        }
    } else {
        (y_label_width.max(MIN_LEFT_MARGIN), MIN_RIGHT_MARGIN, y_inverted)
    };

    let distance_extent = ride_distance_display_extent(distance_profile, duration_seconds, units.speed_unit);
    let (right_margin, x_layout) = fit_x_axis_margins(
        units,
        duration_seconds,
        distance_extent,
        canvas_width_px,
        left_margin,
        min_right_margin,
        LABEL_GAP,
        axis_style,
        window,
    );

    let margins = ChartMargins { left: left_margin, right: right_margin, top: TOP_MARGIN, bottom: BOTTOM_MARGIN };
    let plot_width = (canvas_width_px - margins.left - margins.right).max(1.0);
    let x_range = x_layout.x_min..=x_layout.x_max;
    let rendered_points = match series_style {
        SeriesStyle::Dots => uniform_subsample(&plot_samples, MAX_SCATTER_POINTS),
        _ => minmax_decimate(&plot_samples, plot_width as usize),
    };
    let stroke_width = if series_style == SeriesStyle::AreaFilled {
        gpui::px(1.5)
    } else {
        gpui::px(2.0)
    };
    let area_fill = (series_style == SeriesStyle::AreaFilled)
        .then_some(fill_color)
        .flatten()
        .map(|fill_color| AreaFillLayer { points: rendered_points.clone(), fill_color });
    let dots = (series_style == SeriesStyle::Dots).then(|| DotsLayer {
        points: rendered_points.clone(),
        color: stroke_color,
        radius: gpui::px(SCATTER_DOT_RADIUS),
    });
    let overlay_dots = overlay_dot_samples.filter(|samples| samples.len() >= 2).map(|samples| {
        let plot_overlay = remap_time_samples_to_x(samples, distance_profile, units.x_axis, units.speed_unit);
        DotsLayer {
            points: uniform_subsample(&plot_overlay, MAX_SCATTER_POINTS),
            color: overlay_dot_color.unwrap_or(stroke_color),
            radius: gpui::px(SCATTER_DOT_RADIUS),
        }
    });
    let line = if series_style == SeriesStyle::Dots {
        LinePathLayer { points: Vec::new(), stroke_width, color: stroke_color }
    } else {
        LinePathLayer { points: rendered_points, stroke_width, color: stroke_color }
    };
    let cursor_x_fraction = scrub_x_fraction(
        time_scrub_fraction,
        duration_seconds,
        distance_profile,
        x_layout.x_min,
        x_layout.x_max,
        units.x_axis,
        units.speed_unit,
    );
    let zero_baseline_color = metric.emphasize_zero_baseline().then_some(grid_color.opacity(0.95));
    let right_axes = has_pco_overlay.then(|| RightAxisLabelsLayer {
        y_tick_values: y_ticks.clone(),
        y_tick_step_display: y_display_scale.step,
        metric,
        units,
        y_inverted: false,
        style: axis_style.clone(),
    });

    Some(ChartStack {
        background_color,
        x_domain: PlotDomain2D::new(x_range.clone(), y_range.clone()),
        y_domain: PlotDomain2D::new(x_range, y_range),
        margins,
        y_inverted,
        zero_baseline_color,
        grid: GridLinesLayer { x_tick_values: x_layout.ticks.clone(), y_tick_values: y_ticks.clone(), grid_color },
        axes: AxisLabelsLayer {
            x_tick_values: x_layout.ticks,
            y_tick_values: y_ticks,
            x_tick_step: x_layout.tick_step,
            y_tick_step_display: y_display_scale.step,
            metric,
            units,
            y_inverted: left_y_inverted,
            style: axis_style.clone(),
        },
        right_axes,
        area_fill,
        dots,
        overlay_dots,
        line,
        cursor: InteractionCursorLayer {
            active_fraction: cursor_x_fraction,
            cursor_color,
            marker_color: stroke_color,
            marker_samples: plot_samples,
        },
    })
}
