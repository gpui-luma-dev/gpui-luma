use gpui::{AnyElement, IntoElement, div, prelude::*, px, rgb};

pub(super) fn gallery_pane(title: &'static str, content: AnyElement) -> AnyElement {
    div()
        .size_full()
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .overflow_hidden()
        .child(
            div()
                .relative()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_4()
                .occlude()
                .child(div().text_size(px(20.0)).line_height(px(28.0)).text_color(rgb(0x0f172a)).child(title))
                .child(content),
        )
        .into_any_element()
}
