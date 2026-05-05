use gpui::{AnyElement, IntoElement, div, prelude::*, px};

use crate::gallery::theme::GalleryThemePack;

use super::common::{card_container, card_title};
use super::super::pane::IntroductionPane;

pub(in crate::gallery) fn render_payment_card(pane: &IntroductionPane, theme: &GalleryThemePack) -> AnyElement {
    let chrome = theme.chrome();

    card_container(chrome.border, chrome.panel_background)
        .child(card_title(
            "Payment Method",
            "All transactions are secure and encrypted.",
            chrome.title_text,
            chrome.muted_text,
        ))
        .child(pane.payment.name_field.clone())
        .child(pane.payment.email_field.clone())
        .child(pane.payment.payment_combobox.clone())
        .child(pane.payment.same_as_shipping_checkbox.clone())
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(pane.payment.submit_button.clone())
                .child(pane.payment.cancel_button.clone()),
        )
        .child(pane.payment.payment_method_radio.clone())
        .into_any_element()
}
