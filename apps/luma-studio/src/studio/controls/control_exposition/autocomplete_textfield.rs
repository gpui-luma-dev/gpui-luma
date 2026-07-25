use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::autocomplete::{AutocompleteTextBox, AutocompleteTextBoxEvent, SelectionItem};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::public_interface::render_exposition_doc_sections;
use super::template::render_control_exposition_card;

const EVENT_SPECS: &[EventReferenceSpec] = &[
    EventReferenceSpec {
        event: "AutocompleteTextBoxEvent::Change { query }",
        trigger: "User edits the text field",
        notes: "Filters popup items; does not commit selection.",
    },
    EventReferenceSpec {
        event: "AutocompleteTextBoxEvent::Select { item_id, label }",
        trigger: "User picks an item from the popup",
        notes: "Commits selection and updates the field value.",
    },
    EventReferenceSpec {
        event: "AutocompleteTextBoxEvent::Complete { item_id, label }",
        trigger: "Keyboard complete on highlighted item",
        notes: "Same commit semantics as Select via keyboard.",
    },
    EventReferenceSpec {
        event: "AutocompleteTextBoxEvent::Clear",
        trigger: "Clear affordance or escape policy",
        notes: "Resets query and selection.",
    },
    EventReferenceSpec {
        event: "AutocompleteTextBoxEvent::OpenChanged { open }",
        trigger: "Popup opens or closes",
        notes: "Track overlay visibility.",
    },
    EventReferenceSpec {
        event: "AutocompleteTextBoxEvent::Dismiss",
        trigger: "Click-away while open",
        notes: "Popup closed without commit.",
    },
    EventReferenceSpec {
        event: "AutocompleteTextBoxEvent::FocusChanged { focused }",
        trigger: "Focus enters or leaves the field",
        notes: "Field focus transitions.",
    },
    EventReferenceSpec {
        event: "(none)",
        trigger: "Disabled interaction",
        notes: "Input and popup interaction ignored while disabled.",
    },
];

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "AutocompleteTextBox",
        surface: "Type",
        notes: "Entity<AutocompleteTextBoxControl> — typeahead text field with popup list.",
    },
    PublicInterfaceSpec {
        symbol: "AutocompleteTextBoxEvent",
        surface: "Event",
        notes: "Change, Select, Complete, Clear, OpenChanged, Dismiss, FocusChanged.",
    },
    PublicInterfaceSpec {
        symbol: "look.autocomplete(id, items)",
        surface: "Look",
        notes: "ShadcnLookControlExt factory with default autocomplete templates.",
    },
    PublicInterfaceSpec {
        symbol: "AutocompleteTextBoxBuilder::placeholder / full_width",
        surface: "Builder",
        notes: "Placeholder copy and width before spawn.",
    },
    PublicInterfaceSpec {
        symbol: "AutocompleteTextBoxBuilder::spawn(cx)",
        surface: "Builder",
        notes: "Materialize entity; subscribe for AutocompleteTextBoxEvent.",
    },
];

pub struct AutocompleteTextFieldControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    preview: AutocompleteTextBox,
    event_stream: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}

impl AutocompleteTextFieldControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("autocomplete-textfield").expect("autocomplete-textfield catalog entry");
        let preview = look
            .autocomplete("controls-doc-autocomplete", autocomplete_demo_items())
            .placeholder("Start typing…")
            .full_width(true)
            .clean_on_escape(true)
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-autocomplete-event-log",
                "Type in the field and pick items; AutocompleteTextBoxEvent variants appear below.",
            )
        });

        let subscription = cx.subscribe(&preview, {
            let event_stream = event_stream.clone();
            move |_, _, event: &AutocompleteTextBoxEvent, cx| {
                let line = format_autocomplete_event(event);
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

impl Render for AutocompleteTextFieldControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let preview = div()
                .w_full()
                .flex()
                .items_start()
                .gap(px(20.0))
                .child(div().w(px(320.0)).flex_none().child(self.preview.clone()))
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

fn autocomplete_demo_items() -> Vec<SelectionItem> {
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

fn format_autocomplete_event(event: &AutocompleteTextBoxEvent) -> String {
    match event {
        AutocompleteTextBoxEvent::Change { query } => {
            format!("AutocompleteTextBoxEvent::Change {{ query: \"{query}\" }}")
        }
        AutocompleteTextBoxEvent::Select { item_id, label } => {
            format!("AutocompleteTextBoxEvent::Select {{ item_id: \"{item_id}\", label: \"{label}\" }}")
        }
        AutocompleteTextBoxEvent::Complete { item_id, label } => {
            format!("AutocompleteTextBoxEvent::Complete {{ item_id: \"{item_id}\", label: \"{label}\" }}")
        }
        AutocompleteTextBoxEvent::Clear => "AutocompleteTextBoxEvent::Clear".to_string(),
        AutocompleteTextBoxEvent::OpenChanged { open } => {
            format!("AutocompleteTextBoxEvent::OpenChanged {{ open: {open} }}")
        }
        AutocompleteTextBoxEvent::Dismiss => "AutocompleteTextBoxEvent::Dismiss".to_string(),
        AutocompleteTextBoxEvent::FocusChanged { focused } => {
            format!("AutocompleteTextBoxEvent::FocusChanged {{ focused: {focused} }}")
        }
        _ => "AutocompleteTextBoxEvent::(unknown)".to_string(),
    }
}
