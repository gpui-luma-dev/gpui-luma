//! ListBox control exposition — single, horizontal, and multiple selection samples.

use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::control_group::ControlGroupEvent;
use gpui_luma::controls::listbox::{ListBox, ListBoxItem};
use gpui_luma::{hstack, vstack};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::public_interface::render_exposition_doc_sections;
use super::template::render_control_exposition_card;

const EVENT_SPECS: &[EventReferenceSpec] = &[
    EventReferenceSpec {
        event: "ControlGroupEvent::Change { changed_id, selected, selected_ids }",
        trigger: "Pointer or keyboard toggles a row",
        notes: "ListBox uses control_group selection semantics.",
    },
    EventReferenceSpec {
        event: "ControlGroupEvent::Activate { activated_id }",
        trigger: "Keyboard activate on focused row",
        notes: "Emitted alongside Change for activation gestures.",
    },
    EventReferenceSpec {
        event: "ControlGroupEvent::ItemFocused { item_id }",
        trigger: "Roving focus moves to a row",
        notes: "Useful for screen reader and form coordination.",
    },
];

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "ListBox",
        surface: "Type",
        notes: "Entity<ListBoxControl> — vertical or horizontal selectable lists.",
    },
    PublicInterfaceSpec {
        symbol: "look.listbox(id) / listbox_multiple(id)",
        surface: "Look",
        notes: "Single or multi-select factories; .horizontal() for inline layout.",
    },
    PublicInterfaceSpec {
        symbol: "ListBoxItem::new / label",
        surface: "Model",
        notes: "Row descriptors bound to control_group item templates.",
    },
];

pub struct ListBoxControlExposition {
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

impl ListBoxControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("listbox").expect("listbox catalog entry");

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

        let mut this = Self {
            look,
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

        this._subscriptions.push(cx.subscribe(&single, {
            let event_stream = event_stream.clone();
            move |this, _, event, cx| {
                this.handle_single_event(event, cx);
                append_listbox_event(&event_stream, event, cx);
            }
        }));
        this._subscriptions.push(cx.subscribe(&horizontal, {
            let event_stream = event_stream.clone();
            move |this, _, event, cx| {
                this.handle_horizontal_event(event, cx);
                append_listbox_event(&event_stream, event, cx);
            }
        }));
        this._subscriptions.push(cx.subscribe(&multiple, {
            let event_stream = event_stream.clone();
            move |this, _, event, cx| {
                this.handle_multi_event(event, cx);
                append_listbox_event(&event_stream, event, cx);
            }
        }));

        this
    }

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

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.single.update(cx, |_, cx| cx.notify());
        self.horizontal.update(cx, |_, cx| cx.notify());
        self.multiple.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for ListBoxControlExposition {
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

            render_control_exposition_card(
                look,
                self.entry,
                preview.into_any_element(),
                Some(render_exposition_doc_sections(look, EVENT_SPECS, PUBLIC_INTERFACE_SPECS)),
                ControlExpositionLayout::BORDERLESS,
            )
        })
    }
}

fn append_listbox_event(
    event_stream: &Entity<ControlEventStream>,
    event: &ControlGroupEvent,
    cx: &mut Context<ListBoxControlExposition>,
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
    cx: &mut Context<ListBoxControlExposition>,
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
