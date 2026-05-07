use gpui::{AnyElement, FontWeight, Hsla, div, px, prelude::*};
use lucide_icons::Icon as LucideIcon;

pub const LUCIDE_FONT_FAMILY: &str = "lucide";

pub fn lucide_icon(icon: LucideIcon, color: Hsla, size: f32) -> AnyElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .font_family(LUCIDE_FONT_FAMILY)
        .font_weight(FontWeight::NORMAL)
        .text_size(px(size))
        .line_height(px(size))
        .text_color(color)
        .child(char::from(icon).to_string())
        .into_any_element()
}
