use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use gpui_luma::controls::checkbox::Checkbox;
use gpui_luma::controls::command::button::Button;
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::radio_group::{RadioGroup, RadioGroupItem};
use gpui_luma::controls::textarea::TextArea;
use gpui_luma::controls::textfield::TextField;
use gpui_luma::theme::radix::prelude::*;
use gpui_luma::theme::{ControlSize, RadixTheme};

use super::common::{card, card_header, field_label};
pub struct UpgradePanel {
    radix_theme: Arc<RadixTheme>,
    name_field: TextField,
    email_field: TextField,
    card_field: TextField,
    expiry_field: TextField,
    cvc_field: TextField,
    plan_group: RadioGroup<RadioGroupItem>,
    notes_area: Entity<TextArea>,
    terms_checkbox: Checkbox,
    email_checkbox: Checkbox,
    cancel_button: Entity<Button>,
    upgrade_button: Entity<Button>,
}

impl UpgradePanel {
    pub fn new(cx: &mut Context<Self>, radix_theme: Arc<RadixTheme>, size: ControlSize) -> Self {
        Self {
            name_field: radix_theme.textfield("upgrade-name").placeholder("Name").full_width(true).spawn(cx),
            email_field: radix_theme.textfield("upgrade-email").placeholder("Email").full_width(true).spawn(cx),
            card_field: radix_theme.textfield("upgrade-card").placeholder("Card Number").full_width(true).spawn(cx),
            expiry_field: radix_theme.textfield("upgrade-expiry").placeholder("MM/YY").spawn(cx),
            cvc_field: radix_theme.textfield("upgrade-cvc").placeholder("CVC").spawn(cx),
            plan_group: radix_theme
                .radio_group("upgrade-plan")
                .items([
                    RadioGroupItem::new("starter").label("Starter Plan"),
                    RadioGroupItem::new("pro").label("Pro Plan"),
                ])
                .selected("starter")
                .spawn(cx),
            notes_area: radix_theme.textarea("upgrade-notes").placeholder("Notes").full_width(true).rows(3).spawn(cx),
            terms_checkbox: radix_theme
                .primary_checkbox("upgrade-terms")
                .with_data(true)
                .content(|_, _| div().child("I agree to the terms and conditions").into_any_element())
                .spawn(cx),
            email_checkbox: radix_theme
                .primary_checkbox("upgrade-email-opt")
                .with_data(false)
                .content(|_, _| div().child("Allow us to send you emails").into_any_element())
                .spawn(cx),
            cancel_button: radix_theme.ghost_button("upgrade-cancel").label("Cancel").size(size).spawn(cx),
            upgrade_button: radix_theme.primary_button("upgrade-submit").label("Upgrade Plan").size(size).spawn(cx),
            radix_theme,
        }
    }
}

impl Render for UpgradePanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.radix_theme.chrome();

        card(
            380.0,
            chrome.border,
            chrome.panel_background,
            div()
                .flex()
                .flex_col()
                .gap(px(12.0))
                .child(card_header(
                    "Upgrade your subscription",
                    "You are currently on the free plan. Upgrade to unlock all features.",
                    chrome.title_text,
                    chrome.muted_text,
                ))
                .child(
                    div()
                        .flex()
                        .gap(px(10.0))
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .child(field_label("Name", chrome.body_text))
                                .child(self.name_field.clone()),
                        )
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .child(field_label("Email", chrome.body_text))
                                .child(self.email_field.clone()),
                        ),
                )
                .child(field_label("Card Number", chrome.body_text))
                .child(
                    div()
                        .flex()
                        .gap(px(8.0))
                        .child(div().flex_1().child(self.card_field.clone()))
                        .child(div().w(px(72.0)).child(self.expiry_field.clone()))
                        .child(div().w(px(64.0)).child(self.cvc_field.clone())),
                )
                .child(field_label("Plan", chrome.body_text))
                .child(self.plan_group.clone())
                .child(field_label("Notes", chrome.body_text))
                .child(self.notes_area.clone())
                .child(self.terms_checkbox.clone())
                .child(self.email_checkbox.clone())
                .child(
                    div()
                        .flex()
                        .justify_end()
                        .gap(px(8.0))
                        .child(self.cancel_button.clone())
                        .child(self.upgrade_button.clone()),
                ),
        )
    }
}
