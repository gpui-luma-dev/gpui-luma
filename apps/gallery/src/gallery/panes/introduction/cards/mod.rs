mod builders;
mod common;
mod payment;
mod system;
mod workspace;

use gpui::{AnyElement, IntoElement, div, prelude::*, px};

use crate::gallery::theme::GalleryThemePack;

use super::pane::IntroductionPane;
pub(in crate::gallery) use builders::{build_payment_panel, build_system_panel, build_workspace_panel};
pub(in crate::gallery) use payment::render_payment_card;
pub(in crate::gallery) use system::render_system_card;
pub(in crate::gallery) use workspace::render_workspace_card;

pub(in crate::gallery) fn render_cards_row(pane: &IntroductionPane, theme: &GalleryThemePack) -> AnyElement {
    div()
        .w_full()
        .mt(px(32.0))
        .flex()
        .flex_wrap()
        .justify_center()
        .items_stretch()
        .gap(px(16.0))
        .child(render_payment_card(pane, theme))
        .child(render_workspace_card(pane, theme))
        .child(render_system_card(pane, theme))
        .into_any_element()
}
