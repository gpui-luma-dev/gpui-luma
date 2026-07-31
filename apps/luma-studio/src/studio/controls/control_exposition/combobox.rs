use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::combobox::{ComboBox, ComboBoxEvent, SelectionItem, TypingPolicy};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::combobox_inspector_adapter::{combobox_inspector_adapter, COMBOBOX_INSPECTOR_SPEC};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::input_theme_inspectors::ComboBoxThemeInspector;
use super::inspector_split::InspectorSplitShell;
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
    left_pane: Entity<ComboBoxExpositionLeftPane>,
    theme_inspector: Entity<ComboBoxThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct ComboBoxExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    preview: ComboBox,
    event_stream: Entity<ControlEventStream>,
}

impl ComboBoxExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.preview.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for ComboBoxExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let preview = div()
                .w_full()
                .max_w(px(760.0))
                .flex()
                .flex_col()
                .items_start()
                .gap(px(16.0))
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

            div()
                .id("controls-doc-combobox-left-pane")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .overflow_y_scroll()
                .child(render_control_exposition_card(
                    &self.look,
                    self.entry,
                    preview.into_any_element(),
                    Some(render_exposition_doc_sections(&self.look, EVENT_SPECS, PUBLIC_INTERFACE_SPECS)),
                    ControlExpositionLayout::BORDERLESS,
                ))
        })
    }
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

        let left_pane = cx.new(|_| ComboBoxExpositionLeftPane {
            look: look.clone(),
            entry,
            preview: preview.clone(),
            event_stream: event_stream.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-combobox-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &COMBOBOX_INSPECTOR_SPEC,
            combobox_inspector_adapter(),
        );

        let subscription = cx.subscribe(&preview, {
            let event_stream = event_stream.clone();
            move |_, _, event: &ComboBoxEvent, cx| {
                let line = format_combobox_event(event);
                event_stream.update(cx, |stream, cx| {
                    stream.append_line(&line, cx);
                });
            }
        });

        Self {
            look,
            entry,
            preview,
            event_stream,
            left_pane,
            theme_inspector,
            inspector_split,
            _subscriptions: vec![subscription],
        }
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
        self.preview.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look.clone(), cx));
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        sync_viewport_inspector(look, &self.theme_inspector, &self.inspector_split, cx);
        cx.notify();
    }
}

impl Render for ComboBoxControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-combobox-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
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
