use gpui::{AnyElement, IntoElement, div, prelude::*, px};

use crate::studio::doc_shell::{
    SECTION_HEADING_CONTENT_GAP, SECTION_HEADING_SHELL_PAD_BOTTOM, SECTION_HEADING_SHELL_PAD_TOP,
    render_section_heading_anchor,
};

pub(crate) fn section_shell_with_width(
    width: f32,
    title: &'static str,
    description: &'static str,
    title_color: gpui::Hsla,
    muted_text: gpui::Hsla,
    border: gpui::Hsla,
    _background: gpui::Hsla,
    content: AnyElement,
) -> AnyElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .pt(px(SECTION_HEADING_SHELL_PAD_TOP))
        .pb(px(SECTION_HEADING_SHELL_PAD_BOTTOM))
        .child(render_section_heading_anchor(title, description, title_color, muted_text, border))
        .child(
            div()
                .w_full()
                .flex()
                .justify_center()
                .mt(px(SECTION_HEADING_CONTENT_GAP))
                .child(div().w(px(width)).max_w_full().child(content)),
        )
        .into_any_element()
}
