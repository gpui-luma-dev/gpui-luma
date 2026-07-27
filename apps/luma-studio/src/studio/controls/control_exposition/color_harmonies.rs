//! Color harmonies composition exposition — wheel, lightness ring, harmony selector, and palette readout.

use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::color::color_field::ColorFieldEvent;
use gpui_luma::controls::color::composition::CompositionSize;
use gpui_luma::controls::slider::SliderEvent;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::color_compositions::color_harmonies::{
    ColorHarmoniesDemo, composition_caption_text_size, composition_title_text_size,
};
use super::color_exposition_common::{
    format_color_field_event, format_slider_event, render_demo_section, render_labeled_demo_card,
};
use super::event_stream::ControlEventStream;
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::public_interface::render_exposition_doc_sections;
use super::template::render_control_exposition_card;

const EVENT_SPECS: &[EventReferenceSpec] = &[
    EventReferenceSpec {
        event: "ColorFieldEvent::Change / Release(Hsv)",
        trigger: "Drag inside the hue wheel",
        notes: "Updates hue and saturation on the shared HSL model.",
    },
    EventReferenceSpec {
        event: "SliderEvent::Change / Release { value }",
        trigger: "Drag the lightness ring",
        notes: "Updates lightness on the shared HSL model.",
    },
];

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "HslWheelModel + ColorRingBuilder::lightness",
        surface: "Composition",
        notes: "HSL hue/saturation wheel at current lightness, with outer lightness ring and palette readout.",
    },
    PublicInterfaceSpec {
        symbol: "ColorCombination",
        surface: "Model",
        notes: "Monochromatic through hexadic harmony palettes derived from the base color.",
    },
    PublicInterfaceSpec {
        symbol: "CompositionSize",
        surface: "Layout",
        notes: "Sm, Md, and Lg resolve ring size, wheel thumb, and canvas padding.",
    },
];

pub struct ColorHarmoniesControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    state_sm: Entity<ColorHarmoniesDemo>,
    state_md: Entity<ColorHarmoniesDemo>,
    state_lg: Entity<ColorHarmoniesDemo>,
    event_stream: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}

impl ColorHarmoniesControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("color-harmonies").expect("color-harmonies catalog entry");

        let state_sm = cx.new(|cx| ColorHarmoniesDemo::with_size(look.clone(), CompositionSize::Sm, cx));
        let state_md = cx.new(|cx| ColorHarmoniesDemo::with_size(look.clone(), CompositionSize::Md, cx));
        let state_lg = cx.new(|cx| ColorHarmoniesDemo::with_size(look.clone(), CompositionSize::Lg, cx));

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-color-harmonies-event-log",
                "Edit the medium harmonies demo; ColorFieldEvent and SliderEvent variants appear below.",
            )
        });

        let subscriptions = wire_harmonies_events(&state_md, &event_stream, cx);

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

impl Render for ColorHarmoniesControlExposition {
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
                    "Color Harmonies",
                    "Hue wheel and lightness ring with harmony selector and palette readout.",
                    div()
                        .flex()
                        .flex_wrap()
                        .items_start()
                        .gap(px(16.0))
                        .child(render_labeled_demo_card(
                            look,
                            "Sm",
                            "Compact composition metrics.",
                            ColorHarmoniesDemo::card_width_for(CompositionSize::Sm),
                            composition_title_text_size(CompositionSize::Sm),
                            composition_caption_text_size(CompositionSize::Sm),
                            self.state_sm.clone(),
                        ))
                        .child(render_labeled_demo_card(
                            look,
                            "Md",
                            "Default composition metrics.",
                            ColorHarmoniesDemo::card_width_for(CompositionSize::Md),
                            composition_title_text_size(CompositionSize::Md),
                            composition_caption_text_size(CompositionSize::Md),
                            self.state_md.clone(),
                        ))
                        .child(render_labeled_demo_card(
                            look,
                            "Lg",
                            "Expanded composition metrics.",
                            ColorHarmoniesDemo::card_width_for(CompositionSize::Lg),
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
                ControlExpositionLayout::BORDERLESS_NO_HEADING,
            )
        })
    }
}

fn wire_harmonies_events(
    state: &Entity<ColorHarmoniesDemo>,
    event_stream: &Entity<ControlEventStream>,
    cx: &mut Context<ColorHarmoniesControlExposition>,
) -> Vec<Subscription> {
    let wheel = state.read(cx).wheel();
    let lightness_ring = state.read(cx).lightness_ring();
    let event_stream = event_stream.clone();

    vec![
        cx.subscribe(&wheel, {
            let event_stream = event_stream.clone();
            move |_, _, event: &ColorFieldEvent, cx| {
                if let Some(line) = format_color_field_event(event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            }
        }),
        cx.subscribe(&lightness_ring, move |_, _, event: &SliderEvent, cx| {
            if let Some(line) = format_slider_event("Lightness Ring", event) {
                event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
            }
        }),
    ]
}
