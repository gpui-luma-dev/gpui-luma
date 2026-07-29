use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::context_menu::{ContextMenu, ContextMenuEvent};
use gpui_luma::controls::menu_item::MenuItem;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_icons::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::context_menu_inspector_adapter::{ContextMenuInspectorAdapter, CONTEXT_MENU_INSPECTOR_SPEC};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::public_interface::render_exposition_doc_sections;
use super::standalone_theme_inspectors::ContextMenuThemeInspector;
use super::template::render_control_exposition_card;

const EVENT_SPECS: &[EventReferenceSpec] = &[
    EventReferenceSpec {
        event: "ContextMenuEvent::Select { item_id, label }",
        trigger: "User activates a menu item",
        notes: "Emitted when a leaf item is chosen; submenus navigate without Select.",
    },
    EventReferenceSpec {
        event: "ContextMenuEvent::OpenChanged { open }",
        trigger: "Aux-click opens or dismiss closes the menu",
        notes: "Track overlay visibility for focus restoration.",
    },
    EventReferenceSpec {
        event: "ContextMenuEvent::Dismiss",
        trigger: "Click-away or Escape while open",
        notes: "Menu closed without a selection.",
    },
    EventReferenceSpec {
        event: "ContextMenuEvent::FocusChanged { focused }",
        trigger: "Focus enters or leaves the target",
        notes: "Useful for target chrome while the menu is closed.",
    },
    EventReferenceSpec {
        event: "ContextMenuEvent::HoverChanged { hovered }",
        trigger: "Pointer enters or leaves the enabled target",
        notes: "Target hover only; item hover is template-internal.",
    },
    EventReferenceSpec {
        event: "ContextMenuEvent::EnabledChanged { enabled }",
        trigger: "ContextMenu::set_enabled",
        notes: "Disabling closes an open menu and ignores aux-click.",
    },
    EventReferenceSpec {
        event: "(none)",
        trigger: "Disabled interaction",
        notes: "Aux-click and keyboard open are ignored while disabled.",
    },
];

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "ContextMenu",
        surface: "Type",
        notes: "Entity<ContextMenu> — right-click target with anchored floating menu.",
    },
    PublicInterfaceSpec {
        symbol: "ContextMenuEvent",
        surface: "Event",
        notes: "Non-exhaustive enum: Select, OpenChanged, Dismiss, FocusChanged, HoverChanged, EnabledChanged.",
    },
    PublicInterfaceSpec {
        symbol: "ContextMenu::new(id)",
        surface: "Factory",
        notes: "Starts a ContextMenuBuilder with default template.",
    },
    PublicInterfaceSpec {
        symbol: "ContextMenuBuilder::label / items",
        surface: "Builder",
        notes: "Target label and MenuItem tree (supports submenus).",
    },
    PublicInterfaceSpec {
        symbol: "ContextMenuBuilder::template / spawn(cx)",
        surface: "Builder",
        notes: "Custom ContextMenuTemplate hook and entity materialization.",
    },
    PublicInterfaceSpec {
        symbol: "look.context_menu(id)",
        surface: "Look",
        notes: "ShadcnLookControlExt factory with themed context-menu template.",
    },
];

pub struct ContextMenuControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<ContextMenuExpositionLeftPane>,
    theme_inspector: Entity<ContextMenuThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct ContextMenuExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    preview: Entity<ContextMenu>,
    event_stream: Entity<ControlEventStream>,
}

impl ContextMenuExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.preview.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for ContextMenuExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(16.0))
                .child(div().min_h(px(160.0)).flex().items_center().justify_center().child(self.preview.clone()))
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-context-menu-left-pane")
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

impl ContextMenuControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("context-menu").expect("context-menu catalog entry");
        let preview = look
            .context_menu("controls-doc-context-menu")
            .label("Right-click me")
            .items(context_menu_items())
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-context-menu-event-log",
                "Right-click the target and choose items; ContextMenuEvent variants appear below.",
            )
        });
        let left_pane = cx.new(|_| ContextMenuExpositionLeftPane {
            look: look.clone(),
            entry,
            preview: preview.clone(),
            event_stream: event_stream.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-context-menu-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &CONTEXT_MENU_INSPECTOR_SPEC,
            ContextMenuInspectorAdapter::shared(),
        );

        let subscription = cx.subscribe(&preview, {
            let event_stream = event_stream.clone();
            move |_, _, event: &ContextMenuEvent, cx| {
                let line = format_context_menu_event(event);
                event_stream.update(cx, |stream, cx| {
                    stream.append_line(&line, cx);
                });
            }
        });

        Self { look, entry, left_pane, theme_inspector, inspector_split, _subscriptions: vec![subscription] }
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

impl Render for ContextMenuControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-context-menu-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn context_menu_items() -> [MenuItem; 4] {
    [
        MenuItem::new("open").label("Open").icon(LucideIcon::FolderOpen),
        MenuItem::new("copy").label("Copy").icon(LucideIcon::Copy),
        MenuItem::new("inspect").label("Inspect"),
        MenuItem::new("more").label("More").icon(LucideIcon::Ellipsis).submenu([
            MenuItem::new("download").label("Download").icon(LucideIcon::Download),
            MenuItem::new("external").label("Open externally").icon(LucideIcon::ExternalLink),
        ]),
    ]
}

fn format_context_menu_event(event: &ContextMenuEvent) -> String {
    match event {
        ContextMenuEvent::Select { item_id, label } => {
            format!("ContextMenuEvent::Select {{ item_id: \"{item_id}\", label: \"{label}\" }}")
        }
        ContextMenuEvent::OpenChanged { open } => format!("ContextMenuEvent::OpenChanged {{ open: {open} }}"),
        ContextMenuEvent::Dismiss => "ContextMenuEvent::Dismiss".to_string(),
        ContextMenuEvent::FocusChanged { focused } => {
            format!("ContextMenuEvent::FocusChanged {{ focused: {focused} }}")
        }
        ContextMenuEvent::HoverChanged { hovered } => {
            format!("ContextMenuEvent::HoverChanged {{ hovered: {hovered} }}")
        }
        ContextMenuEvent::EnabledChanged { enabled } => {
            format!("ContextMenuEvent::EnabledChanged {{ enabled: {enabled} }}")
        }
        _ => "ContextMenuEvent::(unknown)".to_string(),
    }
}
