use gpui::{AnyElement, IntoElement, div, prelude::*, px};

use crate::gallery::theme::GalleryThemePack;

use super::super::shared::gallery_pane;

pub(in crate::gallery) fn render(theme: &GalleryThemePack) -> AnyElement {
    let chrome = theme.chrome();

    gallery_pane(
        "Introduction",
        div()
            .max_w(px(520.0))
            .text_color(chrome.body_text)
            .line_height(px(22.0))
            .child(
                "Each control has a focused pane in the sidebar. The pane folder owns its examples, demo state, and gallery-only customization.",
            )
            .into_any_element(),
        theme,
    )
}
