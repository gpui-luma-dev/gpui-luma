use std::f32::consts::PI;
use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::slider::{Slider, SliderEvent, SliderThumbPolicy};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::public_interface::render_exposition_doc_sections;
use super::template::render_control_exposition_card;

const EVENT_SPECS: &[EventReferenceSpec] = &[
    EventReferenceSpec {
        event: "SliderEvent::Change { thumb_id, value }",
        trigger: "Pointer drag or keyboard nudge while dragging",
        notes: "Emitted continuously while the thumb moves.",
    },
    EventReferenceSpec {
        event: "SliderEvent::Release { thumb_id, value }",
        trigger: "Pointer up or keyboard commit after drag",
        notes: "Prefer for committing model state.",
    },
    EventReferenceSpec {
        event: "SliderEvent::DragStart { thumb_id }",
        trigger: "Pointer down on thumb or track activation",
        notes: "Begin drag transaction.",
    },
    EventReferenceSpec {
        event: "SliderEvent::DragEnd { thumb_id, value }",
        trigger: "Pointer up after drag",
        notes: "Pairs with DragStart.",
    },
    EventReferenceSpec {
        event: "SliderEvent::ThumbSelected { thumb_id }",
        trigger: "Multi-stop thumb selection",
        notes: "Active thumb changed in multi-thumb mode.",
    },
    EventReferenceSpec { event: "(none)", trigger: "Disabled interaction", notes: "Ignored while disabled." },
];

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "Slider",
        surface: "Type",
        notes: "Entity<SliderControl> — unified linear and angular slider engine.",
    },
    PublicInterfaceSpec {
        symbol: "SliderEvent",
        surface: "Event",
        notes: "Change, Release, DragStart, DragEnd, ThumbSelected, FocusChanged, multi-thumb lifecycle.",
    },
    PublicInterfaceSpec { symbol: "look.slider(id)", surface: "Look", notes: "ShadcnLookControlExt factory." },
    PublicInterfaceSpec {
        symbol: "SliderBuilder::vertical / reversed / angular / multi_stop",
        surface: "Builder",
        notes: "Orientation, direction, dial strategies, and thumb policy.",
    },
    PublicInterfaceSpec {
        symbol: "SliderBuilder::allowed_intervals / wrapping / corner_radius",
        surface: "Builder",
        notes: "Blocked ranges, circular wrap, and track chrome overrides.",
    },
    PublicInterfaceSpec {
        symbol: "SliderBuilder::spawn(cx)",
        surface: "Builder",
        notes: "Materialize entity; subscribe for SliderEvent.",
    },
];

pub struct SliderControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    fill_slider: Slider,
    blocked_slider: Slider,
    reversed_slider: Slider,
    angular_slider: Slider,
    wrapping_slider: Slider,
    stops_slider: Slider,
    event_stream: Entity<ControlEventStream>,
    fill_value: f32,
    blocked_value: f32,
    reversed_value: f32,
    angular_value: f32,
    wrapping_value: f32,
    stop_summary: String,
    stops_interaction_hint: Option<String>,
    _subscriptions: Vec<Subscription>,
}

impl SliderControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("slider").expect("slider catalog entry");
        let stops_policy = SliderThumbPolicy::multi_stop();
        let stops_interaction_hint = multi_stop_interaction_hint(stops_policy);

        let fill_slider = look.slider("controls-doc-slider-fill").range(1..100).step(1).value(41).spawn(cx);
        let blocked_slider = look
            .slider("controls-doc-slider-blocked")
            .range(0.0..360.0)
            .step(10.0)
            .value(100.0)
            .allowed_intervals(vec![0.0..=120.0, 180.0..=240.0, 300.0..=360.0])
            .spawn(cx);
        let reversed_slider =
            look.slider("controls-doc-slider-reversed").reversed(true).range(1..100).step(1).value(41).spawn(cx);
        let angular_slider = look
            .slider("controls-doc-slider-angular")
            .angular(-1.25 * PI, 0.25 * PI)
            .template(look.slider_angular_template())
            .range(0..100)
            .step(1)
            .value(50)
            .spawn(cx);
        let wrapping_slider = look
            .slider("controls-doc-slider-wrapping")
            .angular(0.0, 2.0 * PI)
            .wrapping(true)
            .domain()
            .template(look.slider_circular_ring_template())
            .range(0.0..360.0)
            .step(1.0)
            .value(180.0)
            .spawn(cx);
        let stops_slider = look
            .slider("controls-doc-slider-stops")
            .multi_stop()
            .range(0.0..360.0)
            .step(1.0)
            .thumb_values([(0.0, None), (180.0, None), (300.0, None)])
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-slider-event-log",
                "Drag sliders; SliderEvent variants from the fill track appear below.",
            )
        });

        let mut subscriptions = Vec::new();
        subscriptions.push(wire_value_slider(cx, &fill_slider, "fill", event_stream.clone(), SliderValueTarget::Fill));
        subscriptions.push(wire_value_slider(
            cx,
            &blocked_slider,
            "blocked",
            event_stream.clone(),
            SliderValueTarget::Blocked,
        ));
        subscriptions.push(wire_value_slider(
            cx,
            &reversed_slider,
            "reversed",
            event_stream.clone(),
            SliderValueTarget::Reversed,
        ));
        subscriptions.push(wire_value_slider(
            cx,
            &angular_slider,
            "angular",
            event_stream.clone(),
            SliderValueTarget::Angular,
        ));
        subscriptions.push(wire_value_slider(
            cx,
            &wrapping_slider,
            "wrapping",
            event_stream.clone(),
            SliderValueTarget::Wrapping,
        ));
        subscriptions.push(cx.subscribe(&stops_slider, {
            let event_stream = event_stream.clone();
            move |this, _, event: &SliderEvent, cx| {
                this.handle_stops_event(event, cx);
                if let Some(line) = format_slider_event("stops", event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            }
        }));

        let mut this = Self {
            look,
            entry,
            fill_slider,
            blocked_slider,
            reversed_slider,
            angular_slider,
            wrapping_slider,
            stops_slider,
            event_stream,
            fill_value: 41.0,
            blocked_value: 100.0,
            reversed_value: 41.0,
            angular_value: 50.0,
            wrapping_value: 180.0,
            stop_summary: String::new(),
            stops_interaction_hint,
            _subscriptions: subscriptions,
        };
        this.sync_stop_summary(cx);
        this
    }

    fn handle_stops_event(&mut self, event: &SliderEvent, cx: &mut Context<Self>) {
        if let SliderEvent::ThumbSelected { .. } = event {}
        self.sync_stop_summary(cx);
        cx.notify();
    }

    fn sync_stop_summary(&mut self, cx: &mut Context<Self>) {
        let slider = self.stops_slider.read(cx);
        let mut stops = slider.thumbs().iter().map(|thumb| slider.range().value_at(thumb.position)).collect::<Vec<_>>();
        stops.sort_by(|left, right| left.total_cmp(right));

        let selected_value = slider
            .active_thumb_id()
            .and_then(|id| slider.thumb_value(id))
            .unwrap_or(stops.first().copied().unwrap_or(0.0));

        self.stop_summary = format!(
            "{} stops [{}] · selected {:.0}",
            stops.len(),
            stops.iter().map(|value| format!("{value:.0}")).collect::<Vec<_>>().join(", "),
            selected_value
        );
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for slider in [
            &self.fill_slider,
            &self.blocked_slider,
            &self.reversed_slider,
            &self.angular_slider,
            &self.wrapping_slider,
            &self.stops_slider,
        ] {
            slider.update(cx, |_, cx| cx.notify());
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

#[derive(Clone, Copy)]
enum SliderValueTarget {
    Fill,
    Blocked,
    Reversed,
    Angular,
    Wrapping,
}

fn wire_value_slider(
    cx: &mut Context<SliderControlExposition>,
    slider: &Slider,
    label: &'static str,
    event_stream: Entity<ControlEventStream>,
    target: SliderValueTarget,
) -> Subscription {
    cx.subscribe(slider, move |this, _, event: &SliderEvent, cx| {
        if let Some(value) = slider_release_or_change_value(event) {
            match target {
                SliderValueTarget::Fill => this.fill_value = value,
                SliderValueTarget::Blocked => this.blocked_value = value,
                SliderValueTarget::Reversed => this.reversed_value = value,
                SliderValueTarget::Angular => this.angular_value = value,
                SliderValueTarget::Wrapping => this.wrapping_value = value,
            }
            cx.notify();
        }
        if matches!(target, SliderValueTarget::Fill) {
            if let Some(line) = format_slider_event(label, event) {
                event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
            }
        }
    })
}

impl Render for SliderControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(24.0))
                .child(demo_section(
                    "Fill track",
                    chrome.muted_text,
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(8.0))
                        .child(div().w(px(320.0)).child(self.fill_slider.clone()))
                        .child(value_label(self.fill_value, chrome.body_text)),
                ))
                .child(demo_section(
                    "Reversed fill track",
                    chrome.muted_text,
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(8.0))
                        .child(div().w(px(320.0)).child(self.reversed_slider.clone()))
                        .child(value_label(self.reversed_value, chrome.body_text)),
                ))
                .child(demo_section(
                    "Blocked intervals",
                    chrome.muted_text,
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(8.0))
                        .child(div().w(px(320.0)).child(self.blocked_slider.clone()))
                        .child(value_label(self.blocked_value, chrome.body_text)),
                ))
                .child(demo_section(
                    "Multi-stop",
                    chrome.muted_text,
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(8.0))
                        .when_some(self.stops_interaction_hint.as_ref(), |this, hint| {
                            this.child(
                                div()
                                    .text_size(px(11.0))
                                    .line_height(px(14.0))
                                    .text_color(chrome.muted_text)
                                    .child(hint.clone()),
                            )
                        })
                        .child(div().w(px(320.0)).child(self.stops_slider.clone()))
                        .child(stops_label(&self.stop_summary, chrome.body_text)),
                ))
                .child(demo_section(
                    "Circular dials",
                    chrome.muted_text,
                    div()
                        .flex()
                        .flex_row()
                        .items_start()
                        .justify_center()
                        .gap(px(32.0))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .text_size(px(11.0))
                                        .line_height(px(14.0))
                                        .text_color(chrome.muted_text)
                                        .child("Angular dial"),
                                )
                                .child(self.angular_slider.clone())
                                .child(value_label(self.angular_value, chrome.body_text)),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .text_size(px(11.0))
                                        .line_height(px(14.0))
                                        .text_color(chrome.muted_text)
                                        .child("Wrapping dial"),
                                )
                                .child(self.wrapping_slider.clone())
                                .child(value_label(self.wrapping_value, chrome.body_text)),
                        ),
                ))
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

fn slider_release_or_change_value(event: &SliderEvent) -> Option<f32> {
    match event {
        SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } => Some(*value),
        _ => None,
    }
}

fn format_slider_event(source: &str, event: &SliderEvent) -> Option<String> {
    match event {
        SliderEvent::Change { value, .. } => Some(format!("SliderEvent::Change - {source} ({value:.0})")),
        SliderEvent::Release { value, .. } => Some(format!("SliderEvent::Release - {source} ({value:.0})")),
        SliderEvent::DragStart { .. } => Some(format!("SliderEvent::DragStart - {source}")),
        SliderEvent::DragEnd { value, .. } => Some(format!("SliderEvent::DragEnd - {source} ({value:.0})")),
        SliderEvent::ThumbSelected { thumb_id } => {
            Some(format!("SliderEvent::ThumbSelected {{ thumb_id: {thumb_id:?} }}"))
        }
        SliderEvent::FocusChanged { focused } => Some(format!("SliderEvent::FocusChanged {{ focused: {focused} }}")),
        _ => None,
    }
}

fn demo_section(title: &'static str, title_color: gpui::Hsla, content: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(4.0))
        .child(section_label(title, title_color))
        .child(content)
}

fn section_label(label: &'static str, color: gpui::Hsla) -> impl IntoElement {
    div().text_size(px(12.0)).line_height(px(16.0)).text_color(color).child(label)
}

fn value_label(value: f32, color: gpui::Hsla) -> impl IntoElement {
    div()
        .text_size(px(12.0))
        .line_height(px(16.0))
        .text_color(color)
        .child(format!("Value: {:.0}", value))
}

fn stops_label(summary: &str, color: gpui::Hsla) -> impl IntoElement {
    div().text_size(px(12.0)).line_height(px(16.0)).text_color(color).child(summary.to_string())
}

fn multi_stop_interaction_hint(policy: SliderThumbPolicy) -> Option<String> {
    let mut parts = Vec::new();
    if policy.supports_click_to_add() {
        parts.push("Click empty track to add");
    }
    if policy.supports_delete_to_remove() {
        parts.push("Delete/Backspace to remove selected");
    }
    (!parts.is_empty()).then(|| parts.join(" · "))
}
