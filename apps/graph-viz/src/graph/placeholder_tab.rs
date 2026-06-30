use std::sync::Arc;

use gpui::{AnyElement, App, IntoElement, Window};
use gpui_luma_look_shadcn::ShadcnLook;

pub fn render_placeholder_tab(
    title: &'static str,
    look: &Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    look.card(&format!("graph-viz-{title}-placeholder"))
        .title(title)
        .description("Coming soon.")
        .elevated(false)
        .render(window, cx)
        .into_any_element()
}
