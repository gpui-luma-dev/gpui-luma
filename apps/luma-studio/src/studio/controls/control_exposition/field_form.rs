use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Render, SharedString, Window, div, prelude::*, px};
use gpui_luma::controls::checkbox::Checkbox;
use gpui_luma::controls::command::button::{Button, ButtonEvent};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::selector::{Selector, SelectorItem};
use gpui_luma::controls::textarea::TextArea;
use gpui_luma::controls::textfield::TextField;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook, ShadcnLookControlExt};

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::model::ControlExpositionLayout;
use super::template::render_control_exposition_card;

pub struct FieldFormControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    preview: Entity<FieldFormPreview>,
}

struct FieldFormPreview {
    look: Arc<ShadcnLook>,
    name: TextField,
    card_number: TextField,
    month: Entity<Selector<SelectorItem>>,
    year: Entity<Selector<SelectorItem>>,
    cvv: TextField,
    same_as_shipping: Checkbox,
    comments: Entity<TextArea>,
    submit_button: Entity<Button<()>>,
    cancel_button: Entity<Button<()>>,
    status: SharedString,
}

pub(crate) struct Field {
    label: SharedString,
    control: AnyElement,
    description: Option<SharedString>,
    required: bool,
}

impl Field {
    pub(crate) fn new(label: impl Into<SharedString>, control: impl Into<AnyElement>) -> Self {
        Self { label: label.into(), control: control.into(), description: None, required: false }
    }

    pub(crate) fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub(crate) fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub(crate) fn build(self) -> AnyElement {
        let label = div()
            .flex()
            .items_center()
            .text_size(px(14.0))
            .line_height(px(20.0))
            .font_weight(gpui::FontWeight::MEDIUM)
            .child(self.label);
        let mut field = div().flex().flex_col().gap(px(8.0)).child(label).child(self.control);
        if let Some(description) = self.description {
            field = field.child(
                div()
                    .text_size(px(13.0))
                    .line_height(px(18.0))
                    .text_color(gpui::hsla(0.0, 0.0, 0.5, 1.0))
                    .child(description),
            );
        }
        field.into_any_element()
    }
}

pub(crate) struct FieldGroup {
    children: Vec<AnyElement>,
}

impl FieldGroup {
    pub(crate) fn new() -> Self {
        Self { children: Vec::new() }
    }

    pub(crate) fn child(mut self, child: impl Into<AnyElement>) -> Self {
        self.children.push(child.into());
        self
    }

    pub(crate) fn build(self) -> AnyElement {
        div().flex().flex_col().gap(px(20.0)).children(self.children).into_any_element()
    }
}

pub(crate) struct FieldSet {
    legend: SharedString,
    description: Option<SharedString>,
    content: Option<AnyElement>,
}

impl FieldSet {
    pub(crate) fn new(legend: impl Into<SharedString>) -> Self {
        Self { legend: legend.into(), description: None, content: None }
    }

    pub(crate) fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub(crate) fn content(mut self, content: impl Into<AnyElement>) -> Self {
        self.content = Some(content.into());
        self
    }

    pub(crate) fn build(self) -> AnyElement {
        let mut section = div().flex().flex_col().gap(px(8.0));
        if !self.legend.is_empty() {
            section = section.child(
                div()
                    .text_size(px(20.0))
                    .line_height(px(28.0))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child(self.legend),
            );
        }
        if let Some(description) = self.description {
            section = section.child(
                div()
                    .text_size(px(14.0))
                    .line_height(px(20.0))
                    .text_color(gpui::hsla(0.0, 0.0, 0.5, 1.0))
                    .child(description),
            );
        }
        if let Some(content) = self.content {
            section = section.child(div().mt(px(12.0)).child(content));
        }
        section.into_any_element()
    }
}

pub(crate) fn field_separator() -> AnyElement {
    div().w_full().h(px(1.0)).bg(gpui::hsla(0.0, 0.0, 0.82, 1.0)).into_any_element()
}

impl FieldFormControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("field-form").expect("field-form catalog entry");
        let preview = cx.new(|cx| FieldFormPreview::new(look.clone(), cx));
        Self { look, entry, preview }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }
    pub fn fills_viewport(&self) -> bool {
        true
    }
    pub fn request_layout_refresh(&mut self, _cx: &mut Context<Self>) {}
    pub fn set_viewport_size(&mut self, _size: gpui::Size<gpui::Pixels>, _cx: &mut Context<Self>) {}

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.preview.update(cx, |preview, cx| preview.sync_look(look, cx));
        cx.notify();
    }
}

impl FieldFormPreview {
    fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let required = Arc::new(|value: &str| !value.is_empty());
        let name = look
            .textfield("field-form-name")
            .placeholder("Evil Rabbit")
            .validator(required.clone())
            .full_width(true)
            .spawn(cx);
        let card_number = look
            .textfield("field-form-card-number")
            .placeholder("1234 5678 9012 3456")
            .validator(required.clone())
            .full_width(true)
            .spawn(cx);
        let month = look
            .selector("field-form-month")
            .label("MM")
            .items([
                SelectorItem::new("01").label("01"),
                SelectorItem::new("02").label("02"),
                SelectorItem::new("03").label("03"),
                SelectorItem::new("04").label("04"),
                SelectorItem::new("05").label("05"),
                SelectorItem::new("06").label("06"),
                SelectorItem::new("07").label("07"),
                SelectorItem::new("08").label("08"),
                SelectorItem::new("09").label("09"),
                SelectorItem::new("10").label("10"),
                SelectorItem::new("11").label("11"),
                SelectorItem::new("12").label("12"),
            ])
            .spawn(cx);
        let year = look
            .selector("field-form-year")
            .label("YYYY")
            .items([
                SelectorItem::new("2026").label("2026"),
                SelectorItem::new("2027").label("2027"),
                SelectorItem::new("2028").label("2028"),
                SelectorItem::new("2029").label("2029"),
                SelectorItem::new("2030").label("2030"),
            ])
            .spawn(cx);
        let cvv = look.textfield("field-form-cvv").placeholder("123").validator(required).full_width(true).spawn(cx);
        let same_as_shipping = look
            .checkbox("field-form-same-as-shipping")
            .with_data(false)
            .content(|_, _| div().child("Same as shipping address").into_any_element())
            .spawn(cx);
        let comments = look
            .textarea("field-form-comments")
            .placeholder("Add any additional comments")
            .rows(3)
            .full_width(true)
            .spawn(cx);
        let submit_button = look.primary_button("field-form-submit").label("Submit").spawn(cx);
        let cancel_button = look.outline_button("field-form-cancel").label("Cancel").spawn(cx);

        cx.subscribe(&submit_button, {
            let name = name.clone();
            let card_number = card_number.clone();
            let month = month.clone();
            let year = year.clone();
            let cvv = cvv.clone();
            move |this, _, event: &ButtonEvent, cx| {
                if !event.is_click() {
                    return;
                }
                let required_fields_present = !name.read(cx).value().is_empty()
                    && !card_number.read(cx).value().is_empty()
                    && month.read(cx).selected_id().is_some()
                    && year.read(cx).selected_id().is_some()
                    && !cvv.read(cx).value().is_empty();
                this.status = if required_fields_present {
                    "Payment method submitted".into()
                } else {
                    "Please complete the required fields".into()
                };
                cx.notify();
            }
        })
        .detach();
        cx.subscribe(&cancel_button, |this, _, event: &ButtonEvent, cx| {
            if !event.is_click() {
                return;
            }
            this.name.update(cx, |field, cx| field.set_value("", cx));
            this.card_number.update(cx, |field, cx| field.set_value("", cx));
            this.cvv.update(cx, |field, cx| field.set_value("", cx));
            this.comments.update(cx, |area, cx| area.set_value("", cx));
            this.status = "Changes cancelled".into();
            cx.notify();
        })
        .detach();

        Self {
            look,
            name,
            card_number,
            month,
            year,
            cvv,
            same_as_shipping,
            comments,
            submit_button,
            cancel_button,
            status: "".into(),
        }
    }

    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.name.update(cx, |field, cx| field.set_template(look.textfield_template(), cx));
        self.card_number.update(cx, |field, cx| field.set_template(look.textfield_template(), cx));
        self.month.update(cx, |selector, cx| selector.set_template(look.selector_template(), cx));
        self.year.update(cx, |selector, cx| selector.set_template(look.selector_template(), cx));
        self.cvv.update(cx, |field, cx| field.set_template(look.textfield_template(), cx));
        self.same_as_shipping.update(cx, |checkbox, cx| {
            checkbox.set_template(look.checkbox_template(ShadcnButtonStyle::ContentOnly), cx)
        });
        self.comments.update(cx, |area, cx| area.set_template(look.textarea_template(), cx));
        self.submit_button
            .update(cx, |button, cx| button.set_template(look.button_template(ShadcnButtonStyle::Primary), cx));
        self.cancel_button
            .update(cx, |button, cx| button.set_template(look.button_template(ShadcnButtonStyle::Outline), cx));
        cx.notify();
    }
}

impl Render for FieldFormPreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        with_look(&self.look, || {
            let payment = FieldSet::new("Payment Method")
                .description("All transactions are secure and encrypted")
                .content(
                    FieldGroup::new()
                        .child(Field::new("Name on Card", self.name.clone().into_any_element()).required(true).build())
                        .child(
                            Field::new("Card Number", self.card_number.clone().into_any_element())
                                .required(true)
                                .description("Enter your 16-digit card number")
                                .build(),
                        )
                        .child(
                            div()
                                .grid()
                                .grid_cols(3)
                                .gap(px(16.0))
                                .children([
                                    Field::new("Month", self.month.clone().into_any_element()).required(true).build(),
                                    Field::new("Year", self.year.clone().into_any_element()).required(true).build(),
                                    Field::new("CVV", self.cvv.clone().into_any_element()).required(true).build(),
                                ])
                                .into_any_element(),
                        )
                        .build(),
                )
                .build();
            let billing = FieldSet::new("Billing Address")
                .description("The billing address associated with your payment method")
                .content(FieldGroup::new().child(self.same_as_shipping.clone().into_any_element()).build())
                .build();
            let comments = FieldSet::new("")
                .content(
                    FieldGroup::new()
                        .child(Field::new("Comments", self.comments.clone().into_any_element()).build())
                        .build(),
                )
                .build();

            div()
                .w_full()
                .max_w(px(620.0))
                .flex()
                .flex_col()
                .gap(px(24.0))
                .child(payment)
                .child(field_separator())
                .child(billing)
                .child(comments)
                .child(div().flex().gap(px(12.0)).child(self.submit_button.clone()).child(self.cancel_button.clone()))
                .when(!self.status.is_empty(), |form| {
                    form.child(
                        div().text_size(px(13.0)).text_color(gpui::hsla(0.0, 0.0, 0.5, 1.0)).child(self.status.clone()),
                    )
                })
        })
    }
}

impl Render for FieldFormControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        with_look(&self.look, || {
            render_control_exposition_card(
                &self.look,
                self.entry,
                self.preview.clone().into_any_element(),
                None,
                ControlExpositionLayout::BORDERLESS,
            )
        })
    }
}
