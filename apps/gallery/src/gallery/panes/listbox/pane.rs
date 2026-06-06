use std::sync::Arc;

use gpui::{AnyElement, Context, Subscription, div, prelude::*, px};
use gpui_luma::controls::control_group::ControlGroupEvent;
use gpui_luma::controls::listbox::{ListBox, ListBoxItem};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;

use super::super::shared::{gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct ListBoxPane {
    single: ListBox,
    multiple: ListBox,
    single_choice: String,
    multi_choices: Vec<String>,
}

impl ListBoxPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let single = look
            .listbox("listbox-density-single")
            .items(density_items())
            .selected("comfortable")
            .spawn(cx);

        let multiple = look
            .listbox_multiple("listbox-density-multiple")
            .items(density_items())
            .selected_ids(["compact", "expanded"])
            .spawn(cx);

        Self {
            single,
            multiple,
            single_choice: "Comfortable".to_string(),
            multi_choices: vec!["Compact".to_string(), "Expanded".to_string()],
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.single, |app, _, event: &ControlGroupEvent, cx| {
            app.panes.listbox.handle_single_event(event, cx);
        }));

        subscriptions.push(cx.subscribe(&self.multiple, |app, _, event: &ControlGroupEvent, cx| {
            app.panes.listbox.handle_multi_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        let chrome = look.chrome();

        gallery_pane_with_usage(
            "ListBox",
            "ListBox",
            div()
                .w(px(420.0))
                .flex()
                .flex_col()
                .gap_4()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(div().text_size(px(12.0)).line_height(px(16.0)).child("Single select"))
                        .child(self.single.clone())
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.body_text)
                                .child(format!("Selected: {}", self.single_choice)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(div().text_size(px(12.0)).line_height(px(16.0)).child("Multiple select"))
                        .child(self.multiple.clone())
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.body_text)
                                .child(format!("Selected: {}", self.multi_choices.join(", "))),
                        ),
                )
                .into_any_element(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.single, cx);
        notify_entity(&self.multiple, cx);
    }

    fn handle_single_event(&mut self, event: &ControlGroupEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ControlGroupEvent::Change { changed_id, .. } => {
                self.single_choice = density_label(changed_id.as_ref());
                cx.notify();
            }
        }
    }

    fn handle_multi_event(&mut self, event: &ControlGroupEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ControlGroupEvent::Change { selected_ids, .. } => {
                self.multi_choices = selected_ids.iter().map(|id| density_label(id.as_ref())).collect();
                if self.multi_choices.is_empty() {
                    self.multi_choices.push("None".to_string());
                }
                cx.notify();
            }
        }
    }
}

fn density_items() -> Vec<ListBoxItem> {
    vec![
        ListBoxItem::new("compact", "compact").label("Compact"),
        ListBoxItem::new("comfortable", "comfortable").label("Comfortable"),
        ListBoxItem::new("expanded", "expanded").label("Expanded"),
    ]
}

fn density_label(id: &str) -> String {
    match id {
        "compact" => "Compact",
        "comfortable" => "Comfortable",
        "expanded" => "Expanded",
        _ => id,
    }
    .to_string()
}
