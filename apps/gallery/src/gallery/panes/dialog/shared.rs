use gpui::{AnyElement, div, prelude::*, px};
use gpui_luma::controls::overlay_window::OverlayWindowEvent;
use gpui_luma_look_shadcn::ShadcnLook;

pub(super) fn render_stage(content: AnyElement) -> AnyElement {
    div()
        .relative()
        .w_full()
        .max_w(px(860.0))
        .min_h(px(420.0))
        .rounded(px(24.0))
        .border_1()
        .border_color(gpui::transparent_black())
        .occlude()
        .child(div().absolute().inset_0().bg(gpui::hsla(0.58, 0.14, 0.97, 1.0)).opacity(0.72))
        .child(div().relative().size_full().flex().items_center().justify_center().p(px(28.0)).child(content))
        .into_any_element()
}

pub(super) fn render_status(label: &'static str, value: &str, _look: &ShadcnLook) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(div().text_xs().child(label))
        .child(div().text_sm().child(value.to_string()))
        .into_any_element()
}

pub(super) fn status_for(event: &OverlayWindowEvent, label: &str) -> String {
    match event {
        OverlayWindowEvent::Opened => format!("Opened {label}."),
        OverlayWindowEvent::Dismissed => format!("Dismissed {label}."),
        _ => format!("Unchanged {label}."),
    }
}
