use std::sync::Arc;

use gpui::{AnyElement, App, FontWeight, IntoElement, SharedString, Window, div, prelude::*, px};
use gpui_luma::column_emphasis;
use gpui_luma::column_numeric;
use gpui_luma::controls::list_view::{ListSelectionMode, ScrollingListView};
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnLookControlExt, ShadcnTextSize};

use super::activity::{LapSummary, RideActivity};
use super::ride_summary::{format_duration, format_elevation_for_unit, format_heart_rate, format_power, format_speed};
use super::units::SpeedUnit;

#[derive(Clone)]
pub struct LapRow {
    pub lap_label: SharedString,
    pub elapsed: SharedString,
    pub distance: SharedString,
    pub avg_speed: SharedString,
    pub max_speed: SharedString,
    pub avg_hr: SharedString,
    pub max_hr: SharedString,
    pub avg_power: SharedString,
    pub max_power: SharedString,
    pub ascent: SharedString,
    pub np: SharedString,
}

pub fn spawn_laps_list_view(
    look: &Arc<ShadcnLook>,
    laps: &[LapSummary],
    speed_unit: SpeedUnit,
    cx: &mut impl gpui::AppContext,
) -> ScrollingListView<LapRow> {
    let rows = build_lap_rows(laps, speed_unit);
    let visible_rows = rows.len().clamp(1, 16);

    look.list_view("graph-viz-laps-list")
        .items(rows)
        .selection_mode(ListSelectionMode::Single)
        .selected_index(0)
        .active_index(0)
        .visible_rows(visible_rows)
        .scroll_snap(true)
        .row_label(|row| row.lap_label.clone())
        .grid_view(vec![
            column_emphasis!("Lap", width = 52 => |row: &LapRow| row.lap_label.clone()),
            column_numeric!("Time", width = 72 => |row: &LapRow| row.elapsed.clone()),
            column_numeric!("Distance", width = 88 => |row: &LapRow| row.distance.clone()),
            column_numeric!("Avg Speed", width = 88 => |row: &LapRow| row.avg_speed.clone()),
            column_numeric!("Max Speed", width = 88 => |row: &LapRow| row.max_speed.clone()),
            column_numeric!("Avg HR", width = 72 => |row: &LapRow| row.avg_hr.clone()),
            column_numeric!("Max HR", width = 72 => |row: &LapRow| row.max_hr.clone()),
            column_numeric!("Avg Power", width = 88 => |row: &LapRow| row.avg_power.clone()),
            column_numeric!("Max Power", width = 88 => |row: &LapRow| row.max_power.clone()),
            column_numeric!("NP", width = 72 => |row: &LapRow| row.np.clone()),
            column_numeric!("Ascent", width = 72 => |row: &LapRow| row.ascent.clone()),
        ])
        .spawn(cx)
}

pub fn refresh_laps_list_view(
    list_view: &ScrollingListView<LapRow>,
    laps: &[LapSummary],
    speed_unit: SpeedUnit,
    cx: &mut impl gpui::AppContext,
) {
    list_view.update(cx, |list, cx| {
        list.set_items(build_lap_rows(laps, speed_unit), cx);
    });
}

pub fn render_laps_tab(
    ride: &RideActivity,
    list_view: &ScrollingListView<LapRow>,
    look: &Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let label_style = look.typography_scale(ShadcnTextSize::Sm);
    let value_style = look.typography_scale(ShadcnTextSize::Xs);
    let lap_count = ride.laps.len();
    let trigger_summary = lap_trigger_summary(&ride.laps);

    look.card("graph-viz-laps-card")
        .title("Laps")
        .description(format!("{lap_count} laps from FIT"))
        .elevated(false)
        .child_render({
            let list_view = list_view.clone();
            move |_, _| {
                div()
                    .id("graph-viz-laps-tab-body")
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap(px(12.0))
                    .child(
                        div()
                            .typography_style(value_style)
                            .text_color(chrome.muted_text)
                            .child("Per-lap summaries from the activity file. Scroll for additional columns."),
                    )
                    .child(
                        div()
                            .typography_style(label_style)
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(chrome.title_text)
                            .child(trigger_summary.clone()),
                    )
                    .child(div().w_full().min_h(px(320.0)).child(list_view.clone()))
                    .into_any_element()
            }
        })
        .render(window, cx)
        .into_any_element()
}

pub fn build_lap_rows(laps: &[LapSummary], unit: SpeedUnit) -> Vec<LapRow> {
    laps.iter().map(|lap| lap_row(lap, unit)).collect()
}

fn lap_row(lap: &LapSummary, unit: SpeedUnit) -> LapRow {
    LapRow {
        lap_label: SharedString::from(format!("{}", lap.index + 1)),
        elapsed: SharedString::from(format_duration(lap.elapsed_time_seconds)),
        distance: SharedString::from(unit.format_distance(lap.total_distance_meters)),
        avg_speed: SharedString::from(format_speed(lap.avg_speed_mps, unit)),
        max_speed: SharedString::from(
            lap.max_speed_mps.map(|speed| format_speed(speed, unit)).unwrap_or_else(|| "—".to_string()),
        ),
        avg_hr: SharedString::from(lap.avg_heart_rate.map(format_heart_rate).unwrap_or_else(|| "—".to_string())),
        max_hr: SharedString::from(lap.max_heart_rate.map(format_heart_rate).unwrap_or_else(|| "—".to_string())),
        avg_power: SharedString::from(lap.avg_power.map(format_power).unwrap_or_else(|| "—".to_string())),
        max_power: SharedString::from(lap.max_power.map(format_power).unwrap_or_else(|| "—".to_string())),
        ascent: SharedString::from(format_elevation_for_unit(lap.total_ascent_meters, unit)),
        np: SharedString::from(lap.normalized_power.map(format_power).unwrap_or_else(|| "—".to_string())),
    }
}

fn lap_trigger_summary(laps: &[LapSummary]) -> String {
    let manual = laps.iter().filter(|lap| lap.lap_trigger.as_deref() == Some("manual")).count();
    if manual == 0 {
        format!("{} laps", laps.len())
    } else {
        format!("{} laps · {manual} manual", laps.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::activity::load_sample_ride;

    #[test]
    fn sample_ride_has_laps_with_core_metrics() {
        let ride = load_sample_ride().expect("sample ride");
        assert_eq!(ride.laps.len(), 12);
        assert!(ride.laps[0].total_distance_meters > 0.0);
        assert!(ride.laps[0].avg_power.is_some());

        let rows = build_lap_rows(&ride.laps, SpeedUnit::Mph);
        assert_eq!(rows.len(), 12);
        assert_eq!(rows[0].lap_label.as_ref(), "1");
        assert!(!rows[0].distance.is_empty());
    }
}
