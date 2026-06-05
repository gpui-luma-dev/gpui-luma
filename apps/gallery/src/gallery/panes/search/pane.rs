use gpui::{AnyElement, IntoElement, div, prelude::*};
use gpui_luma_theme_radix::RadixTheme;

use super::super::shared::gallery_pane;

pub(in crate::gallery) fn render(radix_theme: &RadixTheme) -> AnyElement {
    let chrome = radix_theme.chrome();

    gallery_pane("Search", div().text_color(chrome.body_text).child("Search").into_any_element(), radix_theme)
}
