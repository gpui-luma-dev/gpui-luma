use gpui::{Context, Entity, IntoElement, Render, SharedString, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::checkbox::{self, Checkbox};
use gpui_luma::controls::combobox::{self, ComboBox, ComboBoxEvent, SelectionItem, TypingPolicy};
use gpui_luma::controls::command::button::{Button, ButtonEvent, ButtonKind};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::radio_button;
use gpui_luma::controls::textfield::{self, TextField, TextFieldEvent};

use crate::gallery::theme::GalleryThemePack;

use super::common::{card_container, card_title};
use super::pane::{AppEvent, EventBus};

#[derive(Clone, Copy)]
enum PaymentButton {
    Submit,
    Cancel,
}

#[derive(Clone, Copy)]
enum PaymentField {
    Name,
    Email,
}

pub(super) struct PaymentPanel {
    theme: GalleryThemePack,
    event_bus: Entity<EventBus>,
    submit_button: Entity<Button>,
    cancel_button: Entity<Button>,
    name_field: TextField,
    email_field: TextField,
    payment_combobox: ComboBox,
    same_as_shipping_checkbox: Checkbox,
    payment_method_radio: radio_button::RadioButton,
    name_value: SharedString,
    email_value: SharedString,
    payment_selection_set: bool,
    same_as_shipping: bool,
    _subscriptions: Vec<Subscription>,
}

impl PaymentPanel {
    pub(super) fn new(cx: &mut Context<Self>, theme: &GalleryThemePack, event_bus: Entity<EventBus>) -> Self {
        let submit_button = Button::new("intro-submit").label("Submit").kind(ButtonKind::Prominent).spawn(cx);
        let cancel_button = Button::new("intro-cancel").label("Cancel").spawn(cx);
        let name_field = textfield::new("intro-name")
            .placeholder("Name on card")
            .full_width(true)
            .clean_on_escape(true)
            .spawn(cx);
        let email_field = textfield::new("intro-email")
            .placeholder("Email address")
            .full_width(true)
            .clean_on_escape(true)
            .spawn(cx);
        let payment_combobox = combobox::new("intro-payment-combobox", payment_method_items())
            .placeholder("Select payment method…")
            .full_width(true)
            .clean_on_escape(true)
            .typing_policy(TypingPolicy::Strict)
            .show_down_arrow(true)
            .show_clear_button(false)
            .spawn(cx);
        let same_as_shipping_checkbox = checkbox::new("intro-same-as-shipping")
            .with_data(true)
            .content(|_, _| div().child("Same as shipping address").into_any_element())
            .spawn(cx);
        let payment_method_radio = radio_button::new("intro-payment-method-radio")
            .with_data(true)
            .content(|_, _| div().child("Use this as default payment method").into_any_element())
            .spawn(cx);

        let subscriptions = vec![
            cx.subscribe(&submit_button, |this, _, _: &ButtonEvent, cx| {
                this.handle_button_event(PaymentButton::Submit, cx);
            }),
            cx.subscribe(&cancel_button, |this, _, _: &ButtonEvent, cx| {
                this.handle_button_event(PaymentButton::Cancel, cx);
            }),
            cx.subscribe(&name_field, |this, _, event: &TextFieldEvent, cx| {
                this.handle_textfield_event(PaymentField::Name, event, cx);
            }),
            cx.subscribe(&email_field, |this, _, event: &TextFieldEvent, cx| {
                this.handle_textfield_event(PaymentField::Email, event, cx);
            }),
            cx.subscribe(&payment_combobox, |this, _, event: &ComboBoxEvent, cx| {
                this.handle_payment_combobox_event(event, cx);
            }),
            cx.subscribe(&same_as_shipping_checkbox, |this, _, _: &ButtonEvent, cx| {
                this.handle_same_as_shipping_event(cx);
            }),
        ];

        Self {
            theme: theme.clone(),
            event_bus,
            submit_button,
            cancel_button,
            name_field,
            email_field,
            payment_combobox,
            same_as_shipping_checkbox,
            payment_method_radio,
            name_value: SharedString::default(),
            email_value: SharedString::default(),
            payment_selection_set: false,
            same_as_shipping: true,
            _subscriptions: subscriptions,
        }
    }

    fn handle_button_event(&mut self, button: PaymentButton, cx: &mut Context<Self>) {
        self.event_bus.update(cx, |_bus, cx| match button {
            PaymentButton::Submit => cx.emit(AppEvent::PaymentSubmit),
            PaymentButton::Cancel => cx.emit(AppEvent::PaymentCancel),
        });
    }

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

    fn handle_same_as_shipping_event(&mut self, cx: &mut Context<Self>) {
        self.same_as_shipping = !self.same_as_shipping;
        self.same_as_shipping_checkbox.update(cx, |button, cx| button.set_data(self.same_as_shipping, cx));
        self.emit_change("Checkbox::SameAsShipping", cx);
        cx.notify();
    }

    fn emit_change(&self, event_name: &'static str, cx: &mut Context<Self>) {
        let name = self.name_value.clone();
        let email = self.email_value.clone();
        let payment_selection_set = self.payment_selection_set;
        let same_as_shipping = self.same_as_shipping;

        self.event_bus.update(cx, |_bus, cx| {
            cx.emit(AppEvent::PaymentChanged {
                name,
                email,
                payment_selection_set,
                same_as_shipping,
                event_name: SharedString::from(event_name),
            });
        });
    }
}

impl Render for PaymentPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.theme.chrome();

        card_container(chrome.border, chrome.panel_background)
            .child(card_title(
                "Payment Method",
                "All transactions are secure and encrypted.",
                chrome.title_text,
                chrome.muted_text,
            ))
            .child(self.name_field.clone())
            .child(self.email_field.clone())
            .child(self.payment_combobox.clone())
            .child(self.same_as_shipping_checkbox.clone())
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(self.submit_button.clone())
                    .child(self.cancel_button.clone()),
            )
            .child(self.payment_method_radio.clone())
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
