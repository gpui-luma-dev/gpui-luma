use std::sync::Arc;

use gpui::{AnyElement, Context, FontWeight, IntoElement, div, hsla, prelude::*, px};
use gpui_luma::dock_panel;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;

#[derive(Clone, Default)]
pub(in crate::gallery) struct DockPanelPane;

impl DockPanelPane {
    pub(in crate::gallery) fn new(_cx: &mut Context<GalleryApp>, _look: Arc<ShadcnLook>) -> Self {
        Self
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        let top_debug = hsla(0.58, 0.55, 0.42, 1.0);
        let left_debug = hsla(0.33, 0.55, 0.38, 1.0);
        let right_debug = hsla(0.12, 0.72, 0.46, 1.0);
        let bottom_debug = hsla(0.77, 0.48, 0.44, 1.0);
        let fill_debug = gpui::red();

        let chrome = look.chrome();

        div()
            .size_full()
            .relative()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(chrome.content_background)
            .p(px(28.0))
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_size(px(20.0))
                            .line_height(px(28.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(chrome.title_text)
                            .child("DockPanel"),
                    )
                    .child(
                        div()
                            .max_w(px(760.0))
                            .text_size(px(13.0))
                            .line_height(px(18.0))
                            .text_color(chrome.muted_text)
                            .child("Debug view for ordered docking geometry."),
                    ),
            )
            .child(div().flex_1().min_w(px(0.0)).min_h(px(0.0)).overflow_hidden().child(dock_panel! {
                left: div()
                    .w(px(52.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(left_debug)
                    .child("Left"),
                top: div()
                    .h(px(44.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(top_debug)
                    .child("Top"),
                right: div()
                    .w(px(68.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(right_debug)
                    .child("Right"),
                bottom: div()
                    .h(px(44.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(bottom_debug)
                    .child("Bottom"),
                fill: div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(fill_debug)
                    .child("Center/Fill")
            }))
            .into_any_element()
    }

    pub(in crate::gallery) fn subscribe(
        &self,
        _cx: &mut Context<GalleryApp>,
        _subscriptions: &mut Vec<gpui::Subscription>,
    ) {
    }

    pub(in crate::gallery) fn notify_controls(&self, _cx: &mut Context<GalleryApp>) {}
}
