use std::sync::Arc;

use gpui::{
    AnyElement, App, Bounds, Corners, FontWeight, IntoElement, SharedString, TextAlign, TextRun, Window, canvas, div,
    fill, font, point, prelude::*, px, size,
};
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnFont, ShadcnLook, ShadcnTextSize};

use super::super::activity::{ActivityPowerCurve, PowerCurvePoint};
use super::super::chart_theme::{ChartTheme, GraphVizThemeExt};
use super::super::plot::{
    AreaFillLayer, AxisLabelStyle, ChartMargins, GridLinesLayer, PlotDomain2D, Viewport2D, paint_area_fill,
    paint_background, paint_grid_lines,
};

const CHART_HEIGHT: f32 = 320.0;
const Y_AXIS_MAX_WATTS: f32 = 800.0;
const Y_TICK_STEP_WATTS: f32 = 200.0;

#[derive(Clone, Debug, PartialEq)]
pub struct PowerCurveChartModel {
    curve: ActivityPowerCurve,
    theme: ChartTheme,
    axis_style: AxisLabelStyle,
    activity_fill: gpui::Hsla,
    record_fill: gpui::Hsla,
}

impl PowerCurveChartModel {
    pub fn from_curve(curve: ActivityPowerCurve, look: &ShadcnLook) -> Option<Self> {
        if curve.activity.is_empty() {
            return None;
        }

        let theme = look.resolve_chart_theme();
        let caption = look.typography_scale(ShadcnTextSize::Xs);
        let axis_style = AxisLabelStyle {
            font_family: look.font(ShadcnFont::Sans),
            font_size: caption.size,
            line_height: caption.line_height,
            font_weight: caption.weight,
            color: theme.axis_label,
        };
        let activity_fill = look.token_color("chart-1").unwrap_or(theme.power_stroke).opacity(0.55);
        let record_fill = look.token_color("muted").unwrap_or(theme.grid_line).opacity(0.45);

        Some(Self { curve, theme, axis_style, activity_fill, record_fill })
    }

    fn build_stack(&self, _canvas_width_px: f32) -> Option<PowerCurveChartStack> {
        let last_index = self.curve.activity.last()?.index as f32;
        let x_domain = PlotDomain2D::new(0.0..=last_index.max(1.0), 0.0..=1.0);
        let y_domain = PlotDomain2D::new(0.0..=1.0, 0.0..=Y_AXIS_MAX_WATTS);
        let y_ticks: Vec<f32> = (0..=4).map(|step| step as f32 * Y_TICK_STEP_WATTS).collect();

        Some(PowerCurveChartStack {
            background_color: self.theme.background,
            x_domain,
            y_domain,
            margins: ChartMargins { left: 44.0, right: 16.0, top: 12.0, bottom: 36.0 },
            grid: GridLinesLayer {
                x_tick_values: Vec::new(),
                y_tick_values: y_ticks,
                grid_color: self.theme.grid_line,
            },
            activity_fill: area_layer(&self.curve.activity, self.activity_fill),
            record_fill: self.curve.record.is_empty().then(|| area_layer(&self.curve.record, self.record_fill)),
            activity_line: line_points(&self.curve.activity),
            activity_points: self.curve.activity.clone(),
            axis_style: self.axis_style.clone(),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
struct PowerCurveChartStack {
    background_color: gpui::Hsla,
    x_domain: PlotDomain2D,
    y_domain: PlotDomain2D,
    margins: ChartMargins,
    grid: GridLinesLayer,
    activity_fill: AreaFillLayer,
    record_fill: Option<AreaFillLayer>,
    activity_line: Vec<(f32, f32)>,
    activity_points: Vec<PowerCurvePoint>,
    axis_style: AxisLabelStyle,
}

fn area_layer(points: &[PowerCurvePoint], fill_color: gpui::Hsla) -> AreaFillLayer {
    AreaFillLayer {
        fill_color,
        points: points.iter().map(|point| (point.index as f32, point.max_mean_power_watts)).collect(),
    }
}

fn line_points(points: &[PowerCurvePoint]) -> Vec<(f32, f32)> {
    points.iter().map(|point| (point.index as f32, point.max_mean_power_watts)).collect()
}

pub fn render_power_curve_tab(
    curve: &ActivityPowerCurve,
    look: &Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let legend_style = look.typography_scale(ShadcnTextSize::Xs);

    let Some(model) = PowerCurveChartModel::from_curve(curve.clone(), look) else {
        return look
            .card("graph-viz-power-curve-empty")
            .title("Power Curve")
            .description("No power data found for this activity.")
            .elevated(false)
            .render(window, cx)
            .into_any_element();
    };

    let activity_color = look.token_color("chart-1").unwrap_or(look.resolve_chart_theme().power_stroke);

    div()
        .id("graph-viz-power-curve-tab")
        .w_full()
        .flex()
        .flex_col()
        .gap(px(16.0))
        .child(
            div()
                .id("graph-viz-power-curve-chart-shell")
                .w_full()
                .h(px(CHART_HEIGHT))
                .child(render_chart_canvas(model)),
        )
        .child(render_legend(legend_style, chrome.muted_text, chrome.title_text, activity_color))
        .into_any_element()
}

fn render_chart_canvas(model: PowerCurveChartModel) -> impl IntoElement {
    canvas(
        move |bounds, _window, _| model.build_stack(bounds.size.width.as_f32()),
        move |bounds, stack, window, cx| {
            if let Some(stack) = stack {
                paint_power_curve_stack(window, cx, bounds, &stack);
            }
        },
    )
    .size_full()
}

fn paint_power_curve_stack(
    window: &mut Window,
    cx: &mut App,
    bounds: Bounds<gpui::Pixels>,
    stack: &PowerCurveChartStack,
) {
    paint_background(window, bounds, stack.background_color);

    let viewport = Viewport2D::with_margins(bounds, stack.margins);
    paint_grid_lines(window, &stack.x_domain, &stack.y_domain, &viewport, &stack.grid, false, None);

    if let Some(record_fill) = &stack.record_fill {
        paint_area_fill(window, &stack.x_domain, &stack.y_domain, &viewport, record_fill, false);
    }
    paint_area_fill(window, &stack.x_domain, &stack.y_domain, &viewport, &stack.activity_fill, false);
    paint_duration_axis(window, cx, &viewport, &stack.x_domain, &stack.activity_points, &stack.axis_style);
    paint_y_axis_labels(window, cx, &viewport, &stack.y_domain, &stack.grid.y_tick_values, &stack.axis_style);
    paint_x_markers(window, &viewport, &stack.x_domain, &stack.activity_line, stack.grid.grid_color);
}

fn paint_y_axis_labels(
    window: &mut Window,
    cx: &mut App,
    viewport: &Viewport2D,
    y_domain: &PlotDomain2D,
    ticks: &[f32],
    style: &AxisLabelStyle,
) {
    let anchor_x = viewport.bounds.left() - px(8.0);
    for &value in ticks {
        let fraction = y_domain.y_to_fraction(value);
        let tick_y = viewport.uv_to_pixels(gpui::point(0.0, fraction)).y;
        let label = format!("{} W", value as u32);
        paint_text_label(window, cx, &label, anchor_x, tick_y - px(style.line_height / 2.0), style, TextAlign::Right);
    }
}

fn paint_duration_axis(
    window: &mut Window,
    cx: &mut App,
    viewport: &Viewport2D,
    x_domain: &PlotDomain2D,
    points: &[PowerCurvePoint],
    style: &AxisLabelStyle,
) {
    let y = viewport.bounds.bottom() + px(6.0);
    let last = points.len().saturating_sub(1);
    for (ordinal, point) in points.iter().enumerate() {
        let fraction = x_domain.x_to_fraction(point.index as f32);
        let tick_x = viewport.uv_to_pixels(gpui::point(fraction, 0.0)).x;
        let align = if ordinal == 0 {
            TextAlign::Left
        } else if ordinal == last {
            TextAlign::Right
        } else {
            TextAlign::Center
        };
        paint_text_label(window, cx, point.duration_label, tick_x, y, style, align);
    }
}

fn paint_x_markers(
    window: &mut Window,
    viewport: &Viewport2D,
    x_domain: &PlotDomain2D,
    points: &[(f32, f32)],
    color: gpui::Hsla,
) {
    let radius = px(2.0);
    let baseline_y = viewport.bounds.bottom();
    for &(x, _) in points {
        let tick_x = viewport.uv_to_pixels(gpui::point(x_domain.x_to_fraction(x), 0.0)).x;
        let marker =
            Bounds { origin: point(tick_x - radius, baseline_y - radius), size: size(radius * 2.0, radius * 2.0) };
        window.paint_quad(fill(marker, color).corner_radii(Corners::all(radius)));
    }
}

fn paint_text_label(
    window: &mut Window,
    cx: &mut App,
    text: &str,
    anchor_x: gpui::Pixels,
    y: gpui::Pixels,
    style: &AxisLabelStyle,
    align: TextAlign,
) {
    let label = SharedString::from(text.to_string());
    let run = TextRun {
        len: label.len(),
        font: {
            let mut label_font = font(style.font_family.clone());
            label_font.weight = style.font_weight;
            label_font
        },
        color: style.color,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let line = window.text_system().shape_line(label.clone(), px(style.font_size), &[run], None);
    let width = line.width();
    let origin = match align {
        TextAlign::Left => point(anchor_x, y),
        TextAlign::Center => point(anchor_x - width / 2.0, y),
        TextAlign::Right => point(anchor_x - width, y),
        _ => point(anchor_x, y),
    };
    let _ = line.paint(origin, px(style.line_height), align, None, window, cx);
}

fn render_legend(
    text_style: gpui_luma::theme::LumaTextStyle,
    muted_color: gpui::Hsla,
    title_color: gpui::Hsla,
    activity_color: gpui::Hsla,
) -> AnyElement {
    let record_color = muted_color.opacity(0.45);

    div()
        .id("graph-viz-power-curve-legend")
        .w_full()
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .gap(px(20.0))
        .child(legend_item("Record", record_color, text_style, title_color))
        .child(legend_item("Activity", activity_color, text_style, title_color))
        .into_any_element()
}

fn legend_item(
    label: &'static str,
    swatch: gpui::Hsla,
    text_style: gpui_luma::theme::LumaTextStyle,
    text_color: gpui::Hsla,
) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .child(div().size(px(12.0)).rounded(px(2.0)).bg(swatch))
        .child(
            div()
                .typography_style(text_style)
                .font_weight(FontWeight::MEDIUM)
                .text_color(text_color)
                .child(label),
        )
        .into_any_element()
}
