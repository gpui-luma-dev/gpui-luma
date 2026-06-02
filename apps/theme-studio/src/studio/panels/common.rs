use gpui::{AnyElement, FontWeight, Hsla, IntoElement, div, prelude::*, px};
use gpui_luma::theme::LumaChrome;
use gpui_luma::hstack;

pub fn format_hsla(color: Hsla) -> String {
    format!(
        "hsla({} {}% {}% / {})",
        rounded_channel(color.h * 360.0),
        rounded_channel(color.s * 100.0),
        rounded_channel(color.l * 100.0),
        compact_alpha(color.a)
    )
}

fn rounded_channel(value: f32) -> i32 {
    value.round() as i32
}

fn compact_alpha(alpha: f32) -> String {
    if (alpha - 1.0).abs() < 0.001 {
        "1".to_string()
    } else {
        format!("{alpha:.2}")
    }
}

pub fn panel_drag_handle(chrome: LumaChrome) -> gpui::Div {
    div()
        .w_full()
        .h(px(22.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded_t(px(10.0))
        .bg(gpui::hsla(0.0, 0.0, 1.0, 0.05))
        .text_size(px(11.0))
        .text_color(chrome.muted_text)
        .cursor_pointer()
        .child("⋮⋮")
}

pub fn card(width: f32, border: Hsla, background: Hsla, content: impl IntoElement) -> gpui::Div {
    div()
        .w(px(width))
        .max_w_full()
        .overflow_hidden()
        .border_1()
        .border_color(border)
        .rounded(px(12.0))
        .bg(background)
        .p(px(16.0))
        .child(content)
}

pub fn card_header(title: &'static str, subtitle: &'static str, title_color: Hsla, subtitle_color: Hsla) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(
            div()
                .text_size(px(16.0))
                .line_height(px(22.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(title_color)
                .child(title),
        )
        .child(div().text_size(px(12.0)).line_height(px(16.0)).text_color(subtitle_color).child(subtitle))
        .into_any_element()
}

pub fn or_divider(label: &'static str, border: Hsla, text: Hsla) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(div().flex_1().h(px(1.0)).bg(border))
        .child(div().text_size(px(10.0)).line_height(px(12.0)).text_color(text).child(label))
        .child(div().flex_1().h(px(1.0)).bg(border))
}

pub fn avatar_circle(initials: &'static str, size: f32, bg: Hsla, fg: Hsla) -> impl IntoElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .bg(bg)
        .text_size(px(size * 0.38))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(fg)
        .child(initials)
}

pub fn message_bubble(text: &'static str, align_end: bool, bg: Hsla, fg: Hsla) -> impl IntoElement {
    let bubble = div()
        .max_w(px(220.0))
        .px(px(10.0))
        .py(px(8.0))
        .rounded(px(10.0))
        .bg(bg)
        .text_size(px(12.0))
        .line_height(px(16.0))
        .text_color(fg)
        .child(text);

    if align_end {
        hstack! { justify=end; bubble }
    } else {
        div().child(bubble)
    }
}
