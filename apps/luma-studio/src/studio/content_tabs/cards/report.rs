use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, prelude::*};
use luma::infra::presenter::HasPresenter;
use luma::controls::button::Button;
use luma::controls::selector::{Selector, SelectorItem};
use luma::controls::textarea::TextArea;
use luma::controls::textfield::TextField;
use luma_look_shadcn::prelude::*;
use luma::theme::{ControlSize};
use luma_look_shadcn::ShadcnLook;
use luma::{declare_form, form_field, hstack, vstack};

use super::common::titled_card;

declare_form! {
    pub struct ReportPanel {
        controls: {
            area_selector: Entity<Selector> = look
                .selector("report-area")
                .label("Billing")
                .items(area_items()),
            security_selector: Entity<Selector> = look
                .selector("report-security")
                .label("Severity 2")
                .items(security_items()),
            subject_field: TextField = look.textfield("report-subject").primary(&look).placeholder("Subject").full_width(true),
            description_area: Entity<TextArea> = look
                .textarea("report-description")
                .primary(&look)
                .placeholder("Description")
                .full_width(true)
                .rows(4),
            cancel_button: Entity<Button> = look.ghost_button("report-cancel").label("Cancel").size(size),
            submit_button: Entity<Button> = look.primary_button("report-submit").label("Submit").size(size),
        },
        args: {
            look: Arc<ShadcnLook>,
            size: ControlSize,
        },
        fields: {}
    }
}

impl Render for ReportPanel {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let area_selector = self.area_selector.clone();
        let security_selector = self.security_selector.clone();
        let subject_field = self.subject_field.clone();
        let description_area = self.description_area.clone();
        let cancel_button = self.cancel_button.clone();
        let submit_button = self.submit_button.clone();

        titled_card(
            "luma-studio-report-card",
            &self.look,
            380.0,
            "Report an issue",
            "What area are you having problems with?",
            move |_, _| {
                vstack! {
                    gap=12;
                    hstack! {
                        gap=10;
                        form_field!("Area", chrome; area_selector.clone()).flex_1(),
                        form_field!("Security Level", chrome; security_selector.clone()).flex_1(),
                    },
                    form_field!("Subject", chrome; subject_field.clone()),
                    form_field!("Description", chrome; description_area.clone()),
                    hstack! {
                        gap=8 justify=end;
                        cancel_button.clone(),
                        submit_button.clone(),
                    },
                }
                .into_any_element()
            },
            window,
            _cx,
        )
    }
}

fn area_items() -> Vec<SelectorItem> {
    vec![
        SelectorItem::new("billing").label("Billing"),
        SelectorItem::new("teams").label("Teams"),
        SelectorItem::new("account").label("Account"),
    ]
}

fn security_items() -> Vec<SelectorItem> {
    vec![
        SelectorItem::new("sev1").label("Severity 1"),
        SelectorItem::new("sev2").label("Severity 2"),
        SelectorItem::new("sev3").label("Severity 3"),
    ]
}
