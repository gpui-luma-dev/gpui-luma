use gpui::{AnyElement, Hsla, IntoElement, ParentElement, Styled, div, px, rgb};

pub fn shell_content_pane() -> AnyElement {
    shell_content_pane_with_background(rgb(0x000000).into())
}

/// Render the shared content with a custom outer background fill.
pub fn shell_content_pane_with_background(background: Hsla) -> AnyElement {
    div()
        .size_full()
        .bg(background)
        .p(px(10.0))
        .child(
            div()
                .size_full()
                .flex()
                .child(div().h_full().flex_1().rounded(px(16.0)).bg(rgb(0x242835)))
                .child(div().w(px(20.0))),
        )
        .into_any_element()
}

pub fn inset_content_pane() -> AnyElement {
    div().size_full().rounded(px(16.0)).bg(rgb(0x000000)).into_any_element()
}
