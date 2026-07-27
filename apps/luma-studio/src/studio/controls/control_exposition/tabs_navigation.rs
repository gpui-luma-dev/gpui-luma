//! Tabs navigation control exposition — intrinsic and uniform width examples.

use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::tabs_navigation::{
    TabsNavigation, TabsNavigationEvent, TabsNavigationItem, TabsNavigationWidthMode,
};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::public_interface::render_exposition_doc_sections;
use super::template::render_control_exposition_card;

const EVENT_SPECS: &[EventReferenceSpec] = &[
    EventReferenceSpec {
        event: "TabsNavigationEvent::Activate { tab_id, label }",
        trigger: "Pointer click on an enabled tab",
        notes: "Primary tab selection signal.",
    },
    EventReferenceSpec {
        event: "TabsNavigationEvent::Change { tab_id, label }",
        trigger: "Keyboard roving focus changes active tab",
        notes: "Distinct from Activate for keyboard-driven selection.",
    },
    EventReferenceSpec {
        event: "TabsNavigationEvent::FocusChanged { focused }",
        trigger: "Focus enters or leaves the tab list",
        notes: "Emitted once per effective focus transition.",
    },
];

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "TabsNavigation",
        surface: "Type",
        notes: "Entity<TabsNavigation> — horizontal tab strip with intrinsic or uniform width modes.",
    },
    PublicInterfaceSpec {
        symbol: "look.tabs_navigation(id)",
        surface: "Look",
        notes: "ShadcnLookControlExt factory with items, active, width_mode.",
    },
    PublicInterfaceSpec {
        symbol: "TabsNavigationItem::new / label / enabled",
        surface: "Model",
        notes: "Tab item descriptors with optional accessories.",
    },
];

pub struct TabsNavigationControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    tabs: Entity<TabsNavigation>,
    uniform_tabs: Entity<TabsNavigation>,
    event_stream: Entity<ControlEventStream>,
    active_label: String,
    uniform_active_label: String,
    _subscriptions: Vec<Subscription>,
}

impl TabsNavigationControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("tabs-navigation").expect("tabs-navigation catalog entry");

        let tabs = look.tabs_navigation("controls-doc-tabs").items(project_tabs()).active("activity").spawn(cx);
        let uniform_tabs = look
            .tabs_navigation("controls-doc-tabs-uniform")
            .items(uniform_width_tabs())
            .active("recent-activity")
            .width_mode(TabsNavigationWidthMode::Uniform)
            .spawn(cx);

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-tabs-navigation-event-log",
                "Click tabs or use keyboard navigation; TabsNavigationEvent variants appear below.",
            )
        });

        let mut subscriptions = Vec::new();
        for (entity, label_field) in [(tabs.clone(), "intrinsic"), (uniform_tabs.clone(), "uniform")] {
            let event_stream = event_stream.clone();
            subscriptions.push(cx.subscribe(&entity, move |this, _, event: &TabsNavigationEvent, cx| {
                if let TabsNavigationEvent::Activate { label, .. } = event {
                    match label_field {
                        "intrinsic" => this.active_label = label.to_string(),
                        _ => this.uniform_active_label = label.to_string(),
                    }
                    cx.notify();
                }
                if let Some(line) = format_tabs_navigation_event(event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            }));
        }

        Self {
            look,
            entry,
            tabs,
            uniform_tabs,
            event_stream,
            active_label: "Activity".to_string(),
            uniform_active_label: "Recent Activity".to_string(),
            _subscriptions: subscriptions,
        }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.tabs.update(cx, |_, cx| cx.notify());
        self.uniform_tabs.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for TabsNavigationControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let look = &self.look;

            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(12.0))
                .child(render_tabs_example(
                    look,
                    "Intrinsic width",
                    self.tabs.clone(),
                    &self.active_label,
                    "Each tab keeps its own intrinsic width.",
                    360.0,
                ))
                .child(render_tabs_example(
                    look,
                    "Uniform width (match widest label)",
                    self.uniform_tabs.clone(),
                    &self.uniform_active_label,
                    "The widest label defines the slot width for every tab.",
                    420.0,
                ))
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

fn format_tabs_navigation_event(event: &TabsNavigationEvent) -> Option<String> {
    match event {
        TabsNavigationEvent::Activate { tab_id, label } => {
            Some(format!("TabsNavigationEvent::Activate {{ tab_id: \"{tab_id}\", label: \"{label}\" }}"))
        }
        TabsNavigationEvent::Change { tab_id, label } => {
            Some(format!("TabsNavigationEvent::Change {{ tab_id: \"{tab_id}\", label: \"{label}\" }}"))
        }
        TabsNavigationEvent::FocusChanged { focused } => {
            Some(format!("TabsNavigationEvent::FocusChanged {{ focused: {focused} }}"))
        }
        TabsNavigationEvent::ItemFocused { tab_id, label } => {
            Some(format!("TabsNavigationEvent::ItemFocused {{ tab_id: \"{tab_id}\", label: \"{label}\" }}"))
        }
        TabsNavigationEvent::Reactivate { tab_id, label } => {
            Some(format!("TabsNavigationEvent::Reactivate {{ tab_id: \"{tab_id}\", label: \"{label}\" }}"))
        }
        _ => None,
    }
}

fn render_tabs_example(
    look: &ShadcnLook,
    label: &'static str,
    tabs: Entity<TabsNavigation>,
    active_label: &str,
    detail: &'static str,
    content_width: f32,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(4.0))
        .child(render_example_label(label, look))
        .child(tabs)
        .child(render_tab_content(active_label, detail, content_width, look))
}

fn render_example_label(label: &'static str, look: &ShadcnLook) -> impl IntoElement {
    let chrome = look.chrome();
    div()
        .text_size(px(12.0))
        .line_height(px(16.0))
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(chrome.muted_text)
        .child(label)
}

fn render_tab_content(active_label: &str, detail: &'static str, width: f32, look: &ShadcnLook) -> impl IntoElement {
    let chrome = look.chrome();

    div()
        .w(px(width))
        .min_h(px(112.0))
        .flex()
        .flex_col()
        .gap_2()
        .rounded(px(8.0))
        .border_1()
        .border_color(chrome.border)
        .bg(chrome.panel_background)
        .p(px(16.0))
        .text_color(chrome.body_text)
        .child(
            div()
                .text_size(px(14.0))
                .line_height(px(20.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .child(format!("{active_label} tab")),
        )
        .child(div().text_size(px(13.0)).line_height(px(18.0)).child(detail))
}

fn project_tabs() -> [TabsNavigationItem; 4] {
    [
        TabsNavigationItem::new("overview").label("Overview"),
        TabsNavigationItem::new("activity").label("Activity"),
        TabsNavigationItem::new("metrics").label("Metrics"),
        TabsNavigationItem::new("settings").label("Settings").enabled(false),
    ]
}

fn uniform_width_tabs() -> [TabsNavigationItem; 4] {
    [
        TabsNavigationItem::new("home").label("Home"),
        TabsNavigationItem::new("recent-activity").label("Recent Activity"),
        TabsNavigationItem::new("api-integrations").label("API & Integrations"),
        TabsNavigationItem::new("settings").label("Settings").enabled(false),
    ]
}
