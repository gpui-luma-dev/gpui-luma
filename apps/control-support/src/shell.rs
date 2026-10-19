//! Look-neutral canvas geometry shared with shell-2026.
use gpui::{Div, Hsla, div, px, prelude::*};
// Paint the canvas here: GPUI cannot clip child fills to rounded corners.
pub fn workspace_canvas(background: Hsla, border: Hsla) -> Div {
    div()
        .flex_1()
        .min_w_0()
        .mt(px(2.0))
        .mr(px(3.0))
        .mb(px(3.0))
        .relative()
        .rounded(px(10.0))
        .overflow_hidden()
        .border(px(1.0))
        .border_color(border)
        .bg(background)
}
