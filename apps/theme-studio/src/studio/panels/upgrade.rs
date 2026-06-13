use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::checkbox::Checkbox;
use gpui_luma::controls::command::button::{Button, ButtonEvent};
use gpui_luma::controls::radio_group::{RadioGroup, RadioGroupItem};
use gpui_luma::controls::textarea::TextArea;
use gpui_luma::controls::textfield::TextField;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma::theme::{ControlSize};
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma::{declare_form, form_field, hstack, vstack};

use super::common::titled_card;

declare_form! {
    pub struct UpgradePanel {
        controls: {
            name_field: TextField = look.textfield("upgrade-name").placeholder("Name").full_width(true),
            email_field: TextField = look.textfield("upgrade-email").placeholder("Email").full_width(true),
            card_field: TextField = look.textfield("upgrade-card").placeholder("Card Number").full_width(true),
            expiry_field: TextField = look.textfield("upgrade-expiry").placeholder("MM/YY"),
            cvc_field: TextField = look.textfield("upgrade-cvc").placeholder("CVC"),
            plan_group: RadioGroup<RadioGroupItem> = look
                .radio_group("upgrade-plan")
                .items([
                    RadioGroupItem::new("starter").label("Starter Plan"),
                    RadioGroupItem::new("pro").label("Pro Plan"),
                ])
                .selected("starter"),
            notes_area: Entity<TextArea> = look
                .textarea("upgrade-notes")
                .placeholder("Notes")
                .full_width(true)
                .rows(3),
            terms_checkbox: Checkbox = look
                .primary_checkbox("upgrade-terms")
                .with_data(true)
                .content(|_, _| div().child("I agree to the terms and conditions").into_any_element())
                => ButtonEvent |this, _event, cx| {
                    this.terms_accepted = !this.terms_accepted;
                    this.terms_checkbox.update(cx, |button, cx| button.set_data(this.terms_accepted, cx));
                },
            email_checkbox: Checkbox = look
                .primary_checkbox("upgrade-email-opt")
                .with_data(false)
                .content(|_, _| div().child("Allow us to send you emails").into_any_element())
                => ButtonEvent |this, _event, cx| {
                    this.email_opt_in = !this.email_opt_in;
                    this.email_checkbox.update(cx, |button, cx| button.set_data(this.email_opt_in, cx));
                },
            cancel_button: Entity<Button> = look.secondary_button("upgrade-cancel").label("Cancel").size(size),
            upgrade_button: Entity<Button> = look.primary_button("upgrade-submit").label("Upgrade Plan").size(size),
        },
        args: {
            look: Arc<ShadcnLook>,
            size: ControlSize,
        },
        fields: {
            terms_accepted: bool = true,
            email_opt_in: bool = false,
        }
    }
}

impl Render for UpgradePanel {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let name_field = self.name_field.clone();
        let email_field = self.email_field.clone();
        let card_field = self.card_field.clone();
        let expiry_field = self.expiry_field.clone();
        let cvc_field = self.cvc_field.clone();
        let plan_group = self.plan_group.clone();
        let notes_area = self.notes_area.clone();
        let terms_checkbox = self.terms_checkbox.clone();
        let email_checkbox = self.email_checkbox.clone();
        let cancel_button = self.cancel_button.clone();
        let upgrade_button = self.upgrade_button.clone();

        titled_card(
            "theme-studio-upgrade-card",
            &self.look,
            380.0,
            "Upgrade your subscription",
            "You are currently on the free plan. Upgrade to unlock all features.",
            move |_, _| {
                vstack! {
                    gap=12;
                    hstack! {
                        gap=10;
                        form_field!("Name", chrome; name_field.clone()).flex_1(),
                        form_field!("Email", chrome; email_field.clone()).flex_1(),
                    },
                    form_field!("Card Number", chrome;
                        hstack! {
                            gap=8;
                            div().flex_1().child(card_field.clone()),
                            div().w(px(72.0)).child(expiry_field.clone()),
                            div().w(px(64.0)).child(cvc_field.clone()),
                        }
                    ),
                    form_field!("Plan", chrome; plan_group.clone()),
                    form_field!("Notes", chrome; notes_area.clone()),
                    terms_checkbox.clone(),
                    email_checkbox.clone(),
                    hstack! {
                        gap=8 justify=end;
                        cancel_button.clone(),
                        upgrade_button.clone(),
                    },
                }
                .into_any_element()
            },
            window,
            _cx,
        )
    }
}
