use gpui::{AnyElement, FontWeight, IntoElement, div, prelude::*, px};

pub(super) fn card_container(border: gpui::Hsla, panel: gpui::Hsla, content: impl IntoElement) -> gpui::Div {
    div()
        .w(px(360.0))
        .max_w_full()
        .border_1()
        .border_color(border)
        .rounded(px(12.0))
        .bg(panel)
        .p(px(14.0))
        .child(content)
}

pub(super) fn card_title(
    title: &'static str,
    subtitle: &'static str,
    title_color: gpui::Hsla,
    subtitle_color: gpui::Hsla,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(2.0))
        .child(
            div()
                .text_size(px(18.0))
                .line_height(px(24.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(title_color)
                .child(title),
        )
        .child(div().text_size(px(12.0)).line_height(px(16.0)).text_color(subtitle_color).child(subtitle))
        .into_any_element()
}
