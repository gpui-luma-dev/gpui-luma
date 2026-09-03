//! ListBox control exposition — single, horizontal, and multiple selection samples.

use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use luma::controls::control_group::ControlGroupEvent;
use luma::controls::listbox::{ListBox, ListBoxItem};
use luma::{hstack, vstack};
use luma_look_shadcn::prelude::*;
use luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::collection_theme_inspectors::ListBoxThemeInspector;
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::listbox_inspector_adapter::{ListBoxInspectorAdapter, LISTBOX_INSPECTOR_SPEC};
use super::model::{ControlExpositionLayout};
use super::template::render_control_exposition_card;

pub struct ListBoxControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<ListBoxExpositionLeftPane>,
    theme_inspector: Entity<ListBoxThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
}

struct ListBoxExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    single: ListBox,
    horizontal: ListBox,
    multiple: ListBox,
    single_choice: String,
    horizontal_choice: String,
    multi_choices: Vec<String>,
    event_stream: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}

impl ListBoxExpositionLeftPane {
    fn handle_single_event(&mut self, event: &ControlGroupEvent, cx: &mut Context<Self>) {
        if let ControlGroupEvent::Change { changed_id, .. } = event {
            self.single_choice = fruit_label(changed_id.as_ref());
            cx.notify();
        }
    }

    fn handle_horizontal_event(&mut self, event: &ControlGroupEvent, cx: &mut Context<Self>) {
        if let ControlGroupEvent::Change { changed_id, .. } = event {
            self.horizontal_choice = fruit_label(changed_id.as_ref());
            cx.notify();
        }
    }

    fn handle_multi_event(&mut self, event: &ControlGroupEvent, cx: &mut Context<Self>) {
        if let ControlGroupEvent::Change { selected_ids, .. } = event {
            self.multi_choices = selected_ids.iter().map(|id| fruit_label(id.as_ref())).collect();
            if self.multi_choices.is_empty() {
                self.multi_choices.push("None".to_string());
            }
            cx.notify();
        }
    }

    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.single.update(cx, |_, cx| cx.notify());
        self.horizontal.update(cx, |_, cx| cx.notify());
        self.multiple.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for ListBoxExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let look = &self.look;
            let chrome = look.chrome();

            let preview = vstack! {
                gap=16.0 align=start;
                render_listbox_sample(
                    "Single select",
                    self.single.clone(),
                    format!("Selected: {}", self.single_choice),
                    chrome.muted_text,
                    chrome.body_text,
                ),
                render_listbox_sample(
                    "Horizontal single select",
                    self.horizontal.clone(),
                    format!("Selected: {}", self.horizontal_choice),
                    chrome.muted_text,
                    chrome.body_text,
                ),
                render_listbox_sample(
                    "Multiple select",
                    self.multiple.clone(),
                    format!("Selected: {}", self.multi_choices.join(", ")),
                    chrome.muted_text,
                    chrome.body_text,
                ),
            }
            .w_full()
            .max_w(px(420.0))
            .child(self.event_stream.clone());

            div()
                .id("controls-doc-listbox-left-pane")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .overflow_y_scroll()
                .child(render_control_exposition_card(
                    look,
                    self.entry,
                    preview.into_any_element(),
                    None,
                    ControlExpositionLayout::BORDERLESS,
                ))
        })
    }
}

impl ListBoxControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("listbox").expect("listbox catalog entry");

        let left_pane = cx.new(|cx| {
            let single = fruit_listbox(&look, "controls-doc-listbox-single", ListBoxMode::Single, ["oranges"], cx);
            let horizontal =
                fruit_listbox(&look, "controls-doc-listbox-horizontal", ListBoxMode::HorizontalSingle, ["oranges"], cx);
            let multiple =
                fruit_listbox(&look, "controls-doc-listbox-multiple", ListBoxMode::Multiple, ["apples", "bananas"], cx);

            let event_stream = cx.new(|cx| {
                ControlEventStream::new(
                    cx,
                    look.clone(),
                    "controls-listbox-event-log",
                    "Select list rows; ControlGroupEvent::Change appears below.",
                )
            });

            let mut pane = ListBoxExpositionLeftPane {
                look: look.clone(),
                entry,
                single: single.clone(),
                horizontal: horizontal.clone(),
                multiple: multiple.clone(),
                single_choice: "Oranges".to_string(),
                horizontal_choice: "Oranges".to_string(),
                multi_choices: vec!["Apples".to_string(), "Bananas".to_string()],
                event_stream: event_stream.clone(),
                _subscriptions: Vec::new(),
            };

            pane._subscriptions.push(cx.subscribe(&single, {
                let event_stream = event_stream.clone();
                move |pane, _, event, cx| {
                    pane.handle_single_event(event, cx);
                    append_listbox_event(&event_stream, event, cx);
                }
            }));
            pane._subscriptions.push(cx.subscribe(&horizontal, {
                let event_stream = event_stream.clone();
                move |pane, _, event, cx| {
                    pane.handle_horizontal_event(event, cx);
                    append_listbox_event(&event_stream, event, cx);
                }
            }));
            pane._subscriptions.push(cx.subscribe(&multiple, {
                let event_stream = event_stream.clone();
                move |pane, _, event, cx| {
                    pane.handle_multi_event(event, cx);
                    append_listbox_event(&event_stream, event, cx);
                }
            }));

            pane
        });

        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-listbox-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &LISTBOX_INSPECTOR_SPEC,
            ListBoxInspectorAdapter::shared(),
        );

        Self { look, entry, left_pane, theme_inspector, inspector_split }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn fills_viewport(&self) -> bool {
        true
    }

    pub fn request_layout_refresh(&mut self, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.request_layout_refresh(cx));
    }

    pub fn set_viewport_size(&mut self, size: gpui::Size<gpui::Pixels>, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.set_viewport_size(size, cx));
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        sync_viewport_inspector(look, &self.theme_inspector, &self.inspector_split, cx);
        cx.notify();
    }
}

impl Render for ListBoxControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-listbox-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn append_listbox_event(
    event_stream: &Entity<ControlEventStream>,
    event: &ControlGroupEvent,
    cx: &mut Context<ListBoxExpositionLeftPane>,
) {
    if let Some(line) = format_listbox_event(event) {
        event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
    }
}

fn format_listbox_event(event: &ControlGroupEvent) -> Option<String> {
    match event {
        ControlGroupEvent::Change { changed_id, selected, selected_ids } => Some(format!(
            "ControlGroupEvent::Change {{ changed_id: \"{changed_id}\", selected: {selected}, selected_ids: {selected_ids:?} }}"
        )),
        ControlGroupEvent::Activate { activated_id } => {
            Some(format!("ControlGroupEvent::Activate {{ activated_id: \"{activated_id}\" }}"))
        }
        ControlGroupEvent::ItemFocused { item_id } => {
            Some(format!("ControlGroupEvent::ItemFocused {{ item_id: \"{item_id}\" }}"))
        }
        ControlGroupEvent::FocusChanged { focused } => {
            Some(format!("ControlGroupEvent::FocusChanged {{ focused: {focused} }}"))
        }
        _ => None,
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
    cx: &mut Context<ListBoxExpositionLeftPane>,
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
    listbox: ListBox,
    status: String,
    label_color: gpui::Hsla,
    status_color: gpui::Hsla,
) -> impl IntoElement {
    vstack! {
        gap=8.0 align=start;
        hstack! {
            justify=between align=center gap=8.0;
            div()
                .text_size(px(12.0))
                .line_height(px(16.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(label_color)
                .child(label),
            div()
                .text_size(px(12.0))
                .line_height(px(16.0))
                .text_color(status_color)
                .child(status),
        },
        listbox,
    }
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
