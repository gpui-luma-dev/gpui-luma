use std::sync::Arc;

use gpui::{AnyElement, App, FontWeight, IntoElement, SharedString, Window, div, prelude::*, px};
use gpui_luma::controls::listbox::{ListBox, ListBoxItem};
use gpui_luma::controls::slider::Slider;
use gpui_luma::controls::toggle::Toggle;
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextSize};

use super::activity::{RideActivity, cumulative_distance_profile, distance_axis_available};
use super::charts::{
    SpeedHrOverlayChartModel, TelemetryChartModel, overlay_available, overlay_chart_title,
    render_speed_hr_overlay_chart, render_telemetry_chart,
};
use super::metrics::{TelemetryMetric, left_pco_samples};
use super::plot::SeriesStyle;
use super::ride_summary::{format_duration, render_ride_summary};
use super::units::{MetricDisplay, PowerUnit, SpeedUnit, XAxisMode};

pub fn render_graph_tab(
    ride: &RideActivity,
    visible_metrics: &[TelemetryMetric],
    scrub_fraction: f32,
    metric_listbox: ListBox,
    speed_unit_toggle: Toggle,
    power_unit_toggle: Toggle,
    x_axis_toggle: Toggle,
    timeline_slider: Slider,
    look: &Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let distance_profile = cumulative_distance_profile(&ride.points);
    let distance_available = distance_axis_available(&distance_profile);
    let units = chart_display(
        &speed_unit_toggle,
        &power_unit_toggle,
        ride.rider_weight_kg,
        &x_axis_toggle,
        distance_available,
        cx,
    );

    let mut sections: Vec<AnyElement> = vec![render_chart_controls(
        look,
        metric_listbox,
        speed_unit_toggle,
        power_unit_toggle,
        ride.rider_weight_kg.is_some(),
        x_axis_toggle,
        timeline_slider,
        ride,
        scrub_fraction,
        window,
        cx,
    )];

    sections.push(render_ride_summary(ride, units.speed_unit, look, window, cx).into_any_element());

    let duration = ride.summary.total_duration_seconds.max(f32::EPSILON);

    if overlay_available(&ride.points) {
        push_speed_hr_overlay_chart(&mut sections, &ride.points, duration, units, scrub_fraction, look, window, cx);
    }
    for metric in visible_metrics {
        if *metric == TelemetryMetric::Speed {
            push_metric_chart(
                &mut sections,
                *metric,
                SeriesStyle::Line,
                &ride.points,
                duration,
                units,
                scrub_fraction,
                look,
                window,
                cx,
            );
            push_metric_chart(
                &mut sections,
                *metric,
                SeriesStyle::AreaFilled,
                &ride.points,
                duration,
                units,
                scrub_fraction,
                look,
                window,
                cx,
            );
        } else {
            push_metric_chart(
                &mut sections,
                *metric,
                metric_series_style(*metric),
                &ride.points,
                duration,
                units,
                scrub_fraction,
                look,
                window,
                cx,
            );
        }
    }

    div().w_full().flex().flex_col().gap(px(24.0)).children(sections)
}

fn metric_series_style(metric: TelemetryMetric) -> SeriesStyle {
    match metric {
        TelemetryMetric::RightPlatformCenterOffset => SeriesStyle::Dots,
        _ => SeriesStyle::Line,
    }
}

fn push_speed_hr_overlay_chart(
    sections: &mut Vec<AnyElement>,
    points: &[super::activity::TelemetryPoint],
    duration: f32,
    units: MetricDisplay,
    scrub_fraction: f32,
    look: &Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut App,
) {
    let Some(chart) = SpeedHrOverlayChartModel::from_points(points, duration, units, look, scrub_fraction) else {
        return;
    };
    sections.push(render_speed_hr_overlay_chart(
        SharedString::from(overlay_chart_title(units)),
        chart,
        look,
        window,
        cx,
    ));
}

fn push_metric_chart(
    sections: &mut Vec<AnyElement>,
    metric: TelemetryMetric,
    series_style: SeriesStyle,
    points: &[super::activity::TelemetryPoint],
    duration: f32,
    units: MetricDisplay,
    scrub_fraction: f32,
    look: &Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut App,
) {
    let Some(chart) =
        TelemetryChartModel::from_metric(metric, points, duration, units, series_style, look, scrub_fraction)
    else {
        return;
    };
    sections.push(render_telemetry_chart(
        chart_id(metric, series_style),
        SharedString::from(chart_title(metric, units, series_style, points)),
        chart,
        look,
        window,
        cx,
    ));
}

fn chart_display(
    speed_unit_toggle: &Toggle,
    power_unit_toggle: &Toggle,
    rider_weight_kg: Option<f32>,
    x_axis_toggle: &Toggle,
    distance_available: bool,
    cx: &App,
) -> MetricDisplay {
    let x_axis = selected_x_axis(x_axis_toggle, cx);
    let power_unit = if rider_weight_kg.is_some() {
        PowerUnit::from_toggle(*power_unit_toggle.read(cx).data())
    } else {
        PowerUnit::Watts
    };
    MetricDisplay {
        speed_unit: selected_speed_unit(speed_unit_toggle, cx),
        x_axis: if distance_available && x_axis == XAxisMode::Distance {
            XAxisMode::Distance
        } else {
            XAxisMode::Time
        },
        power_unit,
        rider_weight_kg,
    }
}

fn selected_speed_unit(toggle: &Toggle, cx: &App) -> SpeedUnit {
    SpeedUnit::from_toggle(*toggle.read(cx).data())
}

fn selected_x_axis(toggle: &Toggle, cx: &App) -> XAxisMode {
    XAxisMode::from_toggle(*toggle.read(cx).data())
}

fn chart_title(
    metric: TelemetryMetric,
    units: MetricDisplay,
    series_style: SeriesStyle,
    points: &[super::activity::TelemetryPoint],
) -> String {
    match metric {
        TelemetryMetric::Speed if series_style == SeriesStyle::AreaFilled => {
            format!("Speed Filled ({})", units.speed_unit.label())
        }
        TelemetryMetric::Speed => format!("Speed ({})", units.speed_unit.label()),
        TelemetryMetric::Power => format!("Power ({})", units.power_unit.label()),
        TelemetryMetric::RightPlatformCenterOffset if left_pco_samples(points).len() >= 2 => {
            "Platform Center Offset (Right · Left)".to_string()
        }
        _ => metric.title().to_string(),
    }
}

fn render_chart_controls(
    look: &Arc<ShadcnLook>,
    metric_listbox: ListBox,
    speed_unit_toggle: Toggle,
    power_unit_toggle: Toggle,
    show_power_unit_toggle: bool,
    x_axis_toggle: Toggle,
    timeline_slider: Slider,
    ride: &RideActivity,
    scrub_fraction: f32,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let label_style = look.typography_scale(ShadcnTextSize::Sm);
    let value_style = look.typography_scale(ShadcnTextSize::Xs);
    let scrub_time = ride.summary.total_duration_seconds * scrub_fraction;

    look.card("graph-viz-chart-controls")
        .title("Chart Controls")
        .elevated(false)
        .child_render({
            let metric_listbox = metric_listbox.clone();
            let speed_unit_toggle = speed_unit_toggle.clone();
            let power_unit_toggle = power_unit_toggle.clone();
            let x_axis_toggle = x_axis_toggle.clone();
            let timeline_slider = timeline_slider.clone();
            let scrub_label = SharedString::from(format!(
                "Timeline: {} / {}",
                format_duration(scrub_time),
                format_duration(ride.summary.total_duration_seconds)
            ));
            move |_, _| {
                let section_label = |text: &'static str| {
                    div()
                        .typography_style(label_style)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(chrome.title_text)
                        .child(text)
                };
                let mut body = div().w_full().flex().flex_col().gap(px(12.0));
                body = body
                    .child(section_label("Metrics"))
                    .child(metric_listbox.clone())
                    .child(section_label("Speed Unit"))
                    .child(speed_unit_toggle.clone());
                if show_power_unit_toggle {
                    body = body.child(section_label("Power Unit")).child(power_unit_toggle.clone());
                }
                body.child(section_label("X Axis"))
                    .child(x_axis_toggle.clone())
                    .child(section_label("Timeline"))
                    .child(div().w_full().child(timeline_slider.clone()))
                    .child(div().typography_style(value_style).text_color(chrome.muted_text).child(scrub_label.clone()))
                    .into_any_element()
            }
        })
        .render(window, cx)
        .into_any_element()
}

fn chart_id(metric: TelemetryMetric, series_style: SeriesStyle) -> &'static str {
    match (metric, series_style) {
        (TelemetryMetric::Elevation, _) => "graph-viz-elevation-chart",
        (TelemetryMetric::HeartRate, _) => "graph-viz-heart-rate-chart",
        (TelemetryMetric::RespirationRate, _) => "graph-viz-respiration-chart",
        (TelemetryMetric::RightPlatformCenterOffset, _) => "graph-viz-right-pco-chart",
        (TelemetryMetric::Power, _) => "graph-viz-power-chart",
        (TelemetryMetric::Speed, SeriesStyle::AreaFilled) => "graph-viz-speed-filled-chart",
        (TelemetryMetric::Speed, SeriesStyle::Line) => "graph-viz-speed-chart",
        (_, SeriesStyle::Dots) => "graph-viz-dot-chart",
    }
}

pub fn power_unit_toggle_label(w_per_kg_selected: bool) -> SharedString {
    SharedString::from(if w_per_kg_selected { "W/kg" } else { "W" })
}

pub fn metric_listbox_items(metrics: &[TelemetryMetric]) -> Vec<ListBoxItem> {
    metrics
        .iter()
        .map(|metric| ListBoxItem::new(metric.id(), metric.id()).label(metric.title()))
        .collect()
}

pub fn default_selected_metric_ids(metrics: &[TelemetryMetric]) -> Vec<SharedString> {
    metrics.iter().map(|metric| SharedString::from(metric.id())).collect()
}

pub fn visible_metrics_from_ids(available: &[TelemetryMetric], selected_ids: &[SharedString]) -> Vec<TelemetryMetric> {
    TelemetryMetric::ALL
        .iter()
        .copied()
        .filter(|metric| available.contains(metric) && selected_ids.iter().any(|id| id.as_ref() == metric.id()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_metrics_respect_selection_and_availability() {
        let available = vec![TelemetryMetric::Elevation, TelemetryMetric::HeartRate];
        let selected = vec![SharedString::from("elevation"), SharedString::from("power")];
        assert_eq!(visible_metrics_from_ids(&available, &selected), vec![TelemetryMetric::Elevation]);
    }
}
