use gpui::{AnyElement, IntoElement, div, prelude::*, px, rgb};

use super::super::shared::gallery_pane;

pub(in crate::gallery) fn render() -> AnyElement {
    gallery_pane(
        "Introduction",
        div()
            .max_w(px(520.0))
            .text_color(rgb(0x334155))
            .line_height(px(22.0))
            .child(
                "Each control has a focused pane in the sidebar. The pane folder owns its examples, demo state, and gallery-only customization.",
            )
            .into_any_element(),
    )
}
