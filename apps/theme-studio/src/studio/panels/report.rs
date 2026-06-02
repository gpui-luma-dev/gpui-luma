use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, prelude::*};
use gpui_luma::controls::command::button::Button;
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::selector::{Selector, SelectorItem};
use gpui_luma::controls::textarea::TextArea;
use gpui_luma::controls::textfield::TextField;
use gpui_luma::theme::radix::prelude::*;
use gpui_luma::theme::{ControlSize, RadixTheme};
use gpui_luma::{form_field, hstack, vstack};

use super::common::{card, card_header};

pub struct ReportPanel {
    radix_theme: Arc<RadixTheme>,
    area_selector: Entity<Selector>,
    security_selector: Entity<Selector>,
    subject_field: TextField,
    description_area: Entity<TextArea>,
    cancel_button: Entity<Button>,
    submit_button: Entity<Button>,
}

impl ReportPanel {
    pub fn new(cx: &mut Context<Self>, radix_theme: Arc<RadixTheme>, size: ControlSize) -> Self {
        Self {
            area_selector: radix_theme.selector("report-area").label("Billing").items(area_items()).spawn(cx),
            security_selector: radix_theme
                .selector("report-security")
                .label("Severity 2")
                .items(security_items())
                .spawn(cx),
            subject_field: radix_theme.textfield("report-subject").placeholder("Subject").full_width(true).spawn(cx),
            description_area: radix_theme
                .textarea("report-description")
                .placeholder("Description")
                .full_width(true)
                .rows(4)
                .spawn(cx),
            cancel_button: radix_theme.ghost_button("report-cancel").label("Cancel").size(size).spawn(cx),
            submit_button: radix_theme.primary_button("report-submit").label("Submit").size(size).spawn(cx),
            radix_theme,
        }
    }
}

impl Render for ReportPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.radix_theme.chrome();

        card(
            380.0,
            chrome.border,
            chrome.panel_background,
            vstack! {
                gap=12;
                card_header(
                    "Report an issue",
                    "What area are you having problems with?",
                    chrome.title_text,
                    chrome.muted_text,
                ),
                hstack! {
                    gap=10;
                    form_field!("Area", chrome; self.area_selector.clone()).flex_1(),
                    form_field!("Security Level", chrome; self.security_selector.clone()).flex_1(),
                },
                form_field!("Subject", chrome; self.subject_field.clone()),
                form_field!("Description", chrome; self.description_area.clone()),
                hstack! {
                    gap=8 justify=end;
                    self.cancel_button.clone(),
                    self.submit_button.clone(),
                },
            },
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
