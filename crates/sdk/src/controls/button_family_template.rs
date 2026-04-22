use gpui::{Div, Hsla, SharedString, Stateful, div, hsla, px, prelude::*};

const FOCUS_RING_GAP: f32 = 1.0;
const FOCUS_RING_WIDTH: f32 = 1.0;

pub(crate) fn render_button_family_focus_ring(
    id: SharedString,
    control: Div,
    focus_ring: Option<Hsla>,
    radius: f32,
) -> Stateful<Div> {
    let ring_color = focus_ring.unwrap_or_else(|| hsla(0.0, 0.0, 0.0, 0.0));
    let ring_radius = radius + FOCUS_RING_GAP + FOCUS_RING_WIDTH;

    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .p(px(FOCUS_RING_GAP))
        .border_1()
        .border_color(ring_color)
        .rounded(px(ring_radius))
        .child(control)
}
