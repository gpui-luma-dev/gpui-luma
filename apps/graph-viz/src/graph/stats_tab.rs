use std::sync::Arc;

use gpui::{AnyElement, App, FontWeight, IntoElement, SharedString, Window, div, prelude::*, px};
use gpui_luma::vstack;
use gpui_luma::theme::LumaTextStyle;
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextSize};

use super::activity::{ActivityStats, RideActivity};
use super::ride_summary::{format_duration, format_heart_rate, format_power, format_speed, render_stat_cell, StatSpec};
use super::units::SpeedUnit;

const STAT_MIN_WIDTH: f32 = 108.0;
const SECTION_MIN_WIDTH: f32 = 240.0;

pub fn render_stats_tab(
    ride: &RideActivity,
    speed_unit: SpeedUnit,
    look: &Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let section_title_style = look.typography_scale(ShadcnTextSize::Sm);
    let label_style = look.typography_scale(ShadcnTextSize::Xs);
    let value_style = look.typography_scale(ShadcnTextSize::Sm);
    let sections = stats_sections(&ride.stats, speed_unit);
    let surface_id = SharedString::from(format!("ride-stats-{}", speed_unit.label()));

    look.card("graph-viz-ride-stats")
        .title("Activity Stats")
        .description(stats_description(&ride.stats))
        .elevated(false)
        .child_render({
            let sections = sections.clone();
            move |_, _| {
                let section_cards: Vec<_> = sections
                    .iter()
                    .map(|section| {
                        render_stats_section(
                            section.title,
                            &section.stats,
                            section_title_style,
                            label_style,
                            value_style,
                            chrome.title_text,
                            chrome.muted_text,
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
                    .gap(px(20.0))
                    .items_start()
                    .children(section_cards)
                    .into_any_element()
            }
        })
        .render(window, cx)
        .into_any_element()
}

#[derive(Clone)]
struct StatsSection {
    title: &'static str,
    stats: Vec<StatSpec>,
}

fn stats_description(stats: &ActivityStats) -> String {
    let mut parts = Vec::new();
    if let Some(sport) = &stats.sport {
        parts.push(capitalize(sport));
    }
    if let Some(laps) = stats.num_laps {
        parts.push(format!("{laps} laps"));
    }
    if parts.is_empty() {
        "From FIT session data".to_string()
    } else {
        parts.join(" · ")
    }
}

fn stats_sections(stats: &ActivityStats, unit: SpeedUnit) -> Vec<StatsSection> {
    let mut sections = Vec::new();

    push_section(
        &mut sections,
        "Distance & Time",
        vec![
            stat("Distance", unit.format_distance(stats.total_distance_meters)),
            stat("Elapsed Time", format_duration(stats.elapsed_time_seconds)),
            stat("Moving Time", format_duration(stats.timer_time_seconds)),
            stat_opt("Laps", stats.num_laps.map(|laps| laps.to_string())),
        ],
    );

    push_section(
        &mut sections,
        "Speed",
        vec![
            stat("Avg Speed", format_speed(stats.avg_speed_mps, unit)),
            stat_opt("Max Speed", stats.max_speed_mps.map(|speed| format_speed(speed, unit))),
        ],
    );

    push_section(
        &mut sections,
        "Elevation",
        vec![
            stat("Total Ascent", format_elevation(stats.total_ascent_meters, unit)),
            stat_opt("Total Descent", stats.total_descent_meters.map(|m| format_elevation(m, unit))),
            stat_opt("Min Elev", stats.min_elevation_meters.map(|m| format_elevation(m, unit))),
            stat_opt("Max Elev", stats.max_elevation_meters.map(|m| format_elevation(m, unit))),
        ],
    );

    push_section(
        &mut sections,
        "Heart Rate",
        vec![
            stat_opt("Avg HR", stats.avg_heart_rate.map(format_heart_rate)),
            stat_opt("Max HR", stats.max_heart_rate.map(format_heart_rate)),
        ],
    );

    push_section(
        &mut sections,
        "Power",
        vec![
            stat_opt("Avg Power", stats.avg_power.map(format_power)),
            stat_opt("Max Power", stats.max_power.map(format_power)),
            stat_opt("Normalized Power", stats.normalized_power.map(format_power)),
            stat_opt("Intensity Factor", stats.intensity_factor.map(format_factor)),
            stat_opt("Training Stress Score", stats.training_stress_score.map(format_tss)),
            stat_opt("FTP Setting", stats.threshold_power.map(format_power)),
            stat_opt("Work", stats.total_work_joules.map(format_work)),
            stat_opt("L/R Balance", stats.left_right_balance_label()),
        ],
    );

    push_section(
        &mut sections,
        "Cadence",
        vec![
            stat_opt("Avg Cadence", stats.avg_cadence.map(format_cadence)),
            stat_opt("Max Cadence", stats.max_cadence.map(format_cadence)),
            stat_opt("Total Strokes", stats.total_strokes.map(|strokes| strokes.to_string())),
        ],
    );

    push_section(
        &mut sections,
        "Training Effect",
        vec![
            stat_opt("Aerobic TE", stats.aerobic_training_effect.map(format_training_effect)),
            stat_opt("Anaerobic TE", stats.anaerobic_training_effect.map(format_training_effect)),
            stat_opt("Exercise Load", stats.training_load_peak.map(format_load)),
        ],
    );

    push_section(
        &mut sections,
        "Calories",
        vec![stat_opt("Total Calories", stats.total_calories.map(format_calories))],
    );

    push_section(
        &mut sections,
        "Temperature",
        vec![
            stat_opt("Avg Temp", stats.avg_temperature_c.map(|c| format_temperature(c, unit))),
            stat_opt("Min Temp", stats.min_temperature_c.map(|c| format_temperature(c, unit))),
            stat_opt("Max Temp", stats.max_temperature_c.map(|c| format_temperature(c, unit))),
        ],
    );

    push_section(
        &mut sections,
        "Respiration",
        vec![
            stat_opt("Avg Rate", stats.avg_respiration.map(format_respiration)),
            stat_opt("Min Rate", stats.min_respiration.map(format_respiration)),
            stat_opt("Max Rate", stats.max_respiration.map(format_respiration)),
        ],
    );

    push_section(
        &mut sections,
        "Cycling Dynamics",
        vec![
            stat_opt("Standing Time", stats.time_standing_seconds.map(format_duration)),
            stat_opt("Stand Count", stats.stand_count.map(|count| count.to_string())),
            stat_opt("Avg Seated Power", stats.avg_seated_power.map(format_power)),
            stat_opt("Avg Standing Power", stats.avg_standing_power.map(format_power)),
            stat_opt("Max Seated Power", stats.max_seated_power.map(format_power)),
            stat_opt("Max Standing Power", stats.max_standing_power.map(format_power)),
        ],
    );

    sections
}

fn push_section(sections: &mut Vec<StatsSection>, title: &'static str, stats: Vec<StatSpec>) {
    let stats: Vec<_> = stats.into_iter().filter(|entry| !entry.value.is_empty()).collect();
    if stats.is_empty() {
        return;
    }
    sections.push(StatsSection { title, stats });
}

fn stat(title: &'static str, value: String) -> StatSpec {
    StatSpec { title, value }
}

fn stat_opt(title: &'static str, value: Option<String>) -> StatSpec {
    StatSpec { title, value: value.unwrap_or_default() }
}

fn render_stats_section(
    title: &'static str,
    stats: &[StatSpec],
    section_title_style: LumaTextStyle,
    label_style: LumaTextStyle,
    value_style: LumaTextStyle,
    title_color: gpui::Hsla,
    label_color: gpui::Hsla,
) -> gpui::Div {
    let cells: Vec<_> = stats
        .iter()
        .filter(|entry| !entry.value.is_empty())
        .map(|entry| {
            render_stat_cell(
                entry.title,
                entry.value.clone(),
                label_style,
                value_style,
                label_color,
                title_color,
                STAT_MIN_WIDTH,
            )
            .into_any_element()
        })
        .collect();

    vstack! {
        gap=10 align=start;
        div()
            .min_w(px(SECTION_MIN_WIDTH))
            .typography_style(section_title_style)
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(title_color)
            .child(title),
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(12.0))
            .children(cells),
    }
}

fn format_elevation(meters: f32, unit: SpeedUnit) -> String {
    match unit {
        SpeedUnit::Mph => format!("{:.0} ft", meters * 3.28084),
        SpeedUnit::Kmh => format!("{:.0} m", meters),
    }
}

fn format_cadence(rpm: f32) -> String {
    format!("{rpm:.0} rpm")
}

fn format_factor(factor: f32) -> String {
    format!("{factor:.3}")
}

fn format_tss(tss: f32) -> String {
    format!("{tss:.1}")
}

fn format_work(joules: f32) -> String {
    format!("{:.0} kJ", joules / 1000.0)
}

fn format_calories(calories: f32) -> String {
    format!("{calories:.0}")
}

fn format_training_effect(effect: f32) -> String {
    format!("{effect:.1}")
}

fn format_load(load: f32) -> String {
    format!("{load:.0}")
}

fn format_temperature(celsius: f32, unit: SpeedUnit) -> String {
    match unit {
        SpeedUnit::Mph => format!("{:.1} °F", celsius_to_fahrenheit(celsius)),
        SpeedUnit::Kmh => format!("{celsius:.1} °C"),
    }
}

fn format_respiration(rate: f32) -> String {
    format!("{rate:.0} brpm")
}

fn celsius_to_fahrenheit(celsius: f32) -> f32 {
    celsius * 9.0 / 5.0 + 32.0
}

fn capitalize(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::activity::load_sample_ride;

    #[test]
    fn sample_ride_stats_sections_include_core_metrics() {
        let ride = load_sample_ride().expect("sample ride");
        let sections = stats_sections(&ride.stats, SpeedUnit::Mph);
        let titles: Vec<_> = sections.iter().map(|section| section.title).collect();
        assert!(titles.contains(&"Distance & Time"));
        assert!(titles.contains(&"Power"));
        assert!(sections.iter().any(|section| section.stats.iter().any(|stat| stat.title == "Distance")));
        assert!(ride.stats.avg_power.is_some());
        assert!(ride.stats.training_stress_score.is_some());
    }
}
