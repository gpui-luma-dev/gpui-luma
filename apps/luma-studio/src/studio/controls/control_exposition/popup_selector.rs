use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::selector::{Selector, SelectorEvent, SelectorItem, SelectorPlacement};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_icons::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::input_theme_inspectors::SelectorThemeInspector;
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::selector_inspector_adapter::{SelectorInspectorAdapter, SELECTOR_INSPECTOR_SPEC};
use super::template::render_control_exposition_card;

pub struct PopupSelectorControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<SelectorExpositionLeftPane>,
    theme_inspector: Entity<SelectorThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct SelectorExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    preview_below: Entity<Selector>,
    preview_smart: Entity<Selector>,
    event_stream: Entity<ControlEventStream>,
}

impl SelectorExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.preview_below.update(cx, |_, cx| cx.notify());
        self.preview_smart.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for SelectorExpositionLeftPane {
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
                .id("controls-doc-selector-left-pane")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .overflow_y_scroll()
                .child(render_control_exposition_card(
                    &self.look,
                    self.entry,
                    preview.into_any_element(),
                    None,
                    ControlExpositionLayout::BORDERLESS,
                ))
        })
    }
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
        let left_pane = cx.new(|_| SelectorExpositionLeftPane {
            look: look.clone(),
            entry,
            preview_below: preview_below.clone(),
            preview_smart: preview_smart.clone(),
            event_stream: event_stream.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-selector-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &SELECTOR_INSPECTOR_SPEC,
            SelectorInspectorAdapter::shared(),
        );

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

impl Render for PopupSelectorControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-selector-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
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
