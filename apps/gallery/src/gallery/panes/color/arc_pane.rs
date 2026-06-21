use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, SharedString, Subscription, div, prelude::*, px};
use gpui_luma::controls::color::color_arc::{
    ColorArcEvent, ColorArcRenderer, ColorArcState, HueArcDelegate, LightnessArcDelegate, SaturationArcDelegate,
};
use gpui_luma::controls::color::color_slider::color_spec::Hsv;
use gpui_luma::controls::color::style::Size;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{format_compact_hsla, format_hex_color, notify_entity};

use super::common::{color_gallery_pane, demo_card, demo_section, detail_row};

const ARC_CARD_WIDTH: f32 = 320.0;

#[derive(Clone)]
pub(in crate::gallery) struct ColorArcPane {
    hue_arc: Entity<ColorArcState>,
    saturation_arc: Entity<ColorArcState>,
    lightness_arc: Entity<ColorArcState>,
    vector_arc: Entity<ColorArcState>,
    raster_arc: Entity<ColorArcState>,
    hue_arc_270: Entity<ColorArcState>,
    saturation_arc_270: Entity<ColorArcState>,
    lightness_arc_270: Entity<ColorArcState>,
    thickness_small_arc: Entity<ColorArcState>,
    thickness_large_arc: Entity<ColorArcState>,
    hsv: Hsv,
    last_event: SharedString,
}

impl ColorArcPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, _look: Arc<ShadcnLook>) -> Self {
        let hsv = Hsv { h: 28.0, s: 0.74, v: 0.92, a: 1.0 };
        let lightness = hsv.to_hsla_ext().l;

        let hue_arc = cx.new(|cx| {
            ColorArcState::hue("color-arc-hue", hsv.h, HueArcDelegate { saturation: hsv.s, lightness }, cx)
                .size(Size::Medium)
                .start_degrees(-45.0)
                .sweep_degrees(270.0)
        });
        let saturation_arc = cx.new(|cx| {
            ColorArcState::saturation(
                "color-arc-saturation",
                hsv.s,
                SaturationArcDelegate { hue: hsv.h, hsv_value: hsv.v },
                cx,
            )
            .size(Size::Medium)
            .start_degrees(-45.0)
            .sweep_degrees(270.0)
        });
        let lightness_arc = cx.new(|cx| {
            ColorArcState::lightness(
                "color-arc-lightness",
                lightness,
                LightnessArcDelegate { hue: hsv.h, saturation: hsv.s },
                cx,
            )
            .size(Size::Medium)
            .start_degrees(-45.0)
            .sweep_degrees(270.0)
        });
        let vector_arc = cx.new(|cx| {
            ColorArcState::hue_with_renderer("color-arc-vector", hsv.h, hsv.s, lightness, ColorArcRenderer::Vector, cx)
                .size(Size::Medium)
                .start_degrees(0.0)
                .sweep_degrees(180.0)
        });
        let raster_arc = cx.new(|cx| {
            ColorArcState::hue_with_renderer("color-arc-raster", hsv.h, hsv.s, lightness, ColorArcRenderer::Raster, cx)
                .size(Size::Medium)
                .start_degrees(0.0)
                .sweep_degrees(180.0)
        });
        let hue_arc_270 = cx.new(|cx| {
            ColorArcState::hue("color-arc-hue-270", hsv.h, HueArcDelegate { saturation: hsv.s, lightness }, cx)
                .size(Size::Medium)
                .start_degrees(-45.0)
                .sweep_degrees(270.0)
        });
        let saturation_arc_270 = cx.new(|cx| {
            ColorArcState::saturation(
                "color-arc-saturation-270",
                hsv.s,
                SaturationArcDelegate { hue: hsv.h, hsv_value: hsv.v },
                cx,
            )
            .size(Size::Medium)
            .start_degrees(-45.0)
            .sweep_degrees(270.0)
        });
        let lightness_arc_270 = cx.new(|cx| {
            ColorArcState::lightness(
                "color-arc-lightness-270",
                lightness,
                LightnessArcDelegate { hue: hsv.h, saturation: hsv.s },
                cx,
            )
            .size(Size::Medium)
            .start_degrees(-45.0)
            .sweep_degrees(270.0)
        });
        let thickness_small_arc = cx.new(|cx| {
            ColorArcState::hue("color-arc-thickness-small", hsv.h, HueArcDelegate { saturation: hsv.s, lightness }, cx)
                .size(Size::Medium)
                .arc_thickness_size(Size::Small)
                .start_degrees(0.0)
                .sweep_degrees(180.0)
        });
        let thickness_large_arc = cx.new(|cx| {
            ColorArcState::hue("color-arc-thickness-large", hsv.h, HueArcDelegate { saturation: hsv.s, lightness }, cx)
                .size(Size::Medium)
                .arc_thickness_size(Size::Large)
                .start_degrees(0.0)
                .sweep_degrees(180.0)
        });

        Self {
            hue_arc,
            saturation_arc,
            lightness_arc,
            vector_arc,
            raster_arc,
            hue_arc_270,
            saturation_arc_270,
            lightness_arc_270,
            thickness_small_arc,
            thickness_large_arc,
            hsv,
            last_event: SharedString::from("Release"),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.hue_arc, |app, _, event: &ColorArcEvent, cx| {
            app.panes.color_arc.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        let selected = self.hsv.to_hsla_ext();

        color_gallery_pane(
            "Color Arc",
            "Split-out arc demos from the original project: half arcs, 270-degree arcs, renderer comparison, and thickness variants.",
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(28.0))
                .child(demo_section(
                    "Channels",
                    "Hue, saturation, and lightness delegates rendered as 270-degree arcs.",
                    vec![
                        demo_card(
                            "Hue Arc",
                            "Primary interactive hue arc.",
                            ARC_CARD_WIDTH,
                            centered(self.hue_arc.clone()),
                            look,
                        ),
                        demo_card(
                            "Saturation Arc",
                            "Mirrored saturation arc delegate.",
                            ARC_CARD_WIDTH,
                            centered(self.saturation_arc.clone()),
                            look,
                        ),
                        demo_card(
                            "Lightness Arc",
                            "Mirrored lightness arc delegate.",
                            ARC_CARD_WIDTH,
                            centered(self.lightness_arc.clone()),
                            look,
                        ),
                    ],
                    look,
                ))
                .child(demo_section(
                    "Renderer Compare",
                    "Vector and raster hue arcs rendered with the same geometry.",
                    vec![
                        demo_card(
                            "Vector",
                            "Segmented vector arc.",
                            ARC_CARD_WIDTH,
                            centered(self.vector_arc.clone()),
                            look,
                        ),
                        demo_card(
                            "Raster",
                            "Raster-backed arc.",
                            ARC_CARD_WIDTH,
                            centered(self.raster_arc.clone()),
                            look,
                        ),
                        demo_card(
                            "Live Readout",
                            "Current value from the primary hue arc.",
                            ARC_CARD_WIDTH,
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
                    "Geometry",
                    "The upstream page spends most of its space on sweep-angle and thickness combinations.",
                    vec![
                        demo_card(
                            "270-Degree Set",
                            "Hue, saturation, and lightness arcs with a longer sweep.",
                            ARC_CARD_WIDTH,
                            div()
                                .w_full()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap(px(14.0))
                                .child(self.hue_arc_270.clone())
                                .child(self.saturation_arc_270.clone())
                                .child(self.lightness_arc_270.clone()),
                            look,
                        ),
                        demo_card(
                            "Thickness",
                            "Thinner and thicker variants of the same hue arc.",
                            ARC_CARD_WIDTH,
                            div()
                                .w_full()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap(px(14.0))
                                .child(self.thickness_small_arc.clone())
                                .child(self.thickness_large_arc.clone()),
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
        notify_entity(&self.hue_arc, cx);
        notify_entity(&self.saturation_arc, cx);
        notify_entity(&self.lightness_arc, cx);
        notify_entity(&self.vector_arc, cx);
        notify_entity(&self.raster_arc, cx);
        notify_entity(&self.hue_arc_270, cx);
        notify_entity(&self.saturation_arc_270, cx);
        notify_entity(&self.lightness_arc_270, cx);
        notify_entity(&self.thickness_small_arc, cx);
        notify_entity(&self.thickness_large_arc, cx);
    }

    fn handle_event(&mut self, event: &ColorArcEvent, cx: &mut Context<GalleryApp>) {
        let hue = match event {
            ColorArcEvent::Change(value) => {
                self.last_event = SharedString::from(format!("Change {:.1}", value));
                *value
            }
            ColorArcEvent::Release(value) => {
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
        let value = self.hsv.v;
        let lightness = self.hsv.to_hsla_ext().l;

        self.saturation_arc.update(cx, |arc, cx| {
            arc.set_delegate(Box::new(SaturationArcDelegate { hue, hsv_value: value }), cx);
        });
        self.lightness_arc.update(cx, |arc, cx| {
            arc.set_delegate(Box::new(LightnessArcDelegate { hue, saturation }), cx);
        });
        self.vector_arc.update(cx, |arc, cx| {
            arc.set_value(hue, cx);
            arc.set_delegate(Box::new(HueArcDelegate { saturation, lightness }), cx);
        });
        self.raster_arc.update(cx, |arc, cx| {
            arc.set_value(hue, cx);
            arc.set_delegate(Box::new(HueArcDelegate { saturation, lightness }), cx);
        });
        self.hue_arc_270.update(cx, |arc, cx| {
            arc.set_value(hue, cx);
            arc.set_delegate(Box::new(HueArcDelegate { saturation, lightness }), cx);
        });
        self.saturation_arc_270.update(cx, |arc, cx| {
            arc.set_delegate(Box::new(SaturationArcDelegate { hue, hsv_value: value }), cx);
        });
        self.lightness_arc_270.update(cx, |arc, cx| {
            arc.set_delegate(Box::new(LightnessArcDelegate { hue, saturation }), cx);
        });
        self.thickness_small_arc.update(cx, |arc, cx| {
            arc.set_value(hue, cx);
            arc.set_delegate(Box::new(HueArcDelegate { saturation, lightness }), cx);
        });
        self.thickness_large_arc.update(cx, |arc, cx| {
            arc.set_value(hue, cx);
            arc.set_delegate(Box::new(HueArcDelegate { saturation, lightness }), cx);
        });
    }
}

fn centered(content: impl gpui::IntoElement) -> AnyElement {
    div().w_full().flex().items_center().justify_center().child(content).into_any_element()
}
