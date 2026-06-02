use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::checkbox::Checkbox;
use gpui_luma::controls::command::button::{Button, ButtonEvent};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::radio_group::{RadioGroup, RadioGroupItem};
use gpui_luma::controls::textarea::TextArea;
use gpui_luma::controls::textfield::TextField;
use gpui_luma::theme::radix::prelude::*;
use gpui_luma::theme::{ControlSize, RadixTheme};
use gpui_luma::{form_field, hstack, vstack};

use super::common::{card, card_header};

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
    _subscriptions: Vec<Subscription>,
}

impl UpgradePanel {
    pub fn new(cx: &mut Context<Self>, radix_theme: Arc<RadixTheme>, size: ControlSize) -> Self {
        let terms_checkbox = radix_theme
            .primary_checkbox("upgrade-terms")
            .with_data(true)
            .content(|_, _| div().child("I agree to the terms and conditions").into_any_element())
            .spawn(cx);
        let email_checkbox = radix_theme
            .primary_checkbox("upgrade-email-opt")
            .with_data(false)
            .content(|_, _| div().child("Allow us to send you emails").into_any_element())
            .spawn(cx);

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&terms_checkbox, |_, checkbox, event, cx| {
            if matches!(event, ButtonEvent::Click) {
                checkbox.update(cx, |button, cx| button.set_data(!*button.data(), cx));
            }
        }));
        subscriptions.push(cx.subscribe(&email_checkbox, |_, checkbox, event, cx| {
            if matches!(event, ButtonEvent::Click) {
                checkbox.update(cx, |button, cx| button.set_data(!*button.data(), cx));
            }
        }));

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
            terms_checkbox,
            email_checkbox,
            cancel_button: radix_theme.secondary_button("upgrade-cancel").label("Cancel").size(size).spawn(cx),
            upgrade_button: radix_theme.primary_button("upgrade-submit").label("Upgrade Plan").size(size).spawn(cx),
            radix_theme,
            _subscriptions: subscriptions,
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
            vstack! {
                gap=12;
                card_header(
                    "Upgrade your subscription",
                    "You are currently on the free plan. Upgrade to unlock all features.",
                    chrome.title_text,
                    chrome.muted_text,
                ),
                hstack! {
                    gap=10;
                    form_field!("Name", chrome; self.name_field.clone()).flex_1(),
                    form_field!("Email", chrome; self.email_field.clone()).flex_1(),
                },
                form_field!("Card Number", chrome;
                    hstack! {
                        gap=8;
                        div().flex_1().child(self.card_field.clone()),
                        div().w(px(72.0)).child(self.expiry_field.clone()),
                        div().w(px(64.0)).child(self.cvc_field.clone()),
                    }
                ),
                form_field!("Plan", chrome; self.plan_group.clone()),
                form_field!("Notes", chrome; self.notes_area.clone()),
                self.terms_checkbox.clone(),
                self.email_checkbox.clone(),
                hstack! {
                    gap=8 justify=end;
                    self.cancel_button.clone(),
                    self.upgrade_button.clone(),
                },
            },
        )
    }
}
