use gpui::{AnyElement, IntoElement, div, prelude::*, rgb};

use super::super::shared::gallery_pane;

pub(in crate::gallery) fn render() -> AnyElement {
    gallery_pane("Search", div().text_color(rgb(0x334155)).child("Search").into_any_element())
}
