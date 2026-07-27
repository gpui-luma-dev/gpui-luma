//! HSV wheel composition exposition — hue ring with centered saturation/value square.

use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::color::color_field::ColorFieldEvent;
use gpui_luma::controls::color::composition::CompositionSize;
use gpui_luma::controls::slider::SliderEvent;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::color_compositions::hsv_wheel::HsvWheelDemo;
use super::color_exposition_common::{
    composition_caption_text_size, composition_title_text_size, format_color_field_event, format_slider_event,
    render_demo_section, render_labeled_demo_card,
};
use super::event_stream::ControlEventStream;
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::public_interface::render_exposition_doc_sections;
use super::template::render_control_exposition_card;

const EVENT_SPECS: &[EventReferenceSpec] = &[
    EventReferenceSpec {
        event: "SliderEvent::Change / Release { value }",
        trigger: "Drag the outer hue ring",
        notes: "Updates hue on the shared HSV model.",
    },
    EventReferenceSpec {
        event: "ColorFieldEvent::Change / Release(Hsv)",
        trigger: "Drag inside the centered SV square",
        notes: "Updates saturation and value on the shared HSV model.",
    },
];

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "ColorRingBuilder::hue + ColorFieldState::saturation_value",
        surface: "Composition",
        notes: "Hue ring with centered edge-to-edge SV square.",
    },
    PublicInterfaceSpec {
        symbol: "ColorCompositionSync",
        surface: "Sync",
        notes: "Prevents feedback loops while syncing linked ring and field values.",
    },
    PublicInterfaceSpec {
        symbol: "CompositionSize",
        surface: "Layout",
        notes: "Sm, Md, and Lg resolve ring outer size and inner plane scaling.",
    },
];

pub struct ColorHsvWheelControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    state_sm: Entity<HsvWheelDemo>,
    state_md: Entity<HsvWheelDemo>,
    state_lg: Entity<HsvWheelDemo>,
    event_stream: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}

impl ColorHsvWheelControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("color-hsv-wheel").expect("color-hsv-wheel catalog entry");

        let state_sm = cx.new(|cx| HsvWheelDemo::with_size(look.clone(), CompositionSize::Sm, cx));
        let state_md = cx.new(|cx| HsvWheelDemo::with_size(look.clone(), CompositionSize::Md, cx));
        let state_lg = cx.new(|cx| HsvWheelDemo::with_size(look.clone(), CompositionSize::Lg, cx));

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-color-hsv-wheel-event-log",
                "Edit the medium HSV wheel; SliderEvent and ColorFieldEvent variants appear below.",
            )
        });

        let subscriptions = wire_hsv_wheel_events(&state_md, &event_stream, cx);

        Self { look, entry, state_sm, state_md, state_lg, event_stream, _subscriptions: subscriptions }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for state in [&self.state_sm, &self.state_md, &self.state_lg] {
            state.update(cx, |demo, cx| demo.sync_look(look.clone(), cx));
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(self.look.clone(), cx));
        cx.notify();
    }
}

impl Render for ColorHsvWheelControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let look = &self.look;

            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(28.0))
                .child(render_demo_section(
                    look,
                    "HSV Wheel",
                    "Hue ring with a centered saturation/value square.",
                    div()
                        .flex()
                        .flex_wrap()
                        .items_start()
                        .gap(px(16.0))
                        .child(render_labeled_demo_card(
                            look,
                            "Sm",
                            "Compact composition metrics.",
                            HsvWheelDemo::card_width_for(CompositionSize::Sm),
                            composition_title_text_size(CompositionSize::Sm),
                            composition_caption_text_size(CompositionSize::Sm),
                            self.state_sm.clone(),
                        ))
                        .child(render_labeled_demo_card(
                            look,
                            "Md",
                            "Default composition metrics.",
                            HsvWheelDemo::card_width_for(CompositionSize::Md),
                            composition_title_text_size(CompositionSize::Md),
                            composition_caption_text_size(CompositionSize::Md),
                            self.state_md.clone(),
                        ))
                        .child(render_labeled_demo_card(
                            look,
                            "Lg",
                            "Expanded composition metrics.",
                            HsvWheelDemo::card_width_for(CompositionSize::Lg),
                            composition_title_text_size(CompositionSize::Lg),
                            composition_caption_text_size(CompositionSize::Lg),
                            self.state_lg.clone(),
                        ))
                        .into_any_element(),
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

fn wire_hsv_wheel_events(
    state: &Entity<HsvWheelDemo>,
    event_stream: &Entity<ControlEventStream>,
    cx: &mut Context<ColorHsvWheelControlExposition>,
) -> Vec<Subscription> {
    let color_ring = state.read(cx).color_ring();
    let plane_sv = state.read(cx).plane_sv();
    let event_stream = event_stream.clone();

    vec![
        cx.subscribe(&color_ring, {
            let event_stream = event_stream.clone();
            move |_, _, event: &SliderEvent, cx| {
                if let Some(line) = format_slider_event("Hue Ring", event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            }
        }),
        cx.subscribe(&plane_sv, move |_, _, event: &ColorFieldEvent, cx| {
            if let Some(line) = format_color_field_event(event) {
                event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
            }
        }),
    ]
}
