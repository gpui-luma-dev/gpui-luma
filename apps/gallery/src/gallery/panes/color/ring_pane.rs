use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, SharedString, Subscription, div, prelude::*, px};
use gpui_luma::controls::color::color_ring::{
    ColorRingEvent, ColorRingRenderer, ColorRingState, HueRingDelegate, LightnessRingDelegate, SaturationRingDelegate,
};
use gpui_luma::controls::color::color_slider::color_spec::Hsv;
use gpui_luma::controls::color::style::{Size, Sizable};
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{format_compact_hsla, format_hex_color, notify_entity};

use super::common::{color_gallery_pane, demo_card, demo_section, detail_row};

const RING_CARD_WIDTH: f32 = 320.0;

#[derive(Clone)]
pub(in crate::gallery) struct ColorRingPane {
    hue_ring: Entity<ColorRingState>,
    saturation_ring: Entity<ColorRingState>,
    lightness_ring: Entity<ColorRingState>,
    vector_ring: Entity<ColorRingState>,
    raster_ring: Entity<ColorRingState>,
    size_small_ring: Entity<ColorRingState>,
    size_large_ring: Entity<ColorRingState>,
    thickness_small_ring: Entity<ColorRingState>,
    thickness_large_ring: Entity<ColorRingState>,
    hsv: Hsv,
    last_event: SharedString,
}

impl ColorRingPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, _look: Arc<ShadcnLook>) -> Self {
        let hsv = Hsv { h: 210.0, s: 0.76, v: 0.9, a: 1.0 };
        let lightness = hsv.to_hsla_ext().l;

        let hue_ring = cx.new(|cx| {
            ColorRingState::hue("color-ring-hue", hsv.h, HueRingDelegate { saturation: hsv.s, lightness }, cx)
                .with_size(Size::Medium)
                .allow_inner_target(true)
        });
        let saturation_ring = cx.new(|cx| {
            ColorRingState::saturation(
                "color-ring-saturation",
                hsv.s,
                SaturationRingDelegate { hue: hsv.h, hsv_value: hsv.v },
                cx,
            )
            .with_size(Size::Medium)
        });
        let lightness_ring = cx.new(|cx| {
            ColorRingState::lightness(
                "color-ring-lightness",
                lightness,
                LightnessRingDelegate { hue: hsv.h, saturation: hsv.s },
                cx,
            )
            .with_size(Size::Medium)
        });
        let vector_ring = cx.new(|cx| {
            ColorRingState::hue_with_renderer(
                "color-ring-vector",
                hsv.h,
                hsv.s,
                lightness,
                ColorRingRenderer::Vector,
                cx,
            )
            .with_size(Size::Medium)
        });
        let raster_ring = cx.new(|cx| {
            ColorRingState::hue_with_renderer(
                "color-ring-raster",
                hsv.h,
                hsv.s,
                lightness,
                ColorRingRenderer::Raster,
                cx,
            )
            .with_size(Size::Medium)
        });
        let size_small_ring = cx.new(|cx| {
            ColorRingState::hue("color-ring-size-small", hsv.h, HueRingDelegate { saturation: hsv.s, lightness }, cx)
                .with_size(Size::Small)
        });
        let size_large_ring = cx.new(|cx| {
            ColorRingState::hue("color-ring-size-large", hsv.h, HueRingDelegate { saturation: hsv.s, lightness }, cx)
                .with_size(Size::Large)
        });
        let thickness_small_ring = cx.new(|cx| {
            ColorRingState::hue(
                "color-ring-thickness-small",
                hsv.h,
                HueRingDelegate { saturation: hsv.s, lightness },
                cx,
            )
            .with_size(Size::Medium)
            .ring_thickness_size(Size::Small)
        });
        let thickness_large_ring = cx.new(|cx| {
            ColorRingState::hue(
                "color-ring-thickness-large",
                hsv.h,
                HueRingDelegate { saturation: hsv.s, lightness },
                cx,
            )
            .with_size(Size::Medium)
            .ring_thickness_size(Size::Large)
        });

        Self {
            hue_ring,
            saturation_ring,
            lightness_ring,
            vector_ring,
            raster_ring,
            size_small_ring,
            size_large_ring,
            thickness_small_ring,
            thickness_large_ring,
            hsv,
            last_event: SharedString::from("Release"),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.hue_ring, |app, _, event: &ColorRingEvent, cx| {
            app.panes.color_ring.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        let selected = self.hsv.to_hsla_ext();

        color_gallery_pane(
            "Color Ring",
            "Dedicated ring demos from the original gallery: hue, saturation, lightness, renderer comparison, and scale variants.",
            div()
                .w_full()
                .max_w(px(1120.0))
                .flex()
                .flex_col()
                .gap(px(28.0))
                .child(demo_section(
                    "Channels",
                    "Core ring delegates from the upstream page inventory.",
                    vec![
                        demo_card(
                            "Hue Ring",
                            "Primary interactive hue ring.",
                            RING_CARD_WIDTH,
                            centered(self.hue_ring.clone()),
                            look,
                        ),
                        demo_card(
                            "Saturation Ring",
                            "Mirrored saturation delegate around a circle.",
                            RING_CARD_WIDTH,
                            centered(self.saturation_ring.clone()),
                            look,
                        ),
                        demo_card(
                            "Lightness Ring",
                            "Mirrored lightness delegate around a circle.",
                            RING_CARD_WIDTH,
                            centered(self.lightness_ring.clone()),
                            look,
                        ),
                    ],
                    look,
                ))
                .child(demo_section(
                    "Renderer Compare",
                    "The original page compares vector and raster rendering paths for the same hue ring.",
                    vec![
                        demo_card(
                            "Vector",
                            "Segmented vector rendering.",
                            RING_CARD_WIDTH,
                            centered(self.vector_ring.clone()),
                            look,
                        ),
                        demo_card(
                            "Raster",
                            "Raster-backed ring rendering.",
                            RING_CARD_WIDTH,
                            centered(self.raster_ring.clone()),
                            look,
                        ),
                        demo_card(
                            "Live Readout",
                            "Current value from the primary hue ring.",
                            RING_CARD_WIDTH,
                            div()
                                .w_full()
                                .flex()
                                .flex_col()
                                .gap(px(10.0))
                                .child(
                                    div()
                                        .h(px(44.0))
                                        .rounded(px(12.0))
                                        .border_1()
                                        .border_color(look.chrome().border)
                                        .bg(selected),
                                )
                                .child(detail_row("Hex", format_hex_color(selected), look))
                                .child(detail_row("HSLA", format_compact_hsla(selected), look))
                                .child(detail_row("Hue", format!("{:.1} deg", self.hsv.h), look))
                                .child(detail_row("Last Event", self.last_event.to_string(), look)),
                            look,
                        ),
                    ],
                    look,
                ))
                .child(demo_section(
                    "Scale",
                    "Representative size and thickness variants from the original page.",
                    vec![
                        demo_card(
                            "Sizes",
                            "Small and large ring footprints.",
                            RING_CARD_WIDTH,
                            div()
                                .w_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .gap(px(16.0))
                                .child(self.size_small_ring.clone())
                                .child(self.size_large_ring.clone()),
                            look,
                        ),
                        demo_card(
                            "Thickness",
                            "Medium ring with thinner and thicker track sizes.",
                            RING_CARD_WIDTH,
                            div()
                                .w_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .gap(px(16.0))
                                .child(self.thickness_small_ring.clone())
                                .child(self.thickness_large_ring.clone()),
                            look,
                        ),
                    ],
                    look,
                ))
                .into_any_element(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.hue_ring, cx);
        notify_entity(&self.saturation_ring, cx);
        notify_entity(&self.lightness_ring, cx);
        notify_entity(&self.vector_ring, cx);
        notify_entity(&self.raster_ring, cx);
        notify_entity(&self.size_small_ring, cx);
        notify_entity(&self.size_large_ring, cx);
        notify_entity(&self.thickness_small_ring, cx);
        notify_entity(&self.thickness_large_ring, cx);
    }

    fn handle_event(&mut self, event: &ColorRingEvent, cx: &mut Context<GalleryApp>) {
        let hue = match event {
            ColorRingEvent::Change(value) => {
                self.last_event = SharedString::from(format!("Change {:.1}", value));
                *value
            }
            ColorRingEvent::Release(value) => {
                self.last_event = SharedString::from(format!("Release {:.1}", value));
                *value
            }
        };

        self.hsv.h = hue;
        self.sync_dependent_controls(cx);
        cx.notify();
    }

    fn sync_dependent_controls(&self, cx: &mut Context<GalleryApp>) {
        let hue = self.hsv.h;
        let saturation = self.hsv.s;
        let lightness = self.hsv.to_hsla_ext().l;
        let value = self.hsv.v;

        self.saturation_ring.update(cx, |ring, cx| {
            ring.set_delegate(Box::new(SaturationRingDelegate { hue, hsv_value: value }), cx);
        });
        self.lightness_ring.update(cx, |ring, cx| {
            ring.set_delegate(Box::new(LightnessRingDelegate { hue, saturation }), cx);
        });
        self.vector_ring.update(cx, |ring, cx| {
            ring.set_delegate(Box::new(HueRingDelegate { saturation, lightness }), cx);
            ring.set_value(hue, cx);
        });
        self.raster_ring.update(cx, |ring, cx| {
            ring.set_delegate(Box::new(HueRingDelegate { saturation, lightness }), cx);
            ring.set_value(hue, cx);
        });
        self.size_small_ring.update(cx, |ring, cx| {
            ring.set_delegate(Box::new(HueRingDelegate { saturation, lightness }), cx);
            ring.set_value(hue, cx);
        });
        self.size_large_ring.update(cx, |ring, cx| {
            ring.set_delegate(Box::new(HueRingDelegate { saturation, lightness }), cx);
            ring.set_value(hue, cx);
        });
        self.thickness_small_ring.update(cx, |ring, cx| {
            ring.set_delegate(Box::new(HueRingDelegate { saturation, lightness }), cx);
            ring.set_value(hue, cx);
        });
        self.thickness_large_ring.update(cx, |ring, cx| {
            ring.set_delegate(Box::new(HueRingDelegate { saturation, lightness }), cx);
            ring.set_value(hue, cx);
        });
    }
}

fn centered(content: impl gpui::IntoElement) -> AnyElement {
    div().w_full().flex().items_center().justify_center().child(content).into_any_element()
}
