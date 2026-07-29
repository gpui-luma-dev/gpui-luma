use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::selection_panel::{SelectionPanelControl, SelectionPanelEvent, SelectionPanelItem};
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_icons::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::collection_theme_inspectors::SelectionPanelThemeInspector;
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::public_interface::render_exposition_doc_sections;
use super::selection_panel_inspector_adapter::{SelectionPanelInspectorAdapter, SELECTION_PANEL_INSPECTOR_SPEC};
use super::template::render_control_exposition_card;

const EVENT_SPECS: &[EventReferenceSpec] = &[
    EventReferenceSpec {
        event: "SelectionPanelEvent::HoverChanged { visible_index }",
        trigger: "Pointer moves across rows",
        notes: "Updates hovered row index while open.",
    },
    EventReferenceSpec {
        event: "SelectionPanelEvent::ActivateRow { source_index, visible_index, item_id }",
        trigger: "Pointer click or Enter on a row",
        notes: "Primary row activation — commit or drill-in handler.",
    },
    EventReferenceSpec {
        event: "SelectionPanelEvent::ActiveIndexChanged { visible_index }",
        trigger: "Arrow keys or Home/End",
        notes: "Keyboard highlight moves between visible rows.",
    },
    EventReferenceSpec {
        event: "SelectionPanelEvent::FocusChanged { focused }",
        trigger: "Focus enters or leaves the panel",
        notes: "Panel focus transitions.",
    },
    EventReferenceSpec {
        event: "SelectionPanelEvent::OpenChanged { open }",
        trigger: "Parent opens or closes the panel",
        notes: "Track visibility for layout.",
    },
    EventReferenceSpec {
        event: "(none)",
        trigger: "Disabled interaction",
        notes: "Row activation ignored while disabled.",
    },
];

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "SelectionPanelControl",
        surface: "Type",
        notes: "Entity<SelectionPanelControl<T>> — scrollable selectable item list.",
    },
    PublicInterfaceSpec {
        symbol: "SelectionPanelEvent",
        surface: "Event",
        notes: "HoverChanged, ActivateRow, ActiveIndexChanged, FocusChanged, OpenChanged.",
    },
    PublicInterfaceSpec {
        symbol: "look.selection_panel_builder(id)",
        surface: "Look",
        notes: "ShadcnLookControlExt builder entry point.",
    },
    PublicInterfaceSpec {
        symbol: "SelectionPanelBuilder::items / scrolling / visible_row_limits",
        surface: "Builder",
        notes: "Item source, scroll mode, and viewport row caps.",
    },
    PublicInterfaceSpec {
        symbol: "SelectionPanelBuilder::with_item_template / spawn(cx)",
        surface: "Builder",
        notes: "Custom row renderer and entity materialization.",
    },
];

pub struct SelectionPanelControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<SelectionPanelExpositionLeftPane>,
    theme_inspector: Entity<SelectionPanelThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct SelectionPanelExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    preview: Entity<SelectionPanelControl<SelectionPanelItem>>,
    event_stream: Entity<ControlEventStream>,
}

impl SelectionPanelExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.preview.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for SelectionPanelExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(16.0))
                .child(div().flex().justify_center().child(self.preview.clone()))
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-selection-panel-left-pane")
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

impl SelectionPanelControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("selection-panel").expect("selection-panel catalog entry");
        let wide_panel_look = Arc::new({
            let active_look = look.clone();
            move |size: ControlSize| {
                let mut panel_look = active_look.selection_panel_look(size);
                panel_look.min_width = 280.0;
                panel_look
            }
        });

        let items = (1..=12)
            .map(|index| {
                SelectionPanelItem::new(format!("item-{index}"))
                    .label(format!("Action item {index}"))
                    .icon(LucideIcon::ListChecks)
            })
            .collect::<Vec<_>>();

        let preview = look
            .selection_panel_builder("controls-doc-selection-panel")
            .panel_id("controls-doc-selection-panel-popup")
            .items(items)
            .scrolling(true)
            .look_provider(wide_panel_look)
            .visible_row_limits(5, 5)
            .selected_source_index(Some(1))
            .active_visible_index(Some(1))
            .open(true)
            .spawn(cx);

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-selection-panel-event-log",
                "Hover rows and activate items; SelectionPanelEvent variants appear below.",
            )
        });

        let left_pane = cx.new(|_| SelectionPanelExpositionLeftPane {
            look: look.clone(),
            entry,
            preview: preview.clone(),
            event_stream: event_stream.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-selection-panel-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &SELECTION_PANEL_INSPECTOR_SPEC,
            SelectionPanelInspectorAdapter::shared(),
        );

        let subscription = cx.subscribe(&preview, {
            let event_stream = event_stream.clone();
            move |_, _, event: &SelectionPanelEvent, cx| {
                let line = format_selection_panel_event(event);
                event_stream.update(cx, |stream, cx| {
                    stream.append_line(&line, cx);
                    cx.notify();
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

impl Render for SelectionPanelControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-selection-panel-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn format_selection_panel_event(event: &SelectionPanelEvent) -> String {
    match event {
        SelectionPanelEvent::HoverChanged { visible_index } => {
            format!("SelectionPanelEvent::HoverChanged {{ visible_index: {visible_index:?} }}")
        }
        SelectionPanelEvent::ActivateRow { source_index, visible_index, item_id } => {
            format!(
                "SelectionPanelEvent::ActivateRow {{ source_index: {source_index}, visible_index: {visible_index}, item_id: \"{item_id}\" }}"
            )
        }
        SelectionPanelEvent::ActiveIndexChanged { visible_index } => {
            format!("SelectionPanelEvent::ActiveIndexChanged {{ visible_index: {visible_index:?} }}")
        }
        SelectionPanelEvent::FocusChanged { focused } => {
            format!("SelectionPanelEvent::FocusChanged {{ focused: {focused} }}")
        }
        SelectionPanelEvent::OpenChanged { open } => {
            format!("SelectionPanelEvent::OpenChanged {{ open: {open} }}")
        }
        _ => "SelectionPanelEvent::(unknown)".to_string(),
    }
}
