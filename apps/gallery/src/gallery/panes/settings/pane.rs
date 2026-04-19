use gpui::{AnyElement, IntoElement, div, prelude::*, rgb};

use super::super::shared::gallery_pane;

pub(in crate::gallery) fn render() -> AnyElement {
    gallery_pane("Settings", div().text_color(rgb(0x334155)).child("Settings").into_any_element())
}
