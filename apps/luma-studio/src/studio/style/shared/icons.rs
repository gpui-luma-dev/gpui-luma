use gpui::{AnyElement, IntoElement, div, prelude::*, px};
use lucide_icons::Icon as LucideIcon;

pub(crate) fn render_lucide_icon(icon: LucideIcon, size: f32) -> AnyElement {
    div()
        .font_family("lucide")
        .text_size(px(size))
        .line_height(px(size))
        .child(char::from(icon).to_string())
        .into_any_element()
}
