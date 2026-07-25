use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::selector::{Selector, SelectorEvent, SelectorItem, SelectorPlacement};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_icons::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::public_interface::render_exposition_doc_sections;
use super::template::render_control_exposition_card;

const EVENT_SPECS: &[EventReferenceSpec] = &[
    EventReferenceSpec {
        event: "SelectorEvent::Change { item_id, label }",
        trigger: "User picks an item",
        notes: "Commits selection and updates trigger label.",
    },
    EventReferenceSpec {
        event: "SelectorEvent::OpenChanged { open }",
        trigger: "Trigger click opens or dismiss closes",
        notes: "Track popup visibility.",
    },
    EventReferenceSpec {
        event: "SelectorEvent::Dismiss",
        trigger: "Click-away or Escape while open",
        notes: "Closed without a selection change.",
    },
    EventReferenceSpec {
        event: "SelectorEvent::FocusChanged { focused }",
        trigger: "Focus enters or leaves the trigger",
        notes: "Trigger focus while closed.",
    },
    EventReferenceSpec {
        event: "(none)",
        trigger: "Disabled interaction",
        notes: "Trigger clicks ignored while disabled.",
    },
];

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "Selector",
        surface: "Type",
        notes: "Entity<Selector> — labeled trigger with anchored item popup.",
    },
    PublicInterfaceSpec {
        symbol: "SelectorEvent",
        surface: "Event",
        notes: "Change, OpenChanged, Dismiss, FocusChanged.",
    },
    PublicInterfaceSpec { symbol: "look.selector(id)", surface: "Look", notes: "ShadcnLookControlExt factory." },
    PublicInterfaceSpec {
        symbol: "SelectorBuilder::label / items / placement",
        surface: "Builder",
        notes: "Trigger label, SelectorItem list, and SelectorPlacement.",
    },
    PublicInterfaceSpec {
        symbol: "SelectorBuilder::with_item_template / spawn(cx)",
        surface: "Builder",
        notes: "Custom row renderer and entity materialization.",
    },
];

pub struct PopupSelectorControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    preview_below: Entity<Selector>,
    preview_smart: Entity<Selector>,
    event_stream: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}

impl PopupSelectorControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("popup-selector").expect("popup-selector catalog entry");
        let preview_below = look
            .selector("controls-doc-selector-below")
            .label("Below selector")
            .items(selector_items())
            .placement(SelectorPlacement::BelowStart)
            .spawn(cx);
        let preview_smart = look
            .selector("controls-doc-selector-smart")
            .label("Smart selector")
            .items(selector_items())
            .placement(SelectorPlacement::Smart)
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-popup-selector-event-log",
                "Open a selector and pick items; SelectorEvent variants appear below.",
            )
        });

        let mut subscriptions = Vec::new();
        for preview in [&preview_below, &preview_smart] {
            let event_stream = event_stream.clone();
            subscriptions.push(cx.subscribe(preview, move |_, _, event: &SelectorEvent, cx| {
                let line = format_selector_event(event);
                event_stream.update(cx, |stream, cx| {
                    stream.append_line(&line, cx);
                    cx.notify();
                });
            }));
        }

        Self { look, entry, preview_below, preview_smart, event_stream, _subscriptions: subscriptions }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.preview_below.update(cx, |_, cx| cx.notify());
        self.preview_smart.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for PopupSelectorControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(16.0))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .justify_center()
                        .gap(px(12.0))
                        .child(self.preview_below.clone())
                        .child(self.preview_smart.clone()),
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

fn selector_items() -> [SelectorItem; 4] {
    [
        SelectorItem::new("new").label("New").icon(LucideIcon::FilePlus),
        SelectorItem::new("open").label("Open").icon(LucideIcon::FolderOpen),
        SelectorItem::new("archive").label("Archive").icon(LucideIcon::Archive),
        SelectorItem::new("export").label("Export").icon(LucideIcon::Share2),
    ]
}

fn format_selector_event(event: &SelectorEvent) -> String {
    match event {
        SelectorEvent::Change { item_id, label } => {
            format!("SelectorEvent::Change {{ item_id: \"{item_id}\", label: \"{label}\" }}")
        }
        SelectorEvent::OpenChanged { open } => format!("SelectorEvent::OpenChanged {{ open: {open} }}"),
        SelectorEvent::Dismiss => "SelectorEvent::Dismiss".to_string(),
        SelectorEvent::FocusChanged { focused } => format!("SelectorEvent::FocusChanged {{ focused: {focused} }}"),
        _ => "SelectorEvent::(unknown)".to_string(),
    }
}
