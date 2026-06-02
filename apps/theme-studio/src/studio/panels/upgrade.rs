use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::checkbox::Checkbox;
use gpui_luma::controls::command::button::{Button, ButtonEvent};
use gpui_luma::controls::radio_group::{RadioGroup, RadioGroupItem};
use gpui_luma::controls::textarea::TextArea;
use gpui_luma::controls::textfield::TextField;
use gpui_luma::theme::radix::prelude::*;
use gpui_luma::theme::{ControlSize, RadixTheme};
use gpui_luma::{declare_form, form_field, hstack, vstack};

use super::common::{card, card_header};

declare_form! {
    pub struct UpgradePanel {
        controls: {
            name_field: TextField = radix_theme.textfield("upgrade-name").placeholder("Name").full_width(true),
            email_field: TextField = radix_theme.textfield("upgrade-email").placeholder("Email").full_width(true),
            card_field: TextField = radix_theme.textfield("upgrade-card").placeholder("Card Number").full_width(true),
            expiry_field: TextField = radix_theme.textfield("upgrade-expiry").placeholder("MM/YY"),
            cvc_field: TextField = radix_theme.textfield("upgrade-cvc").placeholder("CVC"),
            plan_group: RadioGroup<RadioGroupItem> = radix_theme
                .radio_group("upgrade-plan")
                .items([
                    RadioGroupItem::new("starter").label("Starter Plan"),
                    RadioGroupItem::new("pro").label("Pro Plan"),
                ])
                .selected("starter"),
            notes_area: Entity<TextArea> = radix_theme
                .textarea("upgrade-notes")
                .placeholder("Notes")
                .full_width(true)
                .rows(3),
            terms_checkbox: Checkbox = radix_theme
                .primary_checkbox("upgrade-terms")
                .with_data(true)
                .content(|_, _| div().child("I agree to the terms and conditions").into_any_element())
                => ButtonEvent |this, _event, cx| {
                    this.terms_accepted = !this.terms_accepted;
                    this.terms_checkbox.update(cx, |button, cx| button.set_data(this.terms_accepted, cx));
                },
            email_checkbox: Checkbox = radix_theme
                .primary_checkbox("upgrade-email-opt")
                .with_data(false)
                .content(|_, _| div().child("Allow us to send you emails").into_any_element())
                => ButtonEvent |this, _event, cx| {
                    this.email_opt_in = !this.email_opt_in;
                    this.email_checkbox.update(cx, |button, cx| button.set_data(this.email_opt_in, cx));
                },
            cancel_button: Entity<Button> = radix_theme.secondary_button("upgrade-cancel").label("Cancel").size(size),
            upgrade_button: Entity<Button> = radix_theme.primary_button("upgrade-submit").label("Upgrade Plan").size(size),
        },
        args: {
            radix_theme: Arc<RadixTheme>,
            size: ControlSize,
        },
        fields: {
            terms_accepted: bool = true,
            email_opt_in: bool = false,
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
