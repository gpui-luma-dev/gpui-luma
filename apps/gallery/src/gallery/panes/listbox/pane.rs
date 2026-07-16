use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, FontWeight, Subscription, div, prelude::*, px};
use gpui_luma::controls::control_group::ControlGroupEvent;
use gpui_luma::controls::listbox::{ListBox, ListBoxItem};
use gpui_luma::{hstack, vstack};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;

use super::inspector_tree::build_listbox_inspect_tree;
use super::super::shared::inspector::{ColorInspectorShell, spawn_color_inspector_tree};
use super::super::shared::{gallery_pane_with_inspector, notify_entity, InspectorToggleRegistry};

#[derive(Clone)]
pub(in crate::gallery) struct ListBoxPane {
    single: ListBox,
    horizontal: ListBox,
    multiple: ListBox,
    single_choice: String,
    horizontal_choice: String,
    multi_choices: Vec<String>,
    inspector: Entity<ColorInspectorShell>,
}

impl ListBoxPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let tree = spawn_color_inspector_tree("listbox-inspector-tree", look.clone(), build_listbox_inspect_tree, cx);
        let inspector = cx.new(|cx| {
            ColorInspectorShell::new(
                look.clone(),
                tree,
                "listbox-inspector",
                "listbox-inspector-split",
                "listbox-inspector-detail",
                build_listbox_inspect_tree,
                cx,
            )
        });

        let single = fruit_listbox(&look, "listbox-fruit-single", ListBoxMode::Single, ["oranges"], cx);
        let horizontal =
            fruit_listbox(&look, "listbox-fruit-horizontal", ListBoxMode::HorizontalSingle, ["oranges"], cx);
        let multiple = fruit_listbox(&look, "listbox-fruit-multiple", ListBoxMode::Multiple, ["apples", "bananas"], cx);

        Self {
            single,
            horizontal,
            multiple,
            single_choice: "Oranges".to_string(),
            horizontal_choice: "Oranges".to_string(),
            multi_choices: vec!["Apples".to_string(), "Bananas".to_string()],
            inspector,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.single, |app, _, event: &ControlGroupEvent, cx| {
            app.panes.listbox.handle_single_event(event, cx);
        }));

        subscriptions.push(cx.subscribe(&self.horizontal, |app, _, event: &ControlGroupEvent, cx| {
            app.panes.listbox.handle_horizontal_event(event, cx);
        }));

        subscriptions.push(cx.subscribe(&self.multiple, |app, _, event: &ControlGroupEvent, cx| {
            app.panes.listbox.handle_multi_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook, toggles: &InspectorToggleRegistry) -> AnyElement {
        let chrome = look.chrome();
        let section_style = look.typography_scale(ShadcnTextSize::Sm);
        let status_style = look.typography_scale(ShadcnTextSize::Sm);

        gallery_pane_with_inspector(
            "listbox",
            "ListBox",
            vstack! {
                gap=16.0;
                render_listbox_sample(
                    "Single select",
                    self.single.clone().into_any_element(),
                    format!("Selected: {}", self.single_choice),
                    section_style,
                    status_style,
                    chrome.muted_text,
                    chrome.body_text,
                ),
                render_listbox_sample(
                    "Horizontal single select",
                    self.horizontal.clone().into_any_element(),
                    format!("Selected: {}", self.horizontal_choice),
                    section_style,
                    status_style,
                    chrome.muted_text,
                    chrome.body_text,
                ),
                render_listbox_sample(
                    "Multiple select",
                    self.multiple.clone().into_any_element(),
                    format!("Selected: {}", self.multi_choices.join(", ")),
                    section_style,
                    status_style,
                    chrome.muted_text,
                    chrome.body_text,
                ),
            }
            .w(px(420.0))
            .into_any_element(),
            self.inspector.clone(),
            toggles,
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.single, cx);
        notify_entity(&self.horizontal, cx);
        notify_entity(&self.multiple, cx);
        notify_entity(&self.inspector, cx);
        notify_entity(&self.inspector.read(cx).tree(), cx);
        notify_entity(&self.inspector.read(cx).detail(), cx);
        notify_entity(&self.inspector.read(cx).split(), cx);
    }

    fn handle_single_event(&mut self, event: &ControlGroupEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ControlGroupEvent::Change { changed_id, .. } => {
                self.single_choice = fruit_label(changed_id.as_ref());
                cx.notify();
            }
            ControlGroupEvent::Activate { .. } => {}
        }
    }

    fn handle_horizontal_event(&mut self, event: &ControlGroupEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ControlGroupEvent::Change { changed_id, .. } => {
                self.horizontal_choice = fruit_label(changed_id.as_ref());
                cx.notify();
            }
            ControlGroupEvent::Activate { .. } => {}
        }
    }

    fn handle_multi_event(&mut self, event: &ControlGroupEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ControlGroupEvent::Change { selected_ids, .. } => {
                self.multi_choices = selected_ids.iter().map(|id| fruit_label(id.as_ref())).collect();
                if self.multi_choices.is_empty() {
                    self.multi_choices.push("None".to_string());
                }
                cx.notify();
            }
            ControlGroupEvent::Activate { .. } => {}
        }
    }
}

#[derive(Clone, Copy)]
enum ListBoxMode {
    Single,
    HorizontalSingle,
    Multiple,
}

fn fruit_listbox(
    look: &Arc<ShadcnLook>,
    id: &'static str,
    mode: ListBoxMode,
    selected_ids: impl IntoIterator<Item = &'static str>,
    cx: &mut Context<GalleryApp>,
) -> ListBox {
    let mut builder = match mode {
        ListBoxMode::Single | ListBoxMode::HorizontalSingle => look.listbox(id),
        ListBoxMode::Multiple => look.listbox_multiple(id),
    };

    if matches!(mode, ListBoxMode::HorizontalSingle) {
        builder = builder.horizontal();
    }

    builder.items(fruit_items()).selected_ids(selected_ids).spawn(cx)
}

fn render_listbox_sample(
    label: &'static str,
    listbox: AnyElement,
    status: String,
    section_style: gpui_luma::theme::LumaTextStyle,
    status_style: gpui_luma::theme::LumaTextStyle,
    label_color: gpui::Hsla,
    status_color: gpui::Hsla,
) -> AnyElement {
    vstack! {
        gap=8.0;
        hstack! {
            justify=between align=center gap=8.0;
            div()
                .typography_style(section_style)
                .font_weight(FontWeight::MEDIUM)
                .text_color(label_color)
                .child(label),
            div()
                .typography_style(status_style)
                .text_color(status_color)
                .child(status),
        },
        listbox,
    }
    .into_any_element()
}

fn fruit_items() -> Vec<ListBoxItem> {
    vec![
        ListBoxItem::new("apples", "apples").label("Apples"),
        ListBoxItem::new("oranges", "oranges").label("Oranges"),
        ListBoxItem::new("bananas", "bananas").label("Bananas"),
    ]
}

fn fruit_label(id: &str) -> String {
    match id {
        "apples" => "Apples",
        "oranges" => "Oranges",
        "bananas" => "Bananas",
        _ => id,
    }
    .to_string()
}
