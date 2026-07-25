use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::combobox::{ComboBox, ComboBoxEvent, SelectionItem, TypingPolicy};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::public_interface::render_exposition_doc_sections;
use super::template::render_control_exposition_card;

const EVENT_SPECS: &[EventReferenceSpec] = &[
    EventReferenceSpec {
        event: "ComboBoxEvent::Change { query }",
        trigger: "User edits the text field",
        notes: "Filters popup items; strict mode rejects partial matches on commit.",
    },
    EventReferenceSpec {
        event: "ComboBoxEvent::Select { item_id, label }",
        trigger: "User picks an item from the popup",
        notes: "Commits selection and updates field text.",
    },
    EventReferenceSpec {
        event: "ComboBoxEvent::Complete { item_id, label }",
        trigger: "Keyboard complete on highlighted item",
        notes: "Same commit semantics as Select via keyboard.",
    },
    EventReferenceSpec {
        event: "ComboBoxEvent::Clear",
        trigger: "Clear affordance",
        notes: "Resets query and selection.",
    },
    EventReferenceSpec {
        event: "ComboBoxEvent::OpenChanged { open }",
        trigger: "Down arrow or typing opens popup",
        notes: "Track overlay visibility.",
    },
    EventReferenceSpec {
        event: "ComboBoxEvent::Dismiss",
        trigger: "Click-away while open",
        notes: "Closed without commit.",
    },
    EventReferenceSpec {
        event: "ComboBoxEvent::FocusChanged { focused }",
        trigger: "Focus enters or leaves the field",
        notes: "Field focus transitions.",
    },
    EventReferenceSpec { event: "(none)", trigger: "Disabled interaction", notes: "Ignored while disabled." },
];

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "ComboBox",
        surface: "Type",
        notes: "Entity<ComboBoxControl> — editable field with anchored item popup.",
    },
    PublicInterfaceSpec {
        symbol: "ComboBoxEvent",
        surface: "Event",
        notes: "Change, Select, Complete, Clear, OpenChanged, Dismiss, FocusChanged.",
    },
    PublicInterfaceSpec { symbol: "look.combobox(id, items)", surface: "Look", notes: "ShadcnLookControlExt factory." },
    PublicInterfaceSpec {
        symbol: "ComboBoxBuilder::typing_policy / show_down_arrow",
        surface: "Builder",
        notes: "Strict vs permissive typing and chrome affordances.",
    },
    PublicInterfaceSpec {
        symbol: "ComboBoxBuilder::items_template / panel_template",
        surface: "Builder",
        notes: "Second-tier list and popup shell customization.",
    },
    PublicInterfaceSpec {
        symbol: "ComboBoxBuilder::spawn(cx)",
        surface: "Builder",
        notes: "Materialize entity; subscribe for ComboBoxEvent.",
    },
];

pub struct ComboBoxControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    preview: ComboBox,
    event_stream: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}

impl ComboBoxControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("combobox").expect("combobox catalog entry");
        let preview = look
            .combobox("controls-doc-combobox", combobox_demo_items())
            .placeholder("Strict mode (exact match only)…")
            .full_width(true)
            .clean_on_escape(true)
            .typing_policy(TypingPolicy::Strict)
            .show_down_arrow(true)
            .show_clear_button(true)
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-combobox-event-log",
                "Edit the combobox and pick items; ComboBoxEvent variants appear below.",
            )
        });

        let subscription = cx.subscribe(&preview, {
            let event_stream = event_stream.clone();
            move |_, _, event: &ComboBoxEvent, cx| {
                let line = format_combobox_event(event);
                event_stream.update(cx, |stream, cx| {
                    stream.append_line(&line, cx);
                });
            }
        });

        Self { look, entry, preview, event_stream, _subscriptions: vec![subscription] }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.preview.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for ComboBoxControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let preview = div()
                .w_full()
                .flex()
                .items_start()
                .gap(px(20.0))
                .child(
                    div()
                        .w(px(320.0))
                        .flex_none()
                        .flex()
                        .flex_col()
                        .gap(px(16.0))
                        .child(
                            div()
                                .text_size(px(11.0))
                                .line_height(px(15.0))
                                .text_color(chrome.muted_text)
                                .child("Strict typing policy + down arrow"),
                        )
                        .child(self.preview.clone()),
                )
                .child(self.event_stream.clone());

            render_control_exposition_card(
                &self.look,
                self.entry,
                preview.into_any_element(),
                Some(render_exposition_doc_sections(&self.look, EVENT_SPECS, PUBLIC_INTERFACE_SPECS)),
                ControlExpositionLayout::BORDERLESS,
            )
        })
    }
}

fn combobox_demo_items() -> Vec<SelectionItem> {
    vec![
        SelectionItem::new("alabama", "Alabama"),
        SelectionItem::new("alaska", "Alaska"),
        SelectionItem::new("arizona", "Arizona"),
        SelectionItem::new("arkansas", "Arkansas"),
        SelectionItem::new("california", "California"),
        SelectionItem::new("colorado", "Colorado"),
        SelectionItem::new("connecticut", "Connecticut"),
        SelectionItem::new("delaware", "Delaware"),
        SelectionItem::new("florida", "Florida"),
        SelectionItem::new("georgia", "Georgia"),
    ]
}

pub(crate) fn format_combobox_event(event: &ComboBoxEvent) -> String {
    match event {
        ComboBoxEvent::Change { query } => format!("ComboBoxEvent::Change {{ query: \"{query}\" }}"),
        ComboBoxEvent::Select { item_id, label } => {
            format!("ComboBoxEvent::Select {{ item_id: \"{item_id}\", label: \"{label}\" }}")
        }
        ComboBoxEvent::Complete { item_id, label } => {
            format!("ComboBoxEvent::Complete {{ item_id: \"{item_id}\", label: \"{label}\" }}")
        }
        ComboBoxEvent::Clear => "ComboBoxEvent::Clear".to_string(),
        ComboBoxEvent::OpenChanged { open } => format!("ComboBoxEvent::OpenChanged {{ open: {open} }}"),
        ComboBoxEvent::Dismiss => "ComboBoxEvent::Dismiss".to_string(),
        ComboBoxEvent::FocusChanged { focused } => format!("ComboBoxEvent::FocusChanged {{ focused: {focused} }}"),
        _ => "ComboBoxEvent::(unknown)".to_string(),
    }
}
