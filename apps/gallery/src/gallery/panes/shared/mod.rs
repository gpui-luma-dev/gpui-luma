use gpui::{AnyElement, Context, Entity, IntoElement, div, prelude::*, px};

use crate::gallery::{control::GalleryApp, theme::GalleryThemePack};

pub(super) fn gallery_pane(title: &'static str, content: AnyElement, theme: &GalleryThemePack) -> AnyElement {
    let chrome = theme.chrome();

    div()
        .size_full()
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .overflow_hidden()
        .bg(chrome.content_background)
        .child(
            div()
                .relative()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_4()
                .occlude()
                .child(div().text_size(px(20.0)).line_height(px(28.0)).text_color(chrome.title_text).child(title))
                .child(content),
        )
        .into_any_element()
}

pub(super) fn notify_entity<T: 'static>(entity: &Entity<T>, cx: &mut Context<GalleryApp>) {
    entity.update(cx, |_, cx| cx.notify());
}
