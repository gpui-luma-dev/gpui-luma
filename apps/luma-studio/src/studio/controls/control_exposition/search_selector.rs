use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::search_selector::{SearchSelector, SearchSelectorEvent, SelectionItem};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::public_interface::render_exposition_doc_sections;
use super::template::render_control_exposition_card;

const EVENT_SPECS: &[EventReferenceSpec] = &[
    EventReferenceSpec {
        event: "SearchSelectorEvent::Change { query }",
        trigger: "User edits popup search input",
        notes: "Filters visible items while the popup is open.",
    },
    EventReferenceSpec {
        event: "SearchSelectorEvent::Select { item_id, label }",
        trigger: "User picks an item",
        notes: "Commits selection and updates trigger label.",
    },
    EventReferenceSpec {
        event: "SearchSelectorEvent::Complete { item_id, label }",
        trigger: "Keyboard complete on highlighted item",
        notes: "Same commit semantics as Select via keyboard.",
    },
    EventReferenceSpec { event: "SearchSelectorEvent::Clear", trigger: "Clear affordance", notes: "Resets selection." },
    EventReferenceSpec {
        event: "SearchSelectorEvent::OpenChanged { open }",
        trigger: "Trigger opens or closes popup",
        notes: "Track overlay visibility.",
    },
    EventReferenceSpec {
        event: "SearchSelectorEvent::Dismiss",
        trigger: "Click-away while open",
        notes: "Closed without commit.",
    },
    EventReferenceSpec {
        event: "SearchSelectorEvent::FocusChanged { focused }",
        trigger: "Focus enters or leaves the control",
        notes: "Trigger focus transitions.",
    },
    EventReferenceSpec { event: "(none)", trigger: "Disabled interaction", notes: "Ignored while disabled." },
];

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "SearchSelector",
        surface: "Type",
        notes: "Entity<SearchSelectorControl> — read-only trigger with searchable popup list.",
    },
    PublicInterfaceSpec {
        symbol: "SearchSelectorEvent",
        surface: "Event",
        notes: "Change, Select, Complete, Clear, OpenChanged, Dismiss, FocusChanged.",
    },
    PublicInterfaceSpec {
        symbol: "look.search_selector(id, items)",
        surface: "Look",
        notes: "ShadcnLookControlExt factory.",
    },
    PublicInterfaceSpec {
        symbol: "SearchSelectorBuilder::placeholder / search_placeholder",
        surface: "Builder",
        notes: "Trigger and popup search placeholders.",
    },
    PublicInterfaceSpec {
        symbol: "SearchSelectorBuilder::spawn(cx)",
        surface: "Builder",
        notes: "Materialize entity; subscribe for SearchSelectorEvent.",
    },
];

pub struct SearchSelectorControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    preview: SearchSelector,
    event_stream: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}

impl SearchSelectorControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("search-selector").expect("search-selector catalog entry");
        let preview = look
            .search_selector("controls-doc-search-selector", search_selector_demo_items())
            .placeholder("Choose a state…")
            .search_placeholder("Search states")
            .full_width(true)
            .clean_on_escape(true)
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-search-selector-event-log",
                "Open the selector and search; SearchSelectorEvent variants appear below.",
            )
        });

        let subscription = cx.subscribe(&preview, {
            let event_stream = event_stream.clone();
            move |_, _, event: &SearchSelectorEvent, cx| {
                let line = format_search_selector_event(event);
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

impl Render for SearchSelectorControlExposition {
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

fn search_selector_demo_items() -> Vec<SelectionItem> {
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

fn format_search_selector_event(event: &SearchSelectorEvent) -> String {
    match event {
        SearchSelectorEvent::Change { query } => format!("SearchSelectorEvent::Change {{ query: \"{query}\" }}"),
        SearchSelectorEvent::Select { item_id, label } => {
            format!("SearchSelectorEvent::Select {{ item_id: \"{item_id}\", label: \"{label}\" }}")
        }
        SearchSelectorEvent::Complete { item_id, label } => {
            format!("SearchSelectorEvent::Complete {{ item_id: \"{item_id}\", label: \"{label}\" }}")
        }
        SearchSelectorEvent::Clear => "SearchSelectorEvent::Clear".to_string(),
        SearchSelectorEvent::OpenChanged { open } => format!("SearchSelectorEvent::OpenChanged {{ open: {open} }}"),
        SearchSelectorEvent::Dismiss => "SearchSelectorEvent::Dismiss".to_string(),
        SearchSelectorEvent::FocusChanged { focused } => {
            format!("SearchSelectorEvent::FocusChanged {{ focused: {focused} }}")
        }
        _ => "SearchSelectorEvent::(unknown)".to_string(),
    }
}
