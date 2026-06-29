use std::sync::Arc;

use gpui::{AnyElement, App, IntoElement, SharedString, Window, canvas, div, prelude::*, px};
use gpui_luma_look_shadcn::{ShadcnFont, ShadcnLook, ShadcnTextSize};

use super::super::activity::{TelemetryPoint, cumulative_distance_profile, total_distance_meters};
use super::super::chart_theme::{ChartTheme, GraphVizThemeExt};
use super::super::metrics::TelemetryMetric;
use super::super::plot::{
    AxisLabelStyle, DualAxisOverlayStack, build_speed_hr_overlay_stack, paint_dual_axis_overlay_stack,
};
use super::super::units::{MetricDisplay, SpeedUnit, XAxisMode};

const CHART_HEIGHT: f32 = 220.0;

#[derive(Clone, Debug, PartialEq)]
pub struct SpeedHrOverlayChartModel {
    pub speed_samples: Vec<(f32, f32)>,
    pub hr_samples: Vec<(f32, f32)>,
    pub duration_seconds: f32,
    pub total_distance_meters: f32,
    pub distance_profile: Vec<(f32, f32)>,
    pub units: MetricDisplay,
    pub theme: ChartTheme,
    pub axis_style: AxisLabelStyle,
    pub scrub_fraction: f32,
}

impl SpeedHrOverlayChartModel {
    pub fn from_points(
        points: &[TelemetryPoint],
        duration_seconds: f32,
        units: MetricDisplay,
        look: &ShadcnLook,
        scrub_fraction: f32,
    ) -> Option<Self> {
        let theme = look.resolve_chart_theme();
        let caption = look.typography_scale(ShadcnTextSize::Xs);
        let axis_style = AxisLabelStyle {
            font_family: look.font(ShadcnFont::Sans),
            font_size: caption.size,
            line_height: caption.line_height,
            font_weight: caption.weight,
            color: theme.axis_label,
        };
        let speed_samples = TelemetryMetric::Speed.samples(points);
        let hr_samples = TelemetryMetric::HeartRate.samples(points);
        let distance_profile = cumulative_distance_profile(points);
        let total_distance_meters = total_distance_meters(&distance_profile);
        if speed_samples.len() < 2 || hr_samples.len() < 2 || duration_seconds <= f32::EPSILON {
            return None;
        }

        Some(Self {
            speed_samples,
            hr_samples,
            duration_seconds,
            total_distance_meters,
            distance_profile,
            units,
            theme,
            axis_style,
            scrub_fraction: scrub_fraction.clamp(0.0, 1.0),
        })
    }

    fn build_stack(&self, canvas_width_px: f32, window: &mut Window) -> Option<DualAxisOverlayStack> {
        build_speed_hr_overlay_stack(
            &self.speed_samples,
            &self.hr_samples,
            self.duration_seconds,
            self.total_distance_meters,
            &self.distance_profile,
            canvas_width_px,
            self.units,
            self.theme.background,
            self.theme.grid_line,
            self.theme.cursor_line,
            self.theme.speed_fill,
            self.theme.speed_stroke,
            self.theme.heartrate_stroke,
            self.scrub_fraction,
            &self.axis_style,
            window,
        )
    }
}

pub fn render_speed_hr_overlay_chart(
    title: SharedString,
    model: SpeedHrOverlayChartModel,
    look: &Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let surface_id = overlay_surface_id(&model);
    look.card("graph-viz-speed-hr-overlay-chart")
        .title(title)
        .elevated(false)
        .child_render({
            let model = model.clone();
            move |_, _| {
                div()
                    .id(surface_id.clone())
                    .w_full()
                    .h(px(CHART_HEIGHT))
                    .child(render_overlay_canvas(model.clone()))
                    .into_any_element()
            }
        })
        .render(window, cx)
        .into_any_element()
}

fn overlay_surface_id(model: &SpeedHrOverlayChartModel) -> SharedString {
    SharedString::from(format!(
        "speed-hr-overlay-surface-{}-{}-{}",
        speed_unit_key(model.units.speed_unit),
        x_axis_key(model.units.x_axis),
        (model.scrub_fraction * 10_000.0).round() as u32,
    ))
}

fn speed_unit_key(unit: SpeedUnit) -> &'static str {
    match unit {
        SpeedUnit::Mph => "mph",
        SpeedUnit::Kmh => "kmh",
    }
}

fn x_axis_key(mode: XAxisMode) -> &'static str {
    match mode {
        XAxisMode::Time => "time",
        XAxisMode::Distance => "distance",
    }
}

fn render_overlay_canvas(model: SpeedHrOverlayChartModel) -> impl IntoElement {
    canvas(
        move |bounds, window, _| model.build_stack(bounds.size.width.as_f32(), window),
        move |bounds, stack, window, cx| {
            if let Some(stack) = stack {
                paint_dual_axis_overlay_stack(window, cx, bounds, &stack);
            }
        },
    )
    .size_full()
}

pub fn overlay_chart_title(units: MetricDisplay) -> String {
    format!("Speed + Heart Rate Overlay ({}, {})", units.speed_unit.label(), units.x_axis.label())
}

pub fn overlay_available(points: &[TelemetryPoint]) -> bool {
    TelemetryMetric::Speed.is_available(points) && TelemetryMetric::HeartRate.is_available(points)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{FontWeight, SharedString, hsla};

    fn sample_theme() -> ChartTheme {
        ChartTheme {
            background: hsla(0.0, 0.0, 0.1, 1.0),
            grid_line: hsla(0.0, 0.0, 0.5, 1.0),
            axis_label: hsla(0.0, 0.0, 0.6, 1.0),
            cursor_line: hsla(0.0, 0.0, 0.7, 1.0),
            elevation_stroke: hsla(0.6, 0.5, 0.5, 1.0),
            heartrate_stroke: hsla(0.0, 0.7, 0.5, 1.0),
            respiration_stroke: hsla(0.5, 0.7, 0.5, 1.0),
            pco_stroke: hsla(0.0, 0.0, 0.55, 0.72),
            left_pco_stroke: hsla(0.0, 0.75, 0.5, 0.72),
            power_stroke: hsla(0.2, 0.7, 0.5, 1.0),
            speed_stroke: hsla(0.4, 0.7, 0.5, 1.0),
            speed_fill: hsla(0.4, 0.7, 0.5, 0.4),
        }
    }

    fn sample_axis_style() -> AxisLabelStyle {
        AxisLabelStyle {
            font_family: SharedString::from("sans"),
            font_size: 11.0,
            line_height: 14.0,
            font_weight: FontWeight::NORMAL,
            color: hsla(0.0, 0.0, 0.6, 1.0),
        }
    }

    #[test]
    fn overlay_surface_id_changes_with_speed_unit() {
        let mph_model = sample_overlay_model(SpeedUnit::Mph);
        let kmh_model = sample_overlay_model(SpeedUnit::Kmh);
        assert_ne!(overlay_surface_id(&mph_model), overlay_surface_id(&kmh_model));
    }

    fn sample_overlay_model(unit: SpeedUnit) -> SpeedHrOverlayChartModel {
        SpeedHrOverlayChartModel {
            speed_samples: vec![(0.0, 10.0), (5.0, 30.0), (10.0, 20.0)],
            hr_samples: vec![(0.0, 120.0), (5.0, 150.0), (10.0, 130.0)],
            duration_seconds: 10.0,
            total_distance_meters: 100.0,
            distance_profile: vec![(0.0, 0.0), (10.0, 100.0)],
            units: MetricDisplay { speed_unit: unit, x_axis: XAxisMode::Time, ..MetricDisplay::default() },
            theme: sample_theme(),
            axis_style: sample_axis_style(),
            scrub_fraction: 0.0,
        }
    }
}
