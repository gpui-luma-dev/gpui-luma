use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::menu_item::MenuItem;
use gpui_luma::controls::popup_menu::{PopupMenu, PopupMenuEvent, PopupMenuPlacement};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_icons::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::popup_menu_inspector_adapter::{PopupMenuInspectorAdapter, POPUP_MENU_INSPECTOR_SPEC};
use super::public_interface::render_exposition_doc_sections;
use super::standalone_theme_inspectors::PopupMenuThemeInspector;
use super::template::render_control_exposition_card;

const EVENT_SPECS: &[EventReferenceSpec] = &[
    EventReferenceSpec {
        event: "PopupMenuEvent::Select { item_id, label }",
        trigger: "User activates a menu item",
        notes: "Leaf selection; nested submenus open without Select.",
    },
    EventReferenceSpec {
        event: "PopupMenuEvent::OpenChanged { open }",
        trigger: "Trigger click opens or dismiss closes",
        notes: "Track popup visibility for layout and focus.",
    },
    EventReferenceSpec {
        event: "PopupMenuEvent::Dismiss",
        trigger: "Click-away or Escape while open",
        notes: "Closed without a selection.",
    },
    EventReferenceSpec {
        event: "PopupMenuEvent::FocusChanged { focused }",
        trigger: "Focus enters or leaves the trigger",
        notes: "Trigger focus while the menu is closed.",
    },
    EventReferenceSpec {
        event: "PopupMenuEvent::HoverChanged { hovered }",
        trigger: "Pointer enters or leaves the enabled trigger",
        notes: "Trigger hover only.",
    },
    EventReferenceSpec {
        event: "PopupMenuEvent::EnabledChanged { enabled }",
        trigger: "PopupMenu::set_enabled",
        notes: "Disabling closes an open menu.",
    },
    EventReferenceSpec {
        event: "(none)",
        trigger: "Disabled interaction",
        notes: "Trigger clicks ignored while disabled.",
    },
];

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "PopupMenu",
        surface: "Type",
        notes: "Entity<PopupMenu> — labeled trigger with anchored popup menu.",
    },
    PublicInterfaceSpec {
        symbol: "PopupMenuEvent",
        surface: "Event",
        notes: "Non-exhaustive enum: Select, OpenChanged, Dismiss, FocusChanged, HoverChanged, EnabledChanged.",
    },
    PublicInterfaceSpec { symbol: "PopupMenu::new(id)", surface: "Factory", notes: "Starts a PopupMenuBuilder." },
    PublicInterfaceSpec {
        symbol: "PopupMenuBuilder::label / items / placement",
        surface: "Builder",
        notes: "Trigger label, MenuItem tree, and PopupMenuPlacement anchor strategy.",
    },
    PublicInterfaceSpec {
        symbol: "PopupMenuBuilder::ghost / spawn(cx)",
        surface: "Builder",
        notes: "Ghost trigger style and entity materialization.",
    },
    PublicInterfaceSpec {
        symbol: "look.popup_menu(id)",
        surface: "Look",
        notes: "ShadcnLookControlExt factory with themed popup-menu template.",
    },
];

pub struct PopupMenuControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<PopupMenuExpositionLeftPane>,
    theme_inspector: Entity<PopupMenuThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct PopupMenuExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    preview_smart: Entity<PopupMenu>,
    preview_below: Entity<PopupMenu>,
    event_stream: Entity<ControlEventStream>,
}

impl PopupMenuExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.preview_smart.update(cx, |_, cx| cx.notify());
        self.preview_below.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for PopupMenuExpositionLeftPane {
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

            div()
                .id("controls-doc-popup-menu-left-pane")
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

impl PopupMenuControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("popup-menu").expect("popup-menu catalog entry");
        let preview_smart = look
            .popup_menu("controls-doc-popup-menu-smart")
            .label("Smart popup")
            .items(popup_menu_items())
            .placement(PopupMenuPlacement::Smart)
            .spawn(cx);
        let preview_below = look
            .popup_menu("controls-doc-popup-menu-below")
            .label("Below popup")
            .items(popup_menu_items())
            .placement(PopupMenuPlacement::BelowStart)
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-popup-menu-event-log",
                "Open either popup and choose items; PopupMenuEvent variants appear below.",
            )
        });
        let left_pane = cx.new(|_| PopupMenuExpositionLeftPane {
            look: look.clone(),
            entry,
            preview_smart: preview_smart.clone(),
            preview_below: preview_below.clone(),
            event_stream: event_stream.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-popup-menu-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &POPUP_MENU_INSPECTOR_SPEC,
            PopupMenuInspectorAdapter::shared(),
        );

        let mut subscriptions = Vec::new();
        for preview in [&preview_smart, &preview_below] {
            let event_stream = event_stream.clone();
            subscriptions.push(cx.subscribe(preview, move |_, _, event: &PopupMenuEvent, cx| {
                let line = format_popup_menu_event(event);
                event_stream.update(cx, |stream, cx| {
                    stream.append_line(&line, cx);
                    cx.notify();
                });
            }));
        }

        Self { look, entry, left_pane, theme_inspector, inspector_split, _subscriptions: subscriptions }
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

impl Render for PopupMenuControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-popup-menu-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn popup_menu_items() -> [MenuItem; 5] {
    [
        MenuItem::new("new").label("New file").icon(LucideIcon::FilePlus),
        MenuItem::new("rename").label("Rename").icon(LucideIcon::Pencil),
        MenuItem::new("archive").label("Archive"),
        MenuItem::new("share").label("Share").icon(LucideIcon::Share2).submenu([
            MenuItem::new("copy-link").label("Copy link").icon(LucideIcon::Link),
            MenuItem::new("email").label("Email").icon(LucideIcon::Mail),
        ]),
        MenuItem::new("disabled").label("Unavailable").icon(LucideIcon::ArchiveX).enabled(false),
    ]
}

fn format_popup_menu_event(event: &PopupMenuEvent) -> String {
    match event {
        PopupMenuEvent::Select { item_id, label } => {
            format!("PopupMenuEvent::Select {{ item_id: \"{item_id}\", label: \"{label}\" }}")
        }
        PopupMenuEvent::OpenChanged { open } => format!("PopupMenuEvent::OpenChanged {{ open: {open} }}"),
        PopupMenuEvent::Dismiss => "PopupMenuEvent::Dismiss".to_string(),
        PopupMenuEvent::FocusChanged { focused } => {
            format!("PopupMenuEvent::FocusChanged {{ focused: {focused} }}")
        }
        PopupMenuEvent::HoverChanged { hovered } => {
            format!("PopupMenuEvent::HoverChanged {{ hovered: {hovered} }}")
        }
        PopupMenuEvent::EnabledChanged { enabled } => {
            format!("PopupMenuEvent::EnabledChanged {{ enabled: {enabled} }}")
        }
        _ => "PopupMenuEvent::(unknown)".to_string(),
    }
}
