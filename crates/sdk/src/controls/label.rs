use gpui::{FontWeight, Hsla, IntoElement, div, prelude::*, px};

/// A standard, lightweight labeled field text component.
pub fn field_label(text: &'static str, color: Hsla) -> impl IntoElement {
    div()
        .text_size(px(11.0))
        .line_height(px(14.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(color)
        .child(text)
}
