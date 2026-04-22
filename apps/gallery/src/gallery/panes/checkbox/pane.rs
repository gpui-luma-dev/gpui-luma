use gpui::{AnyElement, Context, Entity, IntoElement, Subscription, div, prelude::*};
use gpui_luma::controls::checkbox::{Checkbox, CheckboxEvent};

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct CheckboxPane {
    default_checkbox: Entity<Checkbox>,
    border_checkbox: Entity<Checkbox>,
    disabled_checkbox: Entity<Checkbox>,
    default_checked: bool,
    border_checked: bool,
}

#[derive(Clone, Copy)]
enum CheckboxPresentation {
    Default,
    Border,
}

impl CheckboxPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            default_checkbox: Checkbox::new("checkbox-default")
                .label("As-is")
                .checked(true)
                .template(theme.checkbox_template())
                .spawn(cx),
            border_checkbox: Checkbox::new("checkbox-border")
                .label("Border")
                .template(theme.border_checkbox_template())
                .spawn(cx),
            disabled_checkbox: Checkbox::new("disabled-checkbox")
                .label("Disabled checkbox")
                .checked(true)
                .enabled(false)
                .template(theme.checkbox_template())
                .spawn(cx),
            default_checked: true,
            border_checked: false,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.default_checkbox, |app, _, event: &CheckboxEvent, cx| {
            app.panes.checkbox.handle_event(CheckboxPresentation::Default, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.border_checkbox, |app, _, event: &CheckboxEvent, cx| {
            app.panes.checkbox.handle_event(CheckboxPresentation::Border, event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        gallery_pane_with_usage(
            "Checkbox",
            "Checkbox",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(self.default_checkbox.clone())
                        .child(self.border_checkbox.clone())
                        .child(self.disabled_checkbox.clone()),
                )
                .child(
                    div()
                        .text_color(chrome.body_text)
                        .child(format!("Checked: as-is={}, border={}", self.default_checked, self.border_checked)),
                )
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.default_checkbox, cx);
        notify_entity(&self.border_checkbox, cx);
        notify_entity(&self.disabled_checkbox, cx);
    }

    fn handle_event(
        &mut self,
        presentation: CheckboxPresentation,
        event: &CheckboxEvent,
        cx: &mut Context<GalleryApp>,
    ) {
        match event {
            CheckboxEvent::Change { checked } => {
                match presentation {
                    CheckboxPresentation::Default => {
                        self.default_checked = *checked;
                    }
                    CheckboxPresentation::Border => {
                        self.border_checked = *checked;
                    }
                }
                cx.notify();
            }
        }
    }
}
