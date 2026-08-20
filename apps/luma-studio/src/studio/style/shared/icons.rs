use gpui::{AnyElement, IntoElement, div, prelude::*, px};
use lucide_svg_static::Icon as LucideIcon;

pub(crate) fn render_lucide_icon(icon: LucideIcon, size: f32) -> AnyElement {
    div()
        .text_size(px(size))
        .line_height(px(size))
        .child(gpui_luma::controls::icon::lucide_glyph(icon))
        .into_any_element()
}
