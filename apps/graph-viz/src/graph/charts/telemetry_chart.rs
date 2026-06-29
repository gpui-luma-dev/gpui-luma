use std::sync::Arc;

use gpui::{AnyElement, App, IntoElement, SharedString, Window, canvas, div, prelude::*, px};
use gpui_luma_look_shadcn::{ShadcnFont, ShadcnLook, ShadcnTextSize};

use super::super::activity::{TelemetryPoint, cumulative_distance_profile, total_distance_meters};
use super::super::chart_theme::{ChartTheme, GraphVizThemeExt};
use super::super::metrics::{TelemetryMetric, pco_left_plot_samples, pco_right_plot_samples};
use super::super::plot::{AxisLabelStyle, ChartStack, SeriesStyle, build_telemetry_chart_stack, paint_chart_stack};
use super::super::units::{MetricDisplay, PowerUnit, SpeedUnit, XAxisMode};

const CHART_HEIGHT: f32 = 220.0;

#[derive(Clone, Debug, PartialEq)]
pub struct TelemetryChartModel {
    pub metric: TelemetryMetric,
    pub samples: Vec<(f32, f32)>,
    pub duration_seconds: f32,
    pub total_distance_meters: f32,
    pub distance_profile: Vec<(f32, f32)>,
    pub units: MetricDisplay,
    pub series_style: SeriesStyle,
    pub theme: ChartTheme,
    pub axis_style: AxisLabelStyle,
    pub stroke_color: gpui::Hsla,
    pub fill_color: Option<gpui::Hsla>,
    pub overlay_samples: Option<Vec<(f32, f32)>>,
    pub overlay_stroke_color: Option<gpui::Hsla>,
    pub scrub_fraction: f32,
}

impl TelemetryChartModel {
    pub fn from_metric(
        metric: TelemetryMetric,
        points: &[TelemetryPoint],
        duration_seconds: f32,
        units: MetricDisplay,
        series_style: SeriesStyle,
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
        let samples = if metric == TelemetryMetric::RightPlatformCenterOffset {
            pco_right_plot_samples(points)
        } else {
            metric.samples(points)
        };
        let distance_profile = cumulative_distance_profile(points);
        let total_distance_meters = total_distance_meters(&distance_profile);
        let fill_color = match series_style {
            SeriesStyle::AreaFilled => theme.fill_for_metric(metric),
            SeriesStyle::Line | SeriesStyle::Dots => None,
        };
        let overlay_samples = (metric == TelemetryMetric::RightPlatformCenterOffset)
            .then(|| pco_left_plot_samples(points))
            .filter(|samples| samples.len() >= 2);
        let (stroke_color, overlay_stroke_color) = if metric == TelemetryMetric::RightPlatformCenterOffset {
            (theme.pco_stroke, overlay_samples.as_ref().map(|_| theme.left_pco_stroke))
        } else {
            (theme.stroke_for_metric(metric), None)
        };
        Self::new(
            metric,
            samples,
            duration_seconds,
            total_distance_meters,
            distance_profile,
            units,
            series_style,
            theme,
            axis_style,
            stroke_color,
            fill_color,
            overlay_samples,
            overlay_stroke_color,
            scrub_fraction,
        )
    }

    pub fn new(
        metric: TelemetryMetric,
        samples: Vec<(f32, f32)>,
        duration_seconds: f32,
        total_distance_meters: f32,
        distance_profile: Vec<(f32, f32)>,
        units: MetricDisplay,
        series_style: SeriesStyle,
        theme: ChartTheme,
        axis_style: AxisLabelStyle,
        stroke_color: gpui::Hsla,
        fill_color: Option<gpui::Hsla>,
        overlay_samples: Option<Vec<(f32, f32)>>,
        overlay_stroke_color: Option<gpui::Hsla>,
        scrub_fraction: f32,
    ) -> Option<Self> {
        if samples.len() < 2 || duration_seconds <= f32::EPSILON {
            return None;
        }

        Some(Self {
            metric,
            samples,
            duration_seconds,
            total_distance_meters,
            distance_profile,
            units,
            series_style,
            theme,
            axis_style,
            stroke_color,
            fill_color,
            overlay_samples,
            overlay_stroke_color,
            scrub_fraction: scrub_fraction.clamp(0.0, 1.0),
        })
    }

    fn build_stack(&self, canvas_width_px: f32, window: &mut Window) -> Option<ChartStack> {
        build_telemetry_chart_stack(
            self.metric,
            &self.samples,
            self.duration_seconds,
            self.total_distance_meters,
            &self.distance_profile,
            canvas_width_px,
            self.units,
            self.theme.background,
            self.theme.grid_line,
            self.theme.cursor_line,
            self.stroke_color,
            self.fill_color,
            self.series_style,
            self.scrub_fraction,
            self.overlay_samples.as_deref(),
            self.overlay_stroke_color,
            &self.axis_style,
            window,
        )
    }
}

pub fn render_telemetry_chart(
    id: &'static str,
    title: SharedString,
    model: TelemetryChartModel,
    look: &Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let surface_id = chart_surface_id(&model);
    look.card(id)
        .title(title)
        .elevated(false)
        .child_render({
            let model = model.clone();
            move |_, _| {
                div()
                    .id(surface_id.clone())
                    .w_full()
                    .h(px(CHART_HEIGHT))
                    .child(render_chart_canvas(model.clone()))
                    .into_any_element()
            }
        })
        .render(window, cx)
        .into_any_element()
}

fn chart_surface_id(model: &TelemetryChartModel) -> SharedString {
    SharedString::from(format!(
        "telemetry-surface-{}-{:?}-{}-{}-{}-{}",
        model.metric.id(),
        model.series_style,
        speed_unit_key(model.units.speed_unit),
        power_unit_key(model.units.power_unit),
        x_axis_key(model.units.x_axis),
        (model.scrub_fraction * 10_000.0).round() as u32,
    ))
}

fn power_unit_key(unit: PowerUnit) -> &'static str {
    match unit {
        PowerUnit::Watts => "w",
        PowerUnit::WattsPerKg => "wkg",
    }
}

fn x_axis_key(mode: XAxisMode) -> &'static str {
    match mode {
        XAxisMode::Time => "time",
        XAxisMode::Distance => "distance",
    }
}

fn speed_unit_key(unit: SpeedUnit) -> &'static str {
    match unit {
        SpeedUnit::Mph => "mph",
        SpeedUnit::Kmh => "kmh",
    }
}

fn render_chart_canvas(model: TelemetryChartModel) -> impl IntoElement {
    canvas(
        move |bounds, window, _| model.build_stack(bounds.size.width.as_f32(), window),
        move |bounds, stack, window, cx| {
            if let Some(stack) = stack {
                paint_chart_stack(window, cx, bounds, &stack);
            }
        },
    )
    .size_full()
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
    fn chart_surface_id_changes_with_power_unit() {
        let watts_model = sample_power_model(PowerUnit::Watts);
        let wkg_model = sample_power_model(PowerUnit::WattsPerKg);
        assert_ne!(chart_surface_id(&watts_model), chart_surface_id(&wkg_model));
    }

    fn sample_power_model(unit: PowerUnit) -> TelemetryChartModel {
        TelemetryChartModel::new(
            TelemetryMetric::Power,
            vec![(0.0, 100.0), (5.0, 200.0), (10.0, 150.0)],
            10.0,
            100.0,
            vec![(0.0, 0.0), (10.0, 100.0)],
            MetricDisplay {
                speed_unit: SpeedUnit::Mph,
                x_axis: XAxisMode::Time,
                power_unit: unit,
                rider_weight_kg: Some(80.0),
            },
            SeriesStyle::Line,
            sample_theme(),
            sample_axis_style(),
            sample_theme().power_stroke,
            None,
            None,
            None,
            0.0,
        )
        .expect("chart should build")
    }

    #[test]
    fn chart_surface_id_changes_with_speed_unit() {
        let mph_model = sample_speed_model(SpeedUnit::Mph);
        let mut kmh_model = sample_speed_model(SpeedUnit::Kmh);
        assert_ne!(chart_surface_id(&mph_model), chart_surface_id(&kmh_model));

        kmh_model.scrub_fraction = 0.5;
        assert_ne!(chart_surface_id(&mph_model), chart_surface_id(&kmh_model));
    }

    fn sample_speed_model(unit: SpeedUnit) -> TelemetryChartModel {
        TelemetryChartModel::new(
            TelemetryMetric::Speed,
            vec![(0.0, 10.0), (5.0, 30.0), (10.0, 20.0)],
            10.0,
            100.0,
            vec![(0.0, 0.0), (10.0, 100.0)],
            MetricDisplay { speed_unit: unit, x_axis: XAxisMode::Time, ..MetricDisplay::default() },
            SeriesStyle::Line,
            sample_theme(),
            sample_axis_style(),
            sample_theme().speed_stroke,
            None,
            None,
            None,
            0.0,
        )
        .expect("chart should build")
    }

    #[test]
    fn speed_filled_model_requests_area_style() {
        let samples = vec![(0.0, 10.0), (5.0, 30.0), (10.0, 20.0)];
        let model = TelemetryChartModel::new(
            TelemetryMetric::Speed,
            samples,
            10.0,
            100.0,
            vec![(0.0, 0.0), (10.0, 100.0)],
            MetricDisplay::default(),
            SeriesStyle::AreaFilled,
            sample_theme(),
            sample_axis_style(),
            sample_theme().speed_stroke,
            Some(sample_theme().speed_fill),
            None,
            None,
            0.5,
        )
        .expect("chart should build");
        assert_eq!(model.series_style, SeriesStyle::AreaFilled);
        assert!(model.fill_color.is_some());
    }
}
