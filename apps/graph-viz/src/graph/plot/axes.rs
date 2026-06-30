use gpui::{App, FontWeight, Hsla, Pixels, SharedString, TextAlign, TextRun, Window, font, point, px};

use super::{PlotDomain2D, Viewport2D};
use crate::graph::metrics::TelemetryMetric;
use crate::graph::plot::format_x_axis_tick;
use crate::graph::units::MetricDisplay;

const LABEL_GAP: f32 = 8.0;

#[derive(Clone, Debug, PartialEq)]
pub struct AxisLabelStyle {
    pub font_family: SharedString,
    pub font_size: f32,
    pub line_height: f32,
    pub font_weight: FontWeight,
    pub color: Hsla,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum XLabelPlacement {
    Start,
    Middle,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum YAxisLabelSide {
    #[default]
    Left,
    #[allow(dead_code)]
    Right,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AxisLabelsLayer {
    pub x_tick_values: Vec<f32>,
    pub y_tick_values: Vec<f32>,
    pub x_tick_step: f32,
    pub y_tick_step_display: f32,
    pub metric: TelemetryMetric,
    pub units: MetricDisplay,
    pub y_inverted: bool,
    pub y_label_side: YAxisLabelSide,
    pub style: AxisLabelStyle,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RightAxisLabelsLayer {
    pub y_tick_values: Vec<f32>,
    pub y_tick_step_display: f32,
    pub metric: TelemetryMetric,
    pub units: MetricDisplay,
    pub y_inverted: bool,
    pub style: AxisLabelStyle,
}

pub fn measure_label_widths(window: &mut Window, style: &AxisLabelStyle, labels: &[String]) -> Vec<f32> {
    labels.iter().map(|label| shape_label(window, style, label).width().as_f32()).collect()
}

pub fn measure_max_label_width(window: &mut Window, style: &AxisLabelStyle, labels: &[String]) -> f32 {
    measure_label_widths(window, style, labels).into_iter().fold(0.0_f32, f32::max)
}

pub fn paint_axis_labels(
    window: &mut Window,
    cx: &mut App,
    viewport: &Viewport2D,
    x_domain: &PlotDomain2D,
    y_domain: &PlotDomain2D,
    layer: &AxisLabelsLayer,
) {
    match layer.y_label_side {
        YAxisLabelSide::Left => paint_left_y_axis_labels(window, cx, viewport, y_domain, layer),
        YAxisLabelSide::Right => paint_right_y_axis_labels_from_layer(window, cx, viewport, y_domain, layer),
    }
    paint_x_axis_labels(
        window,
        cx,
        viewport,
        x_domain,
        &layer.x_tick_values,
        layer.x_tick_step,
        layer.units,
        &layer.style,
    );
}

pub fn paint_right_axis_labels(
    window: &mut Window,
    cx: &mut App,
    viewport: &Viewport2D,
    y_domain: &PlotDomain2D,
    layer: &RightAxisLabelsLayer,
) {
    let y_anchor_x = viewport.bounds.right() + px(LABEL_GAP);

    for &value in &layer.y_tick_values {
        let display = layer.metric.to_display(value, layer.units);
        let label = layer.metric.format_display_tick(display, layer.y_tick_step_display);
        let fraction = y_fraction(y_domain, value, layer.y_inverted);
        let tick_y = viewport.uv_to_pixels(gpui::point(0.0, fraction)).y;
        paint_y_tick_label_right(
            window,
            cx,
            &label,
            y_anchor_x,
            tick_y - px(layer.style.line_height / 2.0),
            &layer.style,
        );
    }
}

fn paint_right_y_axis_labels_from_layer(
    window: &mut Window,
    cx: &mut App,
    viewport: &Viewport2D,
    y_domain: &PlotDomain2D,
    layer: &AxisLabelsLayer,
) {
    let y_anchor_x = viewport.bounds.right() + px(LABEL_GAP);

    for &value in &layer.y_tick_values {
        let display = layer.metric.to_display(value, layer.units);
        let label = layer.metric.format_display_tick(display, layer.y_tick_step_display);
        let fraction = y_fraction(y_domain, value, layer.y_inverted);
        let tick_y = viewport.uv_to_pixels(gpui::point(0.0, fraction)).y;
        paint_y_tick_label_right(
            window,
            cx,
            &label,
            y_anchor_x,
            tick_y - px(layer.style.line_height / 2.0),
            &layer.style,
        );
    }
}

fn paint_left_y_axis_labels(
    window: &mut Window,
    cx: &mut App,
    viewport: &Viewport2D,
    y_domain: &PlotDomain2D,
    layer: &AxisLabelsLayer,
) {
    let y_anchor_x = viewport.bounds.left() - px(LABEL_GAP);

    for &value in &layer.y_tick_values {
        let display = layer.metric.to_display(value, layer.units);
        let label = layer.metric.format_display_tick(display, layer.y_tick_step_display);
        let fraction = y_fraction(y_domain, value, layer.y_inverted);
        let tick_y = viewport.uv_to_pixels(gpui::point(0.0, fraction)).y;
        paint_y_tick_label(window, cx, &label, y_anchor_x, tick_y - px(layer.style.line_height / 2.0), &layer.style);
    }
}

fn y_fraction(y_domain: &PlotDomain2D, y: f32, invert: bool) -> f32 {
    let fraction = y_domain.y_to_fraction(y);
    if invert { 1.0 - fraction } else { fraction }
}

fn paint_x_axis_labels(
    window: &mut Window,
    cx: &mut App,
    viewport: &Viewport2D,
    x_domain: &PlotDomain2D,
    x_tick_values: &[f32],
    x_tick_step: f32,
    units: MetricDisplay,
    style: &AxisLabelStyle,
) {
    let x_label_y = viewport.bounds.bottom() + px(4.0);
    let last_index = x_tick_values.len().saturating_sub(1);
    for (index, &time) in x_tick_values.iter().enumerate() {
        let label = format_x_axis_tick(time, x_tick_step, units);
        let fraction = x_domain.x_to_fraction(time);
        let tick_x = viewport.uv_to_pixels(gpui::point(fraction, 0.0)).x;
        let placement = if index == 0 {
            XLabelPlacement::Start
        } else if index == last_index {
            XLabelPlacement::End
        } else {
            XLabelPlacement::Middle
        };
        paint_x_tick_label(window, cx, label, tick_x, x_label_y, placement, style);
    }
}

fn paint_y_tick_label(
    window: &mut Window,
    cx: &mut App,
    text: &str,
    anchor_x: Pixels,
    y: Pixels,
    style: &AxisLabelStyle,
) {
    let line = shape_label(window, style, text);
    let width = line.width();
    let origin = point(anchor_x - width, y);
    let _ = line.paint(origin, px(style.line_height), TextAlign::Left, None, window, cx);
}

fn paint_y_tick_label_right(
    window: &mut Window,
    cx: &mut App,
    text: &str,
    anchor_x: Pixels,
    y: Pixels,
    style: &AxisLabelStyle,
) {
    let line = shape_label(window, style, text);
    let origin = point(anchor_x, y);
    let _ = line.paint(origin, px(style.line_height), TextAlign::Left, None, window, cx);
}

fn paint_x_tick_label(
    window: &mut Window,
    cx: &mut App,
    text: String,
    tick_x: Pixels,
    y: Pixels,
    placement: XLabelPlacement,
    style: &AxisLabelStyle,
) {
    let line = shape_label(window, style, &text);
    let width = line.width();
    let origin = match placement {
        XLabelPlacement::Start => point(tick_x, y),
        XLabelPlacement::Middle => point(tick_x - width / 2.0, y),
        XLabelPlacement::End => point(tick_x - width, y),
    };
    let _ = line.paint(origin, px(style.line_height), TextAlign::Left, None, window, cx);
}

fn shape_label(window: &mut Window, style: &AxisLabelStyle, text: &str) -> gpui::ShapedLine {
    let label = SharedString::from(text.to_string());
    let run = TextRun {
        len: label.len(),
        font: {
            let mut font = font(style.font_family.clone());
            font.weight = style.font_weight;
            font
        },
        color: style.color,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    window.text_system().shape_line(label, px(style.font_size), &[run], None)
}
