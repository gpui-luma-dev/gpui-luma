use gpui::{Hsla, IntoElement, div, prelude::*, px};

use crate::theme::LumaTypography;

/// A standard, lightweight labeled field text component.
pub fn field_label(text: &'static str, color: Hsla) -> impl IntoElement {
    let style = LumaTypography::default().text.caption;

    div()
        .text_size(px(style.size))
        .line_height(px(style.line_height))
        .font_weight(style.weight)
        .text_color(color)
        .child(text)
}
