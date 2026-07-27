//! Multi mixer composition exposition — per-color-space channel mixers.

use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::slider::SliderEvent;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::color_compositions::multi_mixer::MultiMixerDemo;
use super::color_exposition_common::{format_slider_event, render_demo_section};
use super::event_stream::ControlEventStream;
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::public_interface::render_exposition_doc_sections;
use super::template::render_control_exposition_card;

const EVENT_SPECS: &[EventReferenceSpec] = &[EventReferenceSpec {
    event: "SliderEvent::Change / Release { value }",
    trigger: "Drag the HSVA hue slider",
    notes: "Representative channel event from the HSVA mixer card.",
}];

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "ColorSliderBuilder::channel / hue / alpha",
        surface: "Factory",
        notes: "Per-color-space channel sliders bound to ColorSpecification delegates.",
    },
    PublicInterfaceSpec {
        symbol: "ColorSpecification",
        surface: "Model",
        notes: "HueAlpha, RGBA, HSLA, HSVA, Lab, and OKLCH specification traits.",
    },
    PublicInterfaceSpec {
        symbol: "ColorSpaceMixerState",
        surface: "Composition",
        notes: "Self-contained mixer card with swatch readout and labeled channel rows.",
    },
];

pub struct ColorMultiMixerControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    demo: Entity<MultiMixerDemo>,
    event_stream: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}

impl ColorMultiMixerControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("color-multi-mixer").expect("color-multi-mixer catalog entry");
        let demo = cx.new(|cx| MultiMixerDemo::new(look.clone(), cx));

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-color-multi-mixer-event-log",
                "Edit the HSVA hue slider; SliderEvent variants appear below.",
            )
        });

        let subscriptions = wire_multi_mixer_events(&demo, &event_stream, cx);

        Self { look, entry, demo, event_stream, _subscriptions: subscriptions }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.demo.update(cx, |demo, cx| demo.sync_look(look.clone(), cx));
        self.event_stream.update(cx, |stream, cx| stream.sync_look(self.look.clone(), cx));
        cx.notify();
    }
}

impl Render for ColorMultiMixerControlExposition {
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
                    "Multi Mixer",
                    "Representative per-color-space channel mixers from the gallery inventory.",
                    div().child(self.demo.clone()).into_any_element(),
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

fn wire_multi_mixer_events(
    demo: &Entity<MultiMixerDemo>,
    event_stream: &Entity<ControlEventStream>,
    cx: &mut Context<ColorMultiMixerControlExposition>,
) -> Vec<Subscription> {
    let hue_slider = demo.read(cx).hsva_hue_slider(cx);

    vec![cx.subscribe(&hue_slider, {
        let event_stream = event_stream.clone();
        move |_, _, event: &SliderEvent, cx| {
            if let Some(line) = format_slider_event("HSVA Hue", event) {
                event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
            }
        }
    })]
}
