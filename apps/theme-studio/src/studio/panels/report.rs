use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, prelude::*};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::command::button::Button;
use gpui_luma::controls::selector::{Selector, SelectorItem};
use gpui_luma::controls::textarea::TextArea;
use gpui_luma::controls::textfield::TextField;
use gpui_luma_theme_radix::prelude::*;
use gpui_luma::theme::{ControlSize};
use gpui_luma_theme_radix::RadixTheme;
use gpui_luma::{declare_form, form_field, hstack, vstack};

use super::common::{card, card_header};

declare_form! {
    pub struct ReportPanel {
        controls: {
            area_selector: Entity<Selector> = radix_theme
                .selector("report-area")
                .label("Billing")
                .items(area_items()),
            security_selector: Entity<Selector> = radix_theme
                .selector("report-security")
                .label("Severity 2")
                .items(security_items()),
            subject_field: TextField = radix_theme.textfield("report-subject").placeholder("Subject").full_width(true),
            description_area: Entity<TextArea> = radix_theme
                .textarea("report-description")
                .placeholder("Description")
                .full_width(true)
                .rows(4),
            cancel_button: Entity<Button> = radix_theme.ghost_button("report-cancel").label("Cancel").size(size),
            submit_button: Entity<Button> = radix_theme.primary_button("report-submit").label("Submit").size(size),
        },
        args: {
            radix_theme: Arc<RadixTheme>,
            size: ControlSize,
        },
        fields: {}
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
