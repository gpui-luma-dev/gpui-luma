use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Render, Subscription, Window, div, prelude::*, px, rgb};
use gpui_luma::controls::scrollbar::{Scrollbar, ScrollbarEvent};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::public_interface::render_exposition_doc_sections;
use super::scrollbar_inspector_adapter::{ScrollbarInspectorAdapter, SCROLLBAR_INSPECTOR_SPEC};
use super::standalone_theme_inspectors::ScrollbarThemeInspector;
use super::template::render_control_exposition_card;

const EVENT_SPECS: &[EventReferenceSpec] = &[
    EventReferenceSpec {
        event: "ScrollbarEvent::Change { value }",
        trigger: "Thumb drag, track click, or keyboard step",
        notes: "Scroll offset updates while interacting.",
    },
    EventReferenceSpec {
        event: "ScrollbarEvent::DragStart",
        trigger: "Pointer down on thumb or track",
        notes: "Begin drag transaction.",
    },
    EventReferenceSpec {
        event: "ScrollbarEvent::DragEnd { value }",
        trigger: "Pointer up after drag",
        notes: "Final committed offset.",
    },
    EventReferenceSpec {
        event: "ScrollbarEvent::FocusChanged { focused }",
        trigger: "Focus enters or leaves the scrollbar",
        notes: "Keyboard navigation readiness.",
    },
    EventReferenceSpec {
        event: "ScrollbarEvent::HoverChanged { hovered }",
        trigger: "Pointer enters or leaves the scrollbar",
        notes: "Hover chrome transitions.",
    },
    EventReferenceSpec { event: "(none)", trigger: "Disabled interaction", notes: "Ignored while disabled." },
];

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "Scrollbar",
        surface: "Type",
        notes: "Entity<ScrollbarControl> — standalone scroll thumb/track control.",
    },
    PublicInterfaceSpec {
        symbol: "ScrollbarEvent",
        surface: "Event",
        notes: "Change, DragStart, DragEnd, FocusChanged, HoverChanged, EnabledChanged.",
    },
    PublicInterfaceSpec { symbol: "look.scrollbar(id)", surface: "Look", notes: "ShadcnLookControlExt factory." },
    PublicInterfaceSpec {
        symbol: "ScrollbarBuilder::horizontal / vertical",
        surface: "Builder",
        notes: "Orientation before spawn.",
    },
    PublicInterfaceSpec {
        symbol: "ScrollbarBuilder::range / step / page_step / thumb_fraction",
        surface: "Builder",
        notes: "Scroll geometry and thumb sizing.",
    },
    PublicInterfaceSpec {
        symbol: "ScrollbarBuilder::spawn(cx)",
        surface: "Builder",
        notes: "Materialize entity; subscribe for ScrollbarEvent.",
    },
];

pub struct ScrollbarControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<ScrollbarExpositionLeftPane>,
    theme_inspector: Entity<ScrollbarThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct ScrollbarExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    horizontal_scrollbar: Entity<Scrollbar>,
    vertical_scrollbar: Entity<Scrollbar>,
    event_stream: Entity<ControlEventStream>,
    horizontal_value: f32,
    vertical_value: f32,
}

impl ScrollbarExpositionLeftPane {
    fn handle_scrollbar_event(&mut self, axis: &str, event: &ScrollbarEvent, cx: &mut Context<Self>) {
        if let ScrollbarEvent::Change { value } = event {
            match axis {
                "horizontal" => self.horizontal_value = *value,
                _ => self.vertical_value = *value,
            }
            cx.notify();
        }
    }

    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.horizontal_scrollbar.update(cx, |_, cx| cx.notify());
        self.vertical_scrollbar.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for ScrollbarExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(16.0))
                .child(scrollbar_pair(
                    self.horizontal_value,
                    self.vertical_value,
                    self.horizontal_scrollbar.clone(),
                    self.vertical_scrollbar.clone(),
                    &self.look,
                ))
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-scrollbar-left-pane")
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

impl ScrollbarControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("scrollbar").expect("scrollbar catalog entry");
        let horizontal_scrollbar = look
            .scrollbar("controls-doc-scrollbar-horizontal")
            .horizontal()
            .range(0..220)
            .step(20)
            .page_step(80)
            .value(40)
            .thumb_fraction(0.54)
            .spawn(cx);
        let vertical_scrollbar = look
            .scrollbar("controls-doc-scrollbar-vertical")
            .vertical()
            .range(0..240)
            .step(20)
            .page_step(80)
            .value(80)
            .thumb_fraction(0.45)
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-scrollbar-event-log",
                "Drag the scrollbars; ScrollbarEvent variants appear below.",
            )
        });

        let left_pane = cx.new(|_| ScrollbarExpositionLeftPane {
            look: look.clone(),
            entry,
            horizontal_scrollbar: horizontal_scrollbar.clone(),
            vertical_scrollbar: vertical_scrollbar.clone(),
            event_stream: event_stream.clone(),
            horizontal_value: 40.0,
            vertical_value: 80.0,
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-scrollbar-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &SCROLLBAR_INSPECTOR_SPEC,
            ScrollbarInspectorAdapter::shared(),
        );

        let mut subscriptions = Vec::new();
        for (scrollbar, axis) in
            [(horizontal_scrollbar.clone(), "horizontal"), (vertical_scrollbar.clone(), "vertical")]
        {
            let event_stream = event_stream.clone();
            let left_pane = left_pane.clone();
            subscriptions.push(cx.subscribe(&scrollbar, move |_, _, event: &ScrollbarEvent, cx| {
                left_pane.update(cx, |pane, cx| pane.handle_scrollbar_event(axis, event, cx));
                if let Some(line) = format_scrollbar_event(axis, event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
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

impl Render for ScrollbarControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-scrollbar-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn format_scrollbar_event(axis: &str, event: &ScrollbarEvent) -> Option<String> {
    match event {
        ScrollbarEvent::Change { value } => Some(format!("ScrollbarEvent::Change {{ value: {value:.0} }} ({axis})")),
        ScrollbarEvent::DragStart => Some(format!("ScrollbarEvent::DragStart ({axis})")),
        ScrollbarEvent::DragEnd { value } => Some(format!("ScrollbarEvent::DragEnd {{ value: {value:.0} }} ({axis})")),
        ScrollbarEvent::FocusChanged { focused } => {
            Some(format!("ScrollbarEvent::FocusChanged {{ focused: {focused} }} ({axis})"))
        }
        ScrollbarEvent::HoverChanged { hovered } => {
            Some(format!("ScrollbarEvent::HoverChanged {{ hovered: {hovered} }} ({axis})"))
        }
        ScrollbarEvent::EnabledChanged { enabled } => {
            Some(format!("ScrollbarEvent::EnabledChanged {{ enabled: {enabled} }} ({axis})"))
        }
        _ => None,
    }
}

fn scrollbar_pair(
    horizontal_value: f32,
    vertical_value: f32,
    horizontal_scrollbar: Entity<Scrollbar>,
    vertical_scrollbar: Entity<Scrollbar>,
    look: &ShadcnLook,
) -> AnyElement {
    let chrome = look.chrome();
    let mut demo_content = div()
        .absolute()
        .left(px(-horizontal_value))
        .top(px(-vertical_value))
        .flex()
        .flex_col()
        .gap_2()
        .p_3();

    for row in 0..12 {
        demo_content = demo_content.child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().w(px(92.0)).text_color(rgb(0x0f172a)).child(format!("Row {:02}", row + 1)))
                .child(div().w(px(110.0)).h(px(22.0)).rounded(px(4.0)).bg(rgb(0xbae6fd)))
                .child(div().w(px(150.0)).h(px(22.0)).rounded(px(4.0)).bg(rgb(0xbbf7d0)))
                .child(div().w(px(96.0)).h(px(22.0)).rounded(px(4.0)).bg(rgb(0xfed7aa))),
        );
    }

    div()
        .flex()
        .items_start()
        .gap_2()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .relative()
                        .w(px(260.0))
                        .h(px(180.0))
                        .overflow_hidden()
                        .rounded(px(6.0))
                        .border_1()
                        .border_color(chrome.border)
                        .bg(chrome.panel_background)
                        .child(demo_content),
                )
                .child(horizontal_scrollbar),
        )
        .child(vertical_scrollbar)
        .into_any_element()
}
