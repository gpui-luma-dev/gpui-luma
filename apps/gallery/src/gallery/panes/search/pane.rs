use gpui::{AnyElement, IntoElement, div, prelude::*};

use crate::gallery::theme::GalleryThemePack;

use super::super::shared::gallery_pane;

pub(in crate::gallery) fn render(theme: &GalleryThemePack) -> AnyElement {
    let chrome = theme.chrome();

    gallery_pane("Search", div().text_color(chrome.body_text).child("Search").into_any_element(), theme)
}
