//! HSV plane composition exposition — Photoshop-style HSV plane with H, S, and V sliders.

use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use luma_color::color_field::ColorFieldEvent;
use luma_color::composition::CompositionSize;
use luma::controls::slider::SliderEvent;
use luma_look_shadcn::prelude::*;
use luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::color_compositions::hsv_plane::HsvPlaneDemo;
use super::color_exposition_common::{
    composition_caption_text_size, composition_title_text_size, format_color_field_event, format_slider_event,
    render_demo_section, render_labeled_demo_card,
};
use super::event_stream::ControlEventStream;
use super::model::{ControlExpositionLayout};
use super::template::render_control_exposition_card;

pub struct ColorHsvPlaneControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    state_sm: Entity<HsvPlaneDemo>,
    state_md: Entity<HsvPlaneDemo>,
    state_lg: Entity<HsvPlaneDemo>,
    event_stream: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}

impl ColorHsvPlaneControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("color-hsv-plane").expect("color-hsv-plane catalog entry");

        let state_sm = cx.new(|cx| HsvPlaneDemo::with_size(look.clone(), CompositionSize::Sm, cx));
        let state_md = cx.new(|cx| HsvPlaneDemo::with_size(look.clone(), CompositionSize::Md, cx));
        let state_lg = cx.new(|cx| HsvPlaneDemo::with_size(look.clone(), CompositionSize::Lg, cx));

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-color-hsv-plane-event-log",
                "Edit the medium HSV plane; ColorFieldEvent and SliderEvent variants appear below.",
            )
        });

        let subscriptions = wire_hsv_plane_events(&state_md, &event_stream, cx);

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

impl Render for ColorHsvPlaneControlExposition {
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
                    "HSV Plane",
                    "Photoshop-style HSV plane with linked H, S, and V channel sliders.",
                    div()
                        .flex()
                        .flex_wrap()
                        .items_start()
                        .gap(px(16.0))
                        .child(render_labeled_demo_card(
                            look,
                            "Sm",
                            "Compact composition metrics.",
                            HsvPlaneDemo::card_width_for(CompositionSize::Sm),
                            composition_title_text_size(CompositionSize::Sm),
                            composition_caption_text_size(CompositionSize::Sm),
                            self.state_sm.clone(),
                        ))
                        .child(render_labeled_demo_card(
                            look,
                            "Md",
                            "Default composition metrics.",
                            HsvPlaneDemo::card_width_for(CompositionSize::Md),
                            composition_title_text_size(CompositionSize::Md),
                            composition_caption_text_size(CompositionSize::Md),
                            self.state_md.clone(),
                        ))
                        .child(render_labeled_demo_card(
                            look,
                            "Lg",
                            "Expanded composition metrics.",
                            HsvPlaneDemo::card_width_for(CompositionSize::Lg),
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
                None,
                ControlExpositionLayout::BORDERLESS_NO_HEADING,
            )
        })
    }
}

fn wire_hsv_plane_events(
    state: &Entity<HsvPlaneDemo>,
    event_stream: &Entity<ControlEventStream>,
    cx: &mut Context<ColorHsvPlaneControlExposition>,
) -> Vec<Subscription> {
    let plane = state.read(cx).plane();
    let slider_h = state.read(cx).slider_h();
    let slider_s = state.read(cx).slider_s();
    let slider_v = state.read(cx).slider_v();
    let event_stream = event_stream.clone();

    vec![
        cx.subscribe(&plane, {
            let event_stream = event_stream.clone();
            move |_, _, event: &ColorFieldEvent, cx| {
                if let Some(line) = format_color_field_event(event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            }
        }),
        cx.subscribe(&slider_h, {
            let event_stream = event_stream.clone();
            move |_, _, event: &SliderEvent, cx| {
                if let Some(line) = format_slider_event("Hue", event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            }
        }),
        cx.subscribe(&slider_s, {
            let event_stream = event_stream.clone();
            move |_, _, event: &SliderEvent, cx| {
                if let Some(line) = format_slider_event("Saturation", event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            }
        }),
        cx.subscribe(&slider_v, move |_, _, event: &SliderEvent, cx| {
            if let Some(line) = format_slider_event("Value", event) {
                event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
            }
        }),
    ]
}
