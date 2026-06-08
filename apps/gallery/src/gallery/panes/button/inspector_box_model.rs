use gpui::{Div, FontWeight, Hsla, IntoElement, SharedString, div, hsla, px, prelude::*};
use gpui_luma::theme::LumaTextStyle;
use gpui_luma_look_shadcn::{ButtonInspectMetrics, ShadcnLook};

#[derive(Clone, Copy, Debug)]
pub(in crate::gallery) struct BoxModelLayerColors {
    pub margin: Hsla,
    pub border: Hsla,
    pub padding: Hsla,
    pub content: Hsla,
    pub stroke: Hsla,
    pub highlight: Hsla,
}

impl BoxModelLayerColors {
    /// DevTools layer order mapped to shadcn chart tokens from the loaded theme.
    pub fn from_look(look: &ShadcnLook) -> Self {
        Self {
            margin: chart_fill(look, "chart-5", fallback_margin()),
            border: chart_fill(look, "chart-4", fallback_border()),
            padding: chart_fill(look, "chart-2", fallback_padding()),
            content: chart_fill(look, "chart-1", fallback_content()),
            stroke: look
                .token_color("foreground")
                .map(|color| Hsla { a: 0.45, ..color })
                .unwrap_or_else(|_| fallback_stroke()),
            highlight: look.token_color("chart-3").unwrap_or_else(|_| fallback_highlight()),
        }
    }
}

fn chart_fill(look: &ShadcnLook, token: &str, fallback: Hsla) -> Hsla {
    look.token_color(token).map(|color| Hsla { a: 0.42, ..color }).unwrap_or(fallback)
}

fn fallback_margin() -> Hsla {
    hsla(30.0 / 360.0, 0.45, 0.55, 0.35)
}

fn fallback_border() -> Hsla {
    hsla(45.0 / 360.0, 0.55, 0.65, 0.45)
}

fn fallback_padding() -> Hsla {
    hsla(110.0 / 360.0, 0.35, 0.55, 0.45)
}

fn fallback_content() -> Hsla {
    hsla(210.0 / 360.0, 0.45, 0.55, 0.55)
}

fn fallback_stroke() -> Hsla {
    hsla(0.0, 0.0, 0.15, 0.55)
}

fn fallback_highlight() -> Hsla {
    hsla(210.0 / 360.0, 0.85, 0.55, 1.0)
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::gallery) enum MetricFieldHighlight {
    None,
    Height,
    PaddingX,
    PaddingY,
    Gap,
    Radius,
    BorderWidth,
    FocusRingWidth,
    FocusRingOffset,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::gallery) struct InspectBoxModelSnapshot {
    pub height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub border_width: f32,
    pub gap: f32,
    pub radius: f32,
    pub focus_ring_width: f32,
    pub focus_ring_offset: f32,
}

impl InspectBoxModelSnapshot {
    pub fn from_metrics(metrics: &ButtonInspectMetrics) -> Self {
        Self {
            height: metrics.height.value_px,
            padding_x: metrics.padding_x.value_px,
            padding_y: metrics.padding_y.value_px,
            border_width: metrics.border_width.value_px,
            gap: metrics.gap.value_px,
            radius: metrics.radius.value_px,
            focus_ring_width: metrics.focus_ring_width.value_px,
            focus_ring_offset: metrics.focus_ring_offset.value_px,
        }
    }
}

#[allow(dead_code)]
pub(in crate::gallery) fn metric_field_highlight(name: &str) -> MetricFieldHighlight {
    match name {
        "height" => MetricFieldHighlight::Height,
        "padding x" => MetricFieldHighlight::PaddingX,
        "padding y" => MetricFieldHighlight::PaddingY,
        "gap" => MetricFieldHighlight::Gap,
        "radius" => MetricFieldHighlight::Radius,
        "border width" => MetricFieldHighlight::BorderWidth,
        "focus ring width" => MetricFieldHighlight::FocusRingWidth,
        "focus ring offset" => MetricFieldHighlight::FocusRingOffset,
        _ => MetricFieldHighlight::Height,
    }
}

mod layout {
    pub(super) const EDGE_BAND: f32 = 22.0;
    pub(super) const EDGE_SIDE: f32 = 28.0;
    pub(super) const CONTENT_HEIGHT: f32 = 36.0;
    pub(super) const LAYER_LABEL_SIZE: f32 = 10.0;
    pub(super) const EDGE_VALUE_SIZE: f32 = 10.0;
    pub(super) const CONTENT_LABEL_SIZE: f32 = 11.0;
}

pub(in crate::gallery) fn render_box_model_diagram(
    model: &InspectBoxModelSnapshot,
    highlight: MetricFieldHighlight,
    colors: BoxModelLayerColors,
    label_color: Hsla,
    mono: &LumaTextStyle,
    mono_font: SharedString,
) -> impl IntoElement {
    let body = build_box_model_core(model, highlight, colors, label_color, mono, mono_font);
    let body = if matches!(highlight, MetricFieldHighlight::FocusRingWidth | MetricFieldHighlight::FocusRingOffset) {
        div()
            .w_full()
            .border_1()
            .border_dashed()
            .border_color(colors.highlight)
            .p(px(model.focus_ring_offset.max(4.0)))
            .child(body)
            .into_any_element()
    } else {
        body.into_any_element()
    };

    let caption = highlight_caption(model, highlight);
    div()
        .id("button-inspector-box-model")
        .w_full()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .when_some(caption, |stack, caption| {
            stack.gap(px(6.0)).child(
                div()
                    .w_full()
                    .text_size(px(mono.size))
                    .line_height(px(mono.line_height))
                    .text_color(label_color)
                    .child(caption),
            )
        })
        .child(body)
}

fn highlight_caption(model: &InspectBoxModelSnapshot, highlight: MetricFieldHighlight) -> Option<String> {
    match highlight {
        MetricFieldHighlight::None => None,
        MetricFieldHighlight::Height => Some(format!("height · {}", format_metric_value(model.height))),
        MetricFieldHighlight::Gap => Some(format!("content gap · {}", format_metric_value(model.gap))),
        MetricFieldHighlight::Radius => Some(format!("corner radius · {}", format_metric_value(model.radius))),
        MetricFieldHighlight::FocusRingWidth => {
            Some(format!("focus ring · width {}", format_metric_value(model.focus_ring_width)))
        }
        MetricFieldHighlight::FocusRingOffset => Some(format!(
            "focus ring · offset {} · width {}",
            format_metric_value(model.focus_ring_offset),
            format_metric_value(model.focus_ring_width)
        )),
        _ => None,
    }
}

fn build_box_model_core(
    model: &InspectBoxModelSnapshot,
    highlight: MetricFieldHighlight,
    colors: BoxModelLayerColors,
    label_color: Hsla,
    mono: &LumaTextStyle,
    mono_font: SharedString,
) -> Div {
    let content_label = "\"content\"".to_string();
    let margin = px_value(0.0);
    let border = px_value(model.border_width);
    let padding_x = px_value(model.padding_x);
    let padding_y = px_value(model.padding_y);

    let content = content_box(content_label, colors.content, colors, label_color, mono_font.clone());

    let padding = box_layer(
        "padding",
        EdgeValues::symmetric_vertical(&padding_y, &padding_x),
        layer_style(
            colors.padding,
            colors,
            true,
            matches!(highlight, MetricFieldHighlight::PaddingX | MetricFieldHighlight::PaddingY),
        ),
        Layer::Padding,
        highlight,
        colors,
        label_color,
        mono,
        mono_font.clone(),
        content,
    );

    let border = box_layer(
        "border",
        EdgeValues::all(&border),
        layer_style(colors.border, colors, false, matches!(highlight, MetricFieldHighlight::BorderWidth)),
        Layer::Border,
        highlight,
        colors,
        label_color,
        mono,
        mono_font.clone(),
        padding,
    );

    box_layer(
        "margin",
        EdgeValues::all(&margin),
        layer_style(colors.margin, colors, true, matches!(highlight, MetricFieldHighlight::Height)),
        Layer::Margin,
        highlight,
        colors,
        label_color,
        mono,
        mono_font,
        border,
    )
}

struct EdgeValues {
    top: SharedString,
    right: SharedString,
    bottom: SharedString,
    left: SharedString,
}

impl EdgeValues {
    fn all(value: &SharedString) -> Self {
        Self { top: value.clone(), right: value.clone(), bottom: value.clone(), left: value.clone() }
    }

    fn symmetric_vertical(vertical: &SharedString, horizontal: &SharedString) -> Self {
        Self { top: vertical.clone(), right: horizontal.clone(), bottom: vertical.clone(), left: horizontal.clone() }
    }
}

struct LayerStyle {
    fill: Hsla,
    stroke: Hsla,
    dashed: bool,
}

fn layer_style(fill: Hsla, colors: BoxModelLayerColors, dashed: bool, highlighted: bool) -> LayerStyle {
    LayerStyle { fill, stroke: if highlighted { colors.highlight } else { colors.stroke }, dashed }
}

#[derive(Clone, Copy)]
enum Layer {
    Margin,
    Border,
    Padding,
}

#[derive(Clone, Copy)]
enum Edge {
    Top,
    Right,
    Bottom,
    Left,
}

fn box_layer(
    label: &'static str,
    edges: EdgeValues,
    style: LayerStyle,
    layer: Layer,
    highlight: MetricFieldHighlight,
    colors: BoxModelLayerColors,
    label_color: Hsla,
    mono: &LumaTextStyle,
    mono_font: SharedString,
    child: impl IntoElement,
) -> Div {
    let mut shell =
        div().w_full().min_w(px(0.0)).flex().flex_col().bg(style.fill).border_1().border_color(style.stroke);
    if style.dashed {
        shell = shell.border_dashed();
    }

    shell
        .child(
            div()
                .relative()
                .w_full()
                .h(px(layout::EDGE_BAND))
                .flex()
                .items_center()
                .justify_center()
                .child(edge_value(
                    edges.top,
                    label_color,
                    colors,
                    mono,
                    mono_font.clone(),
                    highlight_edge(highlight, Edge::Top, layer),
                ))
                .child(div().absolute().top(px(2.0)).left(px(4.0)).child(layer_title(label, label_color))),
        )
        .child(
            div()
                .flex()
                .w_full()
                .items_stretch()
                .child(edge_side(
                    edges.left,
                    label_color,
                    colors,
                    mono,
                    mono_font.clone(),
                    highlight_edge(highlight, Edge::Left, layer),
                ))
                .child(div().flex_1().min_w(px(0.0)).w_full().child(child))
                .child(edge_side(
                    edges.right,
                    label_color,
                    colors,
                    mono,
                    mono_font.clone(),
                    highlight_edge(highlight, Edge::Right, layer),
                )),
        )
        .child(div().w_full().h(px(layout::EDGE_BAND)).flex().items_center().justify_center().child(edge_value(
            edges.bottom,
            label_color,
            colors,
            mono,
            mono_font,
            highlight_edge(highlight, Edge::Bottom, layer),
        )))
}

fn content_box(
    label: String,
    fill: Hsla,
    colors: BoxModelLayerColors,
    label_color: Hsla,
    mono_font: SharedString,
) -> Div {
    div()
        .w_full()
        .min_w(px(0.0))
        .h(px(layout::CONTENT_HEIGHT))
        .flex()
        .items_center()
        .justify_center()
        .bg(fill)
        .border_1()
        .border_color(colors.stroke)
        .child(
            div()
                .font_family(mono_font)
                .text_size(px(layout::CONTENT_LABEL_SIZE))
                .line_height(px(14.0))
                .text_color(label_color)
                .child(label),
        )
}

fn highlight_edge(highlight: MetricFieldHighlight, edge: Edge, layer: Layer) -> bool {
    match (highlight, layer, edge) {
        (MetricFieldHighlight::None, _, _) => false,
        (MetricFieldHighlight::Height, Layer::Margin, _) => true,
        (MetricFieldHighlight::BorderWidth, Layer::Border, _) => true,
        (MetricFieldHighlight::PaddingX, Layer::Padding, Edge::Left | Edge::Right) => true,
        (MetricFieldHighlight::PaddingY, Layer::Padding, Edge::Top | Edge::Bottom) => true,
        _ => false,
    }
}

fn layer_title(label: &'static str, color: Hsla) -> Div {
    div()
        .text_size(px(layout::LAYER_LABEL_SIZE))
        .line_height(px(12.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(color)
        .child(label)
}

fn edge_side(
    value: SharedString,
    color: Hsla,
    colors: BoxModelLayerColors,
    mono: &LumaTextStyle,
    mono_font: SharedString,
    highlighted: bool,
) -> Div {
    div().w(px(layout::EDGE_SIDE)).flex().items_center().justify_center().child(edge_value(
        value,
        color,
        colors,
        mono,
        mono_font,
        highlighted,
    ))
}

fn edge_value(
    value: SharedString,
    color: Hsla,
    colors: BoxModelLayerColors,
    mono: &LumaTextStyle,
    mono_font: SharedString,
    highlighted: bool,
) -> Div {
    div()
        .font_family(mono_font)
        .text_size(px(layout::EDGE_VALUE_SIZE))
        .line_height(px(12.0))
        .font_weight(if highlighted { FontWeight::SEMIBOLD } else { mono.weight })
        .text_color(if highlighted { colors.highlight } else { color })
        .child(value)
}

fn px_value(value: f32) -> SharedString {
    format_metric_value(value).into()
}

fn format_metric_value(value: f32) -> String {
    if (value - value.round()).abs() < f32::EPSILON {
        format!("{}", value.round() as i32)
    } else {
        format!("{value:.1}")
    }
}
