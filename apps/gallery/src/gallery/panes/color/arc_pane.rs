use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, SharedString, Subscription, div, prelude::*, px};
use gpui_luma::controls::color::color_arc::{
    ColorArcBuilder, ColorArcDomainRenderer, ColorArcRenderer, ColorArcTrackContext, HueArcDelegate,
    LightnessArcDelegate, RasterArcDelegate, SaturationArcDelegate, refresh_color_arc, update_arc_delegate,
};
use gpui_luma::controls::color::color_slider::color_spec::Hsv;
use gpui_luma::controls::color::style::Size;
use gpui_luma::controls::slider::{SliderControl, SliderEvent};
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{format_compact_hsla, format_hex_color, notify_entity};

use super::common::{color_gallery_pane, demo_card, demo_section, detail_row};

const ARC_CARD_WIDTH: f32 = 320.0;

#[derive(Clone)]
struct ArcDemo {
    slider: Entity<SliderControl>,
    renderer: Arc<ColorArcDomainRenderer>,
    track_context: ColorArcTrackContext,
}

impl ArcDemo {
    fn spawn(builder: ColorArcBuilder, cx: &mut Context<GalleryApp>) -> Self {
        let renderer = builder.domain_renderer();
        let track_context = builder.track_context();
        let slider = builder.spawn(cx);
        Self { slider, renderer, track_context }
    }

    fn sync_saturation(&self, hue: f32, hsv_value: f32, cx: &mut Context<GalleryApp>) {
        update_arc_delegate(
            &self.renderer,
            Arc::new(SaturationArcDelegate { hue, hsv_value }),
            self.track_context.clone(),
        );
        refresh_color_arc(&self.slider, cx);
    }

    fn sync_lightness(&self, hue: f32, saturation: f32, cx: &mut Context<GalleryApp>) {
        update_arc_delegate(
            &self.renderer,
            Arc::new(LightnessArcDelegate { hue, saturation }),
            self.track_context.clone(),
        );
        refresh_color_arc(&self.slider, cx);
    }

    fn sync_hue_raster(&self, saturation: f32, lightness: f32, cx: &mut Context<GalleryApp>) {
        update_arc_delegate(
            &self.renderer,
            Arc::new(RasterArcDelegate::hue(saturation, lightness)),
            self.track_context.clone(),
        );
        refresh_color_arc(&self.slider, cx);
    }

    fn sync_hue_vector(&self, saturation: f32, lightness: f32, cx: &mut Context<GalleryApp>) {
        update_arc_delegate(
            &self.renderer,
            Arc::new(HueArcDelegate { saturation, lightness }),
            self.track_context.clone(),
        );
        refresh_color_arc(&self.slider, cx);
    }

    fn set_value(&self, value: f32, cx: &mut Context<GalleryApp>) {
        self.slider.update(cx, |slider, cx| slider.set_value(value, cx));
    }
}

#[derive(Clone)]
pub(in crate::gallery) struct ColorArcPane {
    hue_arc: ArcDemo,
    saturation_arc: ArcDemo,
    lightness_arc: ArcDemo,
    vector_arc: ArcDemo,
    raster_arc: ArcDemo,
    hue_arc_270: ArcDemo,
    saturation_arc_270: ArcDemo,
    lightness_arc_270: ArcDemo,
    thickness_small_arc: ArcDemo,
    thickness_large_arc: ArcDemo,
    hsv: Hsv,
    last_event: SharedString,
}

impl ColorArcPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, _look: Arc<ShadcnLook>) -> Self {
        let hsv = Hsv { h: 28.0, s: 0.74, v: 0.92, a: 1.0 };
        let lightness = hsv.to_hsla_ext().l;

        let hue_arc = ArcDemo::spawn(
            ColorArcBuilder::hue("color-arc-hue", hsv.h, hsv.s, lightness)
                .size(Size::Medium)
                .start_degrees(-45.0)
                .sweep_degrees(270.0),
            cx,
        );
        let saturation_arc = ArcDemo::spawn(
            ColorArcBuilder::saturation("color-arc-saturation", hsv.s, hsv.h, hsv.v)
                .size(Size::Medium)
                .start_degrees(-45.0)
                .sweep_degrees(270.0),
            cx,
        );
        let lightness_arc = ArcDemo::spawn(
            ColorArcBuilder::lightness("color-arc-lightness", lightness, hsv.h, hsv.s)
                .size(Size::Medium)
                .start_degrees(-45.0)
                .sweep_degrees(270.0),
            cx,
        );
        let vector_arc = ArcDemo::spawn(
            ColorArcBuilder::hue_with_renderer("color-arc-vector", hsv.h, hsv.s, lightness, ColorArcRenderer::Vector)
                .size(Size::Medium)
                .start_degrees(0.0)
                .sweep_degrees(180.0),
            cx,
        );
        let raster_arc = ArcDemo::spawn(
            ColorArcBuilder::hue_with_renderer("color-arc-raster", hsv.h, hsv.s, lightness, ColorArcRenderer::Raster)
                .size(Size::Medium)
                .start_degrees(0.0)
                .sweep_degrees(180.0),
            cx,
        );
        let hue_arc_270 = ArcDemo::spawn(
            ColorArcBuilder::hue("color-arc-hue-270", hsv.h, hsv.s, lightness)
                .size(Size::Medium)
                .start_degrees(-45.0)
                .sweep_degrees(270.0),
            cx,
        );
        let saturation_arc_270 = ArcDemo::spawn(
            ColorArcBuilder::saturation("color-arc-saturation-270", hsv.s, hsv.h, hsv.v)
                .size(Size::Medium)
                .start_degrees(-45.0)
                .sweep_degrees(270.0),
            cx,
        );
        let lightness_arc_270 = ArcDemo::spawn(
            ColorArcBuilder::lightness("color-arc-lightness-270", lightness, hsv.h, hsv.s)
                .size(Size::Medium)
                .start_degrees(-45.0)
                .sweep_degrees(270.0),
            cx,
        );
        let thickness_small_arc = ArcDemo::spawn(
            ColorArcBuilder::hue("color-arc-thickness-small", hsv.h, hsv.s, lightness)
                .size(Size::Medium)
                .arc_thickness_size(Size::Small)
                .start_degrees(0.0)
                .sweep_degrees(180.0),
            cx,
        );
        let thickness_large_arc = ArcDemo::spawn(
            ColorArcBuilder::hue("color-arc-thickness-large", hsv.h, hsv.s, lightness)
                .size(Size::Medium)
                .arc_thickness_size(Size::Large)
                .start_degrees(0.0)
                .sweep_degrees(180.0),
            cx,
        );

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
        subscriptions.push(cx.subscribe(&self.hue_arc.slider, |app, _, event: &SliderEvent, cx| {
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
                            centered(self.hue_arc.slider.clone()),
                            look,
                        ),
                        demo_card(
                            "Saturation Arc",
                            "Mirrored saturation arc delegate.",
                            ARC_CARD_WIDTH,
                            centered(self.saturation_arc.slider.clone()),
                            look,
                        ),
                        demo_card(
                            "Lightness Arc",
                            "Mirrored lightness arc delegate.",
                            ARC_CARD_WIDTH,
                            centered(self.lightness_arc.slider.clone()),
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
                            centered(self.vector_arc.slider.clone()),
                            look,
                        ),
                        demo_card(
                            "Raster",
                            "Raster-backed arc.",
                            ARC_CARD_WIDTH,
                            centered(self.raster_arc.slider.clone()),
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
                                .child(self.hue_arc_270.slider.clone())
                                .child(self.saturation_arc_270.slider.clone())
                                .child(self.lightness_arc_270.slider.clone()),
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
                                .child(self.thickness_small_arc.slider.clone())
                                .child(self.thickness_large_arc.slider.clone()),
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
        notify_entity(&self.hue_arc.slider, cx);
        notify_entity(&self.saturation_arc.slider, cx);
        notify_entity(&self.lightness_arc.slider, cx);
        notify_entity(&self.vector_arc.slider, cx);
        notify_entity(&self.raster_arc.slider, cx);
        notify_entity(&self.hue_arc_270.slider, cx);
        notify_entity(&self.saturation_arc_270.slider, cx);
        notify_entity(&self.lightness_arc_270.slider, cx);
        notify_entity(&self.thickness_small_arc.slider, cx);
        notify_entity(&self.thickness_large_arc.slider, cx);
    }

    fn handle_event(&mut self, event: &SliderEvent, cx: &mut Context<GalleryApp>) {
        let hue = match event {
            SliderEvent::Change { value, .. } => {
                self.last_event = SharedString::from(format!("Change {:.1}", value));
                *value
            }
            SliderEvent::Release { value, .. } => {
                self.last_event = SharedString::from(format!("Release {:.1}", value));
                *value
            }
            _ => return,
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

        self.saturation_arc.sync_saturation(hue, value, cx);
        self.lightness_arc.sync_lightness(hue, saturation, cx);
        self.vector_arc.set_value(hue, cx);
        self.vector_arc.sync_hue_vector(saturation, lightness, cx);
        self.raster_arc.set_value(hue, cx);
        self.raster_arc.sync_hue_raster(saturation, lightness, cx);
        self.hue_arc_270.set_value(hue, cx);
        self.hue_arc_270.sync_hue_raster(saturation, lightness, cx);
        self.saturation_arc_270.sync_saturation(hue, value, cx);
        self.lightness_arc_270.sync_lightness(hue, saturation, cx);
        self.thickness_small_arc.set_value(hue, cx);
        self.thickness_small_arc.sync_hue_raster(saturation, lightness, cx);
        self.thickness_large_arc.set_value(hue, cx);
        self.thickness_large_arc.sync_hue_raster(saturation, lightness, cx);
    }
}

fn centered(content: impl gpui::IntoElement) -> AnyElement {
    div().w_full().flex().items_center().justify_center().child(content).into_any_element()
}
