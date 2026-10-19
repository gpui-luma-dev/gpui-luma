//! Look-neutral inspector rows shared with Studio; no separate metadata schema.
#[derive(Clone, Debug)]
pub struct InspectPropertyRow {
    pub label: String,
    pub value: String,
    pub source: String,
    pub detail: Option<String>,
}

impl InspectPropertyRow {
    pub fn new(label: impl Into<String>, value: impl Into<String>, source: impl Into<String>) -> Self {
        Self { label: label.into(), value: value.into(), source: source.into(), detail: None }
    }
}

use gpui::{AnyElement, Div, Hsla, div, prelude::*, px, relative};
use gpui_luma::theme::LumaTextStyle;

/// Presentation inputs supplied by the rendering look, independent of resolver metadata.
pub struct InspectorRowStyle {
    pub border: Hsla,
    pub muted_text: Hsla,
    pub caption: LumaTextStyle,
    pub mono_font: String,
    pub wrap_source: bool,
}

/// Shared Studio property-row layout. Look adapters supply colors and typography.
pub fn render_inspector_row(
    style: InspectorRowStyle,
    label: AnyElement,
    value: AnyElement,
    source: String,
    value_width: f32,
    show_separator: bool,
) -> Div {
    div()
        .min_h(px(34.0))
        .min_w(px(0.0))
        .flex()
        .flex_wrap()
        .items_start()
        .gap(px(4.0))
        .when(show_separator, |row| row.border_t_1().border_color(style.border))
        .px(px(9.0))
        .py(px(6.0))
        .child(div().w(relative(0.28)).min_w(px(180.0)).flex_shrink_0().mr(px(6.0)).child(label))
        .child(
            div()
                .w(px(value_width))
                .flex_shrink_0()
                .mr(px(6.0))
                .font_family(style.mono_font.clone())
                .text_size(px(style.caption.size))
                .line_height(px(style.caption.line_height))
                .text_color(style.muted_text)
                .child(value),
        )
        .child(
            div()
                .flex_none()
                .whitespace_nowrap()
                .when(style.wrap_source, |source| source.w_full().whitespace_normal())
                .font_family(style.mono_font)
                .text_size(px(style.caption.size))
                .line_height(px(style.caption.line_height))
                .text_color(style.muted_text)
                .child(source),
        )
}
