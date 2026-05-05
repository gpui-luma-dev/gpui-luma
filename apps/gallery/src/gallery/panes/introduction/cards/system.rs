use gpui::{AnyElement, FontWeight, IntoElement, div, prelude::*, px};

use crate::gallery::theme::GalleryThemePack;

use super::common::{card_container, card_title};
use super::super::pane::IntroductionPane;

pub(in crate::gallery) fn render_system_card(pane: &IntroductionPane, theme: &GalleryThemePack) -> AnyElement {
    let chrome = theme.chrome();

    card_container(chrome.border, chrome.panel_background)
        .child(card_title(
            "System & Preferences",
            "Choice controls plus progress feedback.",
            chrome.title_text,
            chrome.muted_text,
        ))
        .child(pane.system.two_factor_switch.clone())
        .child(pane.system.terms_checkbox.clone())
        .child(pane.system.social_checkbox.clone())
        .child(pane.system.referral_checkbox.clone())
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(
                    div()
                        .text_size(px(12.0))
                        .line_height(px(16.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(chrome.body_text)
                        .child(format!("Budget: {:.0}%", pane.budget)),
                )
                .child(pane.system.budget_slider.clone()),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(
                    div()
                        .text_size(px(12.0))
                        .line_height(px(16.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(chrome.body_text)
                        .child(format!("Profile completion: {:.0}%", pane.completion)),
                )
                .child(pane.system.completion_progress.clone()),
        )
        .into_any_element()
}
