use std::sync::Arc;

use gpui::{Context, Entity, IntoElement, Render, SharedString, Window, div, prelude::*, px};
use gpui_luma::controls::checkbox::Checkbox;
use gpui_luma::controls::combobox::{ComboBox, ComboBoxEvent, SelectionItem, TypingPolicy};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::command::button::{Button, ButtonEvent};
use gpui_luma::controls::textfield::{TextField, TextFieldEvent};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma::{declare_form, hstack, vstack};
use gpui_luma_look_shadcn::ShadcnLook;

use super::pane::{AppEvent, EventBus};

#[derive(Clone, Copy)]
enum PaymentField {
    Name,
    Email,
}

declare_form! {
    pub(super) struct PaymentPanel {
        controls: {
            submit_button: Entity<Button> = look.primary_button("intro-submit").label("Submit")
                => ButtonEvent |this, _event, cx| {
                    this.event_bus.update(cx, |_bus, cx| {
                        cx.emit(AppEvent::PaymentSubmit);
                    });
                },
            cancel_button: Entity<Button> = look.secondary_button("intro-cancel").label("Cancel")
                => ButtonEvent |this, _event, cx| {
                    this.event_bus.update(cx, |_bus, cx| {
                        cx.emit(AppEvent::PaymentCancel);
                    });
                },
            name_field: TextField = look
                .textfield("intro-name")
                .placeholder("Name on card")
                .full_width(true)
                .clean_on_escape(true)
                => TextFieldEvent |this, event, cx| {
                    this.handle_textfield_event(PaymentField::Name, event, cx);
                },
            email_field: TextField = look
                .textfield("intro-email")
                .placeholder("Email address")
                .full_width(true)
                .clean_on_escape(true)
                => TextFieldEvent |this, event, cx| {
                    this.handle_textfield_event(PaymentField::Email, event, cx);
                },
            payment_combobox: ComboBox = look
                .combobox("intro-payment-combobox", payment_method_items())
                .placeholder("Select payment method…")
                .full_width(true)
                .clean_on_escape(true)
                .typing_policy(TypingPolicy::Strict)
                .show_down_arrow(true)
                .show_clear_button(false)
                => ComboBoxEvent |this, event, cx| {
                    this.handle_payment_combobox_event(event, cx);
                },
            same_as_shipping_checkbox: Checkbox = look
                .primary_checkbox("intro-same-as-shipping")
                .with_data(true)
                .content(|_, _| div().child("Same as shipping address").into_any_element())
                => ButtonEvent |this, _event, cx| {
                    this.same_as_shipping = !this.same_as_shipping;
                    this.same_as_shipping_checkbox.update(cx, |button, cx| button.set_data(this.same_as_shipping, cx));
                    this.emit_change("Checkbox::SameAsShipping", cx);
                    cx.notify();
                },
            default_payment_method_checkbox: Checkbox = look
                .secondary_checkbox("intro-default-payment-method")
                .with_data(true)
                .content(|_, _| div().child("Use this as default payment method").into_any_element())
                => ButtonEvent |this, _event, cx| {
                    this.default_payment_method = !this.default_payment_method;
                    this.default_payment_method_checkbox
                        .update(cx, |button, cx| button.set_data(this.default_payment_method, cx));
                    this.emit_change("Checkbox::DefaultPaymentMethod", cx);
                    cx.notify();
                },
        },
        args: {
            look: Arc<ShadcnLook>,
            event_bus: Entity<EventBus>,
        },
        fields: {
            name_value: SharedString = SharedString::default(),
            email_value: SharedString = SharedString::default(),
            payment_selection_set: bool = false,
            same_as_shipping: bool = true,
            default_payment_method: bool = true,
        }
    }
}

impl PaymentPanel {
    fn handle_textfield_event(&mut self, field: PaymentField, event: &TextFieldEvent, cx: &mut Context<Self>) {
        match event {
            TextFieldEvent::Change { value } | TextFieldEvent::Submit { value } => {
                let next_value: SharedString = value.clone().into();
                match field {
                    PaymentField::Name => self.name_value = next_value,
                    PaymentField::Email => self.email_value = next_value,
                }
                self.emit_change("TextField::Change", cx);
            }
            TextFieldEvent::Focus | TextFieldEvent::Blur => {}
        }
    }

    fn handle_payment_combobox_event(&mut self, event: &ComboBoxEvent, cx: &mut Context<Self>) {
        match event {
            ComboBoxEvent::Select | ComboBoxEvent::Complete => {
                self.payment_selection_set = true;
                self.emit_change("ComboBox::Select", cx);
            }
            ComboBoxEvent::Clear => {
                self.payment_selection_set = false;
                self.emit_change("ComboBox::Clear", cx);
            }
            ComboBoxEvent::Change => {
                self.emit_change("ComboBox::Change", cx);
            }
        }
    }

    fn emit_change(&self, event_name: &'static str, cx: &mut Context<Self>) {
        let name = self.name_value.clone();
        let email = self.email_value.clone();
        let payment_selection_set = self.payment_selection_set;
        let same_as_shipping = self.same_as_shipping;
        let default_payment_method = self.default_payment_method;

        self.event_bus.update(cx, |_bus, cx| {
            cx.emit(AppEvent::PaymentChanged {
                name,
                email,
                payment_selection_set,
                same_as_shipping,
                default_payment_method,
                event_name: SharedString::from(event_name),
            });
        });
    }
}

impl Render for PaymentPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let name_field = self.name_field.clone();
        let email_field = self.email_field.clone();
        let payment_combobox = self.payment_combobox.clone();
        let same_as_shipping_checkbox = self.same_as_shipping_checkbox.clone();
        let default_payment_method_checkbox = self.default_payment_method_checkbox.clone();
        let submit_button = self.submit_button.clone();
        let cancel_button = self.cancel_button.clone();

        div().w(px(360.0)).max_w_full().h_full().child(
            self.look
                .card("intro-payment-card")
                .title("Payment Method")
                .description("All transactions are secure and encrypted.")
                .elevated(false)
                .full_height(true)
                .body_fill(true)
                .child_render(move |_, _| {
                    vstack! {
                        gap=10.0;
                        name_field.clone(),
                        email_field.clone(),
                        payment_combobox.clone(),
                        same_as_shipping_checkbox.clone(),
                        default_payment_method_checkbox.clone(),
                    }
                    .into_any_element()
                })
                .footer(move |_, _| {
                    hstack! {
                        gap=8.0 align=center;
                        submit_button.clone(),
                        cancel_button.clone(),
                    }
                    .into_any_element()
                })
                .render(window, cx),
        )
    }
}

fn payment_method_items() -> Vec<SelectionItem> {
    vec![
        SelectionItem::new("visa", "Visa •••• 4242"),
        SelectionItem::new("mastercard", "Mastercard •••• 4444"),
        SelectionItem::new("amex", "Amex •••• 0005"),
        SelectionItem::new("apple-pay", "Apple Pay"),
        SelectionItem::new("google-pay", "Google Pay"),
    ]
}
