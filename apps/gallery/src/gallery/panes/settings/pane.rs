use gpui::{AnyElement, IntoElement, div, prelude::*};
use gpui_luma_look_shadcn::ShadcnLook;

use super::super::shared::gallery_pane;

pub(in crate::gallery) fn render(look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();

    gallery_pane("Settings", div().text_color(chrome.body_text).child("Settings").into_any_element(), look)
}
