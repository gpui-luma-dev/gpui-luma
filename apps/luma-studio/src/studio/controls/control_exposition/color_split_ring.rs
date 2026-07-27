//! Split ring composition exposition — offset saturation/lightness arcs with inner hue ring.

use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::color::composition::CompositionSize;
use gpui_luma::controls::slider::SliderEvent;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::color_compositions::split_ring::SplitRingDemo;
use super::color_exposition_common::{
    composition_caption_text_size, composition_title_text_size, format_slider_event, render_demo_section,
    render_labeled_demo_card,
};
use super::event_stream::ControlEventStream;
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::public_interface::render_exposition_doc_sections;
use super::template::render_control_exposition_card;

const EVENT_SPECS: &[EventReferenceSpec] = &[EventReferenceSpec {
    event: "SliderEvent::Change / Release { value }",
    trigger: "Drag the inner hue ring",
    notes: "Updates hue on the shared HSL model; saturation and lightness arcs stay linked.",
}];

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "ColorArcBuilder + ColorRingBuilder::hue",
        surface: "Composition",
        notes: "Offset saturation and lightness arcs framing an inner hue ring.",
    },
    PublicInterfaceSpec {
        symbol: "ColorCompositionSync",
        surface: "Sync",
        notes: "Keeps arc delegates and hue ring in sync without feedback loops.",
    },
    PublicInterfaceSpec {
        symbol: "CompositionSize",
        surface: "Layout",
        notes: "Sm, Md, and Lg resolve outer size, track width, and arc offsets.",
    },
];

pub struct ColorSplitRingControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    state_sm: Entity<SplitRingDemo>,
    state_md: Entity<SplitRingDemo>,
    state_lg: Entity<SplitRingDemo>,
    event_stream: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}

impl ColorSplitRingControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("color-split-ring").expect("color-split-ring catalog entry");

        let state_sm = cx.new(|cx| SplitRingDemo::with_size(look.clone(), CompositionSize::Sm, cx));
        let state_md = cx.new(|cx| SplitRingDemo::with_size(look.clone(), CompositionSize::Md, cx));
        let state_lg = cx.new(|cx| SplitRingDemo::with_size(look.clone(), CompositionSize::Lg, cx));

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-color-split-ring-event-log",
                "Edit the medium split ring; hue ring SliderEvent variants appear below.",
            )
        });

        let subscriptions = wire_split_ring_events(&state_md, &event_stream, cx);

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

impl Render for ColorSplitRingControlExposition {
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
                    "Split Ring",
                    "Offset saturation and lightness arcs with an inner hue ring.",
                    div()
                        .flex()
                        .flex_wrap()
                        .items_start()
                        .gap(px(16.0))
                        .child(render_labeled_demo_card(
                            look,
                            "Sm",
                            "Compact composition metrics.",
                            SplitRingDemo::card_width_for(CompositionSize::Sm),
                            composition_title_text_size(CompositionSize::Sm),
                            composition_caption_text_size(CompositionSize::Sm),
                            self.state_sm.clone(),
                        ))
                        .child(render_labeled_demo_card(
                            look,
                            "Md",
                            "Default composition metrics.",
                            SplitRingDemo::card_width_for(CompositionSize::Md),
                            composition_title_text_size(CompositionSize::Md),
                            composition_caption_text_size(CompositionSize::Md),
                            self.state_md.clone(),
                        ))
                        .child(render_labeled_demo_card(
                            look,
                            "Lg",
                            "Expanded composition metrics.",
                            SplitRingDemo::card_width_for(CompositionSize::Lg),
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

fn wire_split_ring_events(
    state: &Entity<SplitRingDemo>,
    event_stream: &Entity<ControlEventStream>,
    cx: &mut Context<ColorSplitRingControlExposition>,
) -> Vec<Subscription> {
    let hue_ring = state.read(cx).hue_ring();

    vec![cx.subscribe(&hue_ring, {
        let event_stream = event_stream.clone();
        move |_, _, event: &SliderEvent, cx| {
            if let Some(line) = format_slider_event("Hue Ring", event) {
                event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
            }
        }
    })]
}
