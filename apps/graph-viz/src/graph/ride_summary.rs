use std::sync::Arc;

use gpui::{App, FontWeight, IntoElement, SharedString, Window, div, prelude::*, px};
use gpui_luma::vstack;
use gpui_luma::theme::LumaTextStyle;
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextSize};

use super::activity::{RideActivity, RideSummary};
use super::units::SpeedUnit;

const STAT_MIN_WIDTH: f32 = 88.0;

pub fn render_ride_summary(
    ride: &RideActivity,
    speed_unit: SpeedUnit,
    look: &Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let chrome = look.chrome();
    let label_style = look.typography_scale(ShadcnTextSize::Xs);
    let value_style = look.typography_scale(ShadcnTextSize::Sm);
    let metrics = metric_specs(&ride.summary, speed_unit);
    let label_color = chrome.muted_text;
    let value_color = chrome.title_text;

    look.card("graph-viz-ride-summary")
        .title("Ride Summary")
        .description(format!("{} telemetry points", ride.points.len()))
        .elevated(false)
        .child_render({
            let metrics = metrics.clone();
            let surface_id = SharedString::from(format!("ride-summary-{}", speed_unit.label()));
            move |_, _| {
                let stat_cells: Vec<_> = metrics
                    .clone()
                    .into_iter()
                    .map(|metric| {
                        render_stat_cell(
                            metric.title,
                            metric.value,
                            label_style,
                            value_style,
                            label_color,
                            value_color,
                            STAT_MIN_WIDTH,
                        )
                        .into_any_element()
                    })
                    .collect();

                div()
                    .id(surface_id.clone())
                    .w_full()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(px(16.0))
                    .items_start()
                    .children(stat_cells)
                    .into_any_element()
            }
        })
        .render(window, cx)
}

#[derive(Clone)]
pub struct StatSpec {
    pub title: &'static str,
    pub value: String,
}

fn metric_specs(summary: &RideSummary, speed_unit: SpeedUnit) -> Vec<StatSpec> {
    let mut metrics = vec![
        StatSpec { title: "Distance", value: speed_unit.format_distance(summary.total_distance_meters) },
        StatSpec { title: "Duration", value: format_duration(summary.total_duration_seconds) },
        StatSpec { title: "Gain", value: format_elevation(summary.elevation_gain_meters) },
        StatSpec { title: "Speed", value: format_speed(summary.avg_speed_mps, speed_unit) },
    ];

    if let Some(avg_heart_rate) = summary.avg_heart_rate {
        metrics.push(StatSpec { title: "HR", value: format_heart_rate(avg_heart_rate) });
    }
    if let Some(avg_power) = summary.avg_power {
        metrics.push(StatSpec { title: "Power", value: format_power(avg_power) });
    }

    metrics
}

pub(crate) fn render_stat_cell(
    label: &'static str,
    value: String,
    label_style: LumaTextStyle,
    value_style: LumaTextStyle,
    label_color: gpui::Hsla,
    value_color: gpui::Hsla,
    min_width: f32,
) -> gpui::Div {
    vstack! {
        gap=2 align=start;
        div()
            .min_w(px(min_width))
            .typography_style(label_style)
            .text_color(label_color)
            .child(label),
        div()
            .typography_style(value_style)
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(value_color)
            .child(value),
    }
}

pub fn format_duration(seconds: f32) -> String {
    let total = seconds.round().max(0.0) as u32;
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let secs = total % 60;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{secs:02}")
    } else {
        format!("{minutes}:{secs:02}")
    }
}

pub fn format_elevation(meters: f32) -> String {
    format_elevation_for_unit(meters, SpeedUnit::Kmh)
}

pub fn format_elevation_for_unit(meters: f32, unit: SpeedUnit) -> String {
    match unit {
        SpeedUnit::Mph => format!("{:.0} ft", meters * 3.28084),
        SpeedUnit::Kmh => format!("{:.0} m", meters),
    }
}

pub fn format_speed(meters_per_second: f32, unit: SpeedUnit) -> String {
    format!("{:.1} {}", unit.mps_to_display(meters_per_second), unit.label())
}

pub fn format_heart_rate(beats_per_minute: f32) -> String {
    format!("{:.0} bpm", beats_per_minute)
}

pub fn format_power(watts: f32) -> String {
    format!("{:.0} W", watts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_ride_summary_values() {
        let summary = RideSummary {
            total_distance_meters: 32_091.2,
            total_duration_seconds: 4_166.0,
            elevation_gain_meters: 139.0,
            avg_speed_mps: 7.7,
            avg_heart_rate: Some(143.0),
            avg_power: Some(178.0),
        };

        assert_eq!(SpeedUnit::Kmh.format_distance(summary.total_distance_meters), "32.1 km");
        assert_eq!(SpeedUnit::Mph.format_distance(summary.total_distance_meters), "19.9 mi");
        assert_eq!(format_duration(summary.total_duration_seconds), "1:09:26");
        assert_eq!(format_elevation(summary.elevation_gain_meters), "139 m");
        assert_eq!(format_elevation_for_unit(summary.elevation_gain_meters, SpeedUnit::Mph), "456 ft");
        assert_eq!(format_speed(summary.avg_speed_mps, SpeedUnit::Kmh), "27.7 km/h");
        assert_eq!(format_speed(summary.avg_speed_mps, SpeedUnit::Mph), "17.2 mph");
        assert_eq!(format_heart_rate(summary.avg_heart_rate.unwrap()), "143 bpm");
        assert_eq!(format_power(summary.avg_power.unwrap()), "178 W");
    }
}
