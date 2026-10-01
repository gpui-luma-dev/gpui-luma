//! Soft informational callout built from the current Radix accent scale.

use gpui::{Div, SharedString, div, prelude::*, px, svg};
use gpui_luma::controls::button::ControlIcon;

use crate::{Look, ScaleFamily};

/// Paint a callout using accent 3 for its surface and accent 11 for text and icon.
/// Applications supply content and assets; spacing and colors belong to the look.
pub fn callout(look: &Look, text: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> Div {
    let foreground = look.resolve_step(ScaleFamily::Color, 11).hsla();
    let path = match icon.into() {
        ControlIcon::Lucide(icon) => SharedString::from(icon.asset_path()),
        ControlIcon::SvgPath(path) => path,
    };
    div()
        .flex()
        .items_center()
        .gap(px(12.0))
        .w_full()
        .rounded_lg()
        .p_3()
        .bg(look.resolve_step(ScaleFamily::Color, 3).hsla())
        .child(div().flex_none().child(svg().path(path).size(px(15.0)).text_color(foreground)))
        .child(div().text_sm().text_color(foreground).child(text.into()))
}
