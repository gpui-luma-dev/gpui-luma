use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Subscription, black, div, hsla, prelude::*, px, white};
use gpui_luma::controls::color::color_ring::{
    primary_slider_value, refresh_color_ring, update_ring_delegate, ColorRingBuilder,
    ColorRingDomainRenderer, ColorRingRenderer, ColorRingTrackContext, HueRingDelegate,
    LightnessRingDelegate, RasterRingDelegate, SaturationRingDelegate,
};
use gpui_luma::controls::color::color_slider::color_spec::{Hsl, Hsv};
use gpui_luma::controls::color::color_slider::ColorSpecification;
use gpui_luma::controls::color::style::Size;
use gpui_luma::controls::slider::{SliderControl, SliderEvent};
use gpui_luma::theme::ThemeMode;
use gpui_luma::{vstack, wrappanel};
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextSize};

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::notify_entity;

use super::common::{color_gallery_pane, demo_card, demo_section, detail_row};

const WIDE_CARD: f32 = 1120.0;
const SWATCH_SIZE: f32 = 220.0;
const RING_FRAME_PX: f32 = 256.0;
const RING_MEDIUM_PX: f32 = 220.0;
const SLIDER_COLUMN_WIDTH: f32 = 280.0;
const READOUT_WIDTH: f32 = 160.0;

#[derive(Clone)]
struct RingDemo {
    slider: Entity<SliderControl>,
    renderer: Arc<ColorRingDomainRenderer>,
    track_context: ColorRingTrackContext,
}

impl RingDemo {
    fn spawn(builder: ColorRingBuilder, cx: &mut Context<GalleryApp>) -> Self {
        let renderer = builder.domain_renderer();
        let track_context = builder.track_context();
        let slider = builder.spawn(cx);
        Self { slider, renderer, track_context }
    }

    fn sync_saturation(&self, hue: f32, hsv_value: f32, cx: &mut Context<GalleryApp>) {
        update_ring_delegate(
            &self.renderer,
            Arc::new(SaturationRingDelegate { hue, hsv_value }),
            self.track_context.clone(),
        );
        refresh_color_ring(&self.slider, cx);
    }

    fn sync_lightness(&self, hue: f32, saturation: f32, cx: &mut Context<GalleryApp>) {
        update_ring_delegate(
            &self.renderer,
            Arc::new(LightnessRingDelegate { hue, saturation }),
            self.track_context.clone(),
        );
        refresh_color_ring(&self.slider, cx);
    }

    fn sync_hue_raster(&self, saturation: f32, lightness: f32, cx: &mut Context<GalleryApp>) {
        update_ring_delegate(
            &self.renderer,
            Arc::new(RasterRingDelegate::hue(saturation, lightness)),
            self.track_context.clone(),
        );
        refresh_color_ring(&self.slider, cx);
    }

    fn sync_hue_vector(&self, saturation: f32, lightness: f32, cx: &mut Context<GalleryApp>) {
        update_ring_delegate(
            &self.renderer,
            Arc::new(HueRingDelegate { saturation, lightness }),
            self.track_context.clone(),
        );
        refresh_color_ring(&self.slider, cx);
    }

    fn set_value(&self, value: f32, cx: &mut Context<GalleryApp>) {
        self.slider.update(cx, |slider, cx| slider.set_value(value, cx));
    }
}

#[derive(Clone)]
pub(in crate::gallery) struct ColorRingPane {
    color_ring: RingDemo,
    color_ring_vector_compare: RingDemo,
    color_ring_raster: RingDemo,
    color_ring_vector_compare_inner_target: RingDemo,
    color_ring_raster_inner_target: RingDemo,
    color_ring_saturation_vector: RingDemo,
    color_ring_saturation_vector_rotated: RingDemo,
    color_ring_saturation_raster: RingDemo,
    color_ring_saturation_raster_rotated: RingDemo,
    color_ring_lightness_vector: RingDemo,
    color_ring_lightness_raster: RingDemo,
    color_ring_disabled_vector: RingDemo,
    color_ring_disabled_raster: RingDemo,
    ring_no_border: RingDemo,
    ring_inner_border: RingDemo,
    ring_outer_border: RingDemo,
    ring_both_borders: RingDemo,
    ring_foreground_borders: RingDemo,
    ring_size_xsmall: RingDemo,
    ring_size_small: RingDemo,
    ring_size_medium: RingDemo,
    ring_size_large: RingDemo,
    ring_thickness_xsmall: RingDemo,
    ring_thickness_small: RingDemo,
    ring_thickness_medium: RingDemo,
    ring_thickness_large: RingDemo,
    ring_saturation: RingDemo,
    ring_lightness: RingDemo,
    ring_hsl: Hsl,
    ring_color: gpui::Hsla,
    ring_event_name: &'static str,
    ring_event_value: f32,
    ring_saturation_event_name: &'static str,
    ring_saturation_event_value: f32,
    ring_lightness_event_name: &'static str,
    ring_lightness_event_value: f32,
}

impl ColorRingPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let hsla = hsla(0.0, 1.0, 0.5, 1.0);
        let hsl = Hsl::from_hsla(hsla);
        let hsv_value = Hsv::from_hsla_ext(hsla).v;
        let contrast_border = if look.mode() == ThemeMode::Dark {
            white()
        } else {
            black()
        };

        let color_ring = RingDemo::spawn(
            ColorRingBuilder::hue("color_ring", hsl.h, hsl.s, hsl.l)
                .size(Size::Medium)
                .allow_inner_target(true),
            cx,
        );

        let color_ring_vector_compare = RingDemo::spawn(
            ColorRingBuilder::hue_with_renderer(
                "color_ring_vector_compare",
                hsl.h,
                hsl.s,
                hsl.l,
                ColorRingRenderer::Vector,
            )
            .size(Size::Medium),
            cx,
        );
        let color_ring_raster = RingDemo::spawn(
            ColorRingBuilder::hue_with_renderer("color_ring_raster", hsl.h, hsl.s, hsl.l, ColorRingRenderer::Raster)
                .size(Size::Medium),
            cx,
        );
        let color_ring_vector_compare_inner_target = RingDemo::spawn(
            ColorRingBuilder::hue_with_renderer(
                "color_ring_vector_compare_inner_target",
                hsl.h,
                hsl.s,
                hsl.l,
                ColorRingRenderer::Vector,
            )
            .size(Size::Medium)
            .allow_inner_target(true),
            cx,
        );
        let color_ring_raster_inner_target = RingDemo::spawn(
            ColorRingBuilder::hue_with_renderer(
                "color_ring_raster_inner_target",
                hsl.h,
                hsl.s,
                hsl.l,
                ColorRingRenderer::Raster,
            )
            .size(Size::Medium)
            .allow_inner_target(true),
            cx,
        );

        let color_ring_saturation_vector = RingDemo::spawn(
            ColorRingBuilder::saturation_with_renderer(
                "color_ring_saturation_vector",
                hsl.s,
                hsl.h,
                hsv_value,
                ColorRingRenderer::Vector,
            )
            .size(Size::Medium),
            cx,
        );
        let color_ring_saturation_vector_rotated = RingDemo::spawn(
            ColorRingBuilder::saturation_with_renderer(
                "color_ring_saturation_vector_rotated",
                hsl.s,
                hsl.h,
                hsv_value,
                ColorRingRenderer::Vector,
            )
            .size(Size::Medium)
            .rotation_degrees(180.0),
            cx,
        );
        let color_ring_saturation_raster = RingDemo::spawn(
            ColorRingBuilder::saturation_with_renderer(
                "color_ring_saturation_raster",
                hsl.s,
                hsl.h,
                1.0,
                ColorRingRenderer::Raster,
            )
            .size(Size::Medium),
            cx,
        );
        let color_ring_saturation_raster_rotated = RingDemo::spawn(
            ColorRingBuilder::saturation_with_renderer(
                "color_ring_saturation_raster_rotated",
                hsl.s,
                hsl.h,
                1.0,
                ColorRingRenderer::Raster,
            )
            .size(Size::Medium)
            .rotation_degrees(180.0),
            cx,
        );

        let color_ring_lightness_vector = RingDemo::spawn(
            ColorRingBuilder::lightness_with_renderer(
                "color_ring_lightness_vector",
                hsl.l,
                hsl.h,
                hsl.s,
                ColorRingRenderer::Vector,
            )
            .size(Size::Medium),
            cx,
        );
        let color_ring_lightness_raster = RingDemo::spawn(
            ColorRingBuilder::lightness_with_renderer(
                "color_ring_lightness_raster",
                hsl.l,
                hsl.h,
                hsl.s,
                ColorRingRenderer::Raster,
            )
            .size(Size::Medium),
            cx,
        );

        let color_ring_disabled_vector = RingDemo::spawn(
            ColorRingBuilder::hue_with_renderer(
                "color_ring_disabled_vector",
                hsl.h,
                hsl.s,
                hsl.l,
                ColorRingRenderer::Vector,
            )
            .size(Size::Medium)
            .enabled(false),
            cx,
        );
        let color_ring_disabled_raster = RingDemo::spawn(
            ColorRingBuilder::hue_with_renderer(
                "color_ring_disabled_raster",
                hsl.h,
                hsl.s,
                hsl.l,
                ColorRingRenderer::Raster,
            )
            .size(Size::Medium)
            .enabled(false),
            cx,
        );

        let ring_no_border = RingDemo::spawn(
            ColorRingBuilder::new("ring_no_border", hsl.h, Arc::new(HueRingDelegate { saturation: hsl.s, lightness: hsl.l }))
                .size(Size::Small)
                .ring_inner_border(false)
                .ring_outer_border(false),
            cx,
        );
        let ring_inner_border = RingDemo::spawn(
            ColorRingBuilder::new("ring_inner_border", hsl.h, Arc::new(HueRingDelegate { saturation: hsl.s, lightness: hsl.l }))
                .size(Size::Small)
                .ring_inner_border(true)
                .ring_outer_border(false),
            cx,
        );
        let ring_outer_border = RingDemo::spawn(
            ColorRingBuilder::new("ring_outer_border", hsl.h, Arc::new(HueRingDelegate { saturation: hsl.s, lightness: hsl.l }))
                .size(Size::Small)
                .ring_inner_border(false)
                .ring_outer_border(true),
            cx,
        );
        let ring_both_borders = RingDemo::spawn(
            ColorRingBuilder::new("ring_both_borders", hsl.h, Arc::new(HueRingDelegate { saturation: hsl.s, lightness: hsl.l }))
                .size(Size::Small)
                .ring_inner_border(true)
                .ring_outer_border(true),
            cx,
        );
        let ring_foreground_borders = RingDemo::spawn(
            ColorRingBuilder::new(
                "ring_foreground_borders",
                hsl.h,
                Arc::new(HueRingDelegate { saturation: hsl.s, lightness: hsl.l }),
            )
            .size(Size::Small)
            .ring_inner_border(true)
            .ring_outer_border(true)
            .ring_border_color(contrast_border),
            cx,
        );

        let ring_size_xsmall = RingDemo::spawn(
            ColorRingBuilder::new("ring_size_xsmall", hsl.h, Arc::new(HueRingDelegate { saturation: hsl.s, lightness: hsl.l }))
                .size(Size::XSmall),
            cx,
        );
        let ring_size_small = RingDemo::spawn(
            ColorRingBuilder::new("ring_size_small", hsl.h, Arc::new(HueRingDelegate { saturation: hsl.s, lightness: hsl.l }))
                .size(Size::Small),
            cx,
        );
        let ring_size_medium = RingDemo::spawn(
            ColorRingBuilder::new("ring_size_medium", hsl.h, Arc::new(HueRingDelegate { saturation: hsl.s, lightness: hsl.l }))
                .size(Size::Medium),
            cx,
        );
        let ring_size_large = RingDemo::spawn(
            ColorRingBuilder::new("ring_size_large", hsl.h, Arc::new(HueRingDelegate { saturation: hsl.s, lightness: hsl.l }))
                .size(Size::Large),
            cx,
        );

        let ring_thickness_xsmall = RingDemo::spawn(
            ColorRingBuilder::new(
                "ring_thickness_xsmall",
                hsl.h,
                Arc::new(HueRingDelegate { saturation: hsl.s, lightness: hsl.l }),
            )
            .size(Size::Medium)
            .ring_thickness_size(Size::XSmall),
            cx,
        );
        let ring_thickness_small = RingDemo::spawn(
            ColorRingBuilder::new(
                "ring_thickness_small",
                hsl.h,
                Arc::new(HueRingDelegate { saturation: hsl.s, lightness: hsl.l }),
            )
            .size(Size::Medium)
            .ring_thickness_size(Size::Small),
            cx,
        );
        let ring_thickness_medium = RingDemo::spawn(
            ColorRingBuilder::new(
                "ring_thickness_medium",
                hsl.h,
                Arc::new(HueRingDelegate { saturation: hsl.s, lightness: hsl.l }),
            )
            .size(Size::Medium)
            .ring_thickness_size(Size::Medium),
            cx,
        );
        let ring_thickness_large = RingDemo::spawn(
            ColorRingBuilder::new(
                "ring_thickness_large",
                hsl.h,
                Arc::new(HueRingDelegate { saturation: hsl.s, lightness: hsl.l }),
            )
            .size(Size::Medium)
            .ring_thickness_size(Size::Large),
            cx,
        );

        let ring_saturation = RingDemo::spawn(
            ColorRingBuilder::saturation("ring_saturation", hsl.s, hsl.h, hsv_value)
                .size(Size::Medium)
                .allow_inner_target(true),
            cx,
        );

        let ring_lightness = RingDemo::spawn(
            ColorRingBuilder::lightness("ring_lightness", hsl.l, hsl.h, hsl.s)
                .size(Size::Medium)
                .allow_inner_target(true),
            cx,
        );

        let mut pane = Self {
            color_ring,
            color_ring_vector_compare,
            color_ring_raster,
            color_ring_vector_compare_inner_target,
            color_ring_raster_inner_target,
            color_ring_saturation_vector,
            color_ring_saturation_vector_rotated,
            color_ring_saturation_raster,
            color_ring_saturation_raster_rotated,
            color_ring_lightness_vector,
            color_ring_lightness_raster,
            color_ring_disabled_vector,
            color_ring_disabled_raster,
            ring_no_border,
            ring_inner_border,
            ring_outer_border,
            ring_both_borders,
            ring_foreground_borders,
            ring_size_xsmall,
            ring_size_small,
            ring_size_medium,
            ring_size_large,
            ring_thickness_xsmall,
            ring_thickness_small,
            ring_thickness_medium,
            ring_thickness_large,
            ring_saturation,
            ring_lightness,
            ring_hsl: hsl,
            ring_color: hsla,
            ring_event_name: "Change",
            ring_event_value: hsl.h,
            ring_saturation_event_name: "Change",
            ring_saturation_event_value: hsl.s,
            ring_lightness_event_name: "Change",
            ring_lightness_event_value: hsl.l,
        };
        pane.sync_ring(cx, SyncSource::HueRing);
        pane
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.color_ring.slider, |app, _, event: &SliderEvent, cx| {
            app.panes.color_ring.handle_hue_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.ring_saturation.slider, |app, _, event: &SliderEvent, cx| {
            app.panes.color_ring.handle_saturation_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.ring_lightness.slider, |app, _, event: &SliderEvent, cx| {
            app.panes.color_ring.handle_lightness_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        let hue = self.ring_hsl.h;
        let saturation = self.ring_hsl.s;
        let lightness = self.ring_hsl.l;

        let chrome = look.chrome();
        let main = wrappanel! {
            gap=24 align=center;
            div()
                .size(px(SWATCH_SIZE))
                .flex_shrink_0()
                .rounded(px(8.0))
                .bg(self.ring_color)
                .border_1()
                .border_color(chrome.border),
            div()
                .size(px(RING_FRAME_PX))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .child(ring_event_overlay(
                    self.color_ring.slider.clone(),
                    self.ring_event_name,
                    self.ring_event_value,
                    look,
                )),
            div()
                .w(px(SLIDER_COLUMN_WIDTH))
                .flex_shrink_0()
                .child(vstack! {
                    gap=16;
                    div()
                        .size(px(RING_MEDIUM_PX))
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(ring_event_overlay(
                            self.ring_saturation.slider.clone(),
                            self.ring_saturation_event_name,
                            self.ring_saturation_event_value,
                            look,
                        )),
                    div()
                        .size(px(RING_MEDIUM_PX))
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(ring_event_overlay(
                            self.ring_lightness.slider.clone(),
                            self.ring_lightness_event_name,
                            self.ring_lightness_event_value,
                            look,
                        )),
                }),
            div()
                .w(px(READOUT_WIDTH))
                .flex_shrink_0()
                .child(render_color_readout(self.ring_color, look)),
        };

        color_gallery_pane(
            "Color Ring",
            "Ring story adapted from the original gallery with interactive HSL controls and renderer/size/thickness/border variants.",
            vstack! {
                gap=28;
                demo_section(
                    "Color Ring",
                    "Primary hue, saturation, and lightness rings linked as an HSL mixer.",
                    vec![demo_card(
                        "Interactive Sample",
                        "Drag the hue, saturation, or lightness rings to edit the shared color.",
                        WIDE_CARD,
                        main,
                        look,
                    )],
                    look,
                ),
                demo_section(
                    "Implementation Compare",
                    "Vector and raster rendering paths for the same hue ring.",
                    vec![demo_card(
                        "Renderer Compare",
                        "Vector paths vs raster pre-image for the same hue ring.",
                        WIDE_CARD,
                        variant_panel(vec![
                            render_variant(
                                "Vector (paths)",
                                self.color_ring_vector_compare.slider.clone(),
                                RING_MEDIUM_PX,
                                hue,
                                true,
                                look,
                            ),
                            render_variant(
                                "Vector (paths, inner target)",
                                self.color_ring_vector_compare_inner_target.slider.clone(),
                                RING_MEDIUM_PX,
                                hue,
                                true,
                                look,
                            ),
                            render_variant(
                                "Raster (pre-imaged)",
                                self.color_ring_raster.slider.clone(),
                                RING_MEDIUM_PX,
                                hue,
                                false,
                                look,
                            ),
                            render_variant(
                                "Raster (pre-imaged, inner target)",
                                self.color_ring_raster_inner_target.slider.clone(),
                                RING_MEDIUM_PX,
                                hue,
                                false,
                                look,
                            ),
                        ]),
                        look,
                    )],
                    look,
                ),
                demo_section(
                    "Saturation Ring (Continuous)",
                    "Mirrored saturation delegates with optional 180° rotation.",
                    vec![demo_card(
                        "Saturation Rings",
                        "Continuous saturation delegates with optional 180° rotation.",
                        WIDE_CARD,
                        variant_panel(vec![
                            render_variant(
                                "Vector saturation",
                                self.color_ring_saturation_vector.slider.clone(),
                                RING_MEDIUM_PX,
                                saturation,
                                true,
                                look,
                            ),
                            render_variant(
                                "Vector saturation (180deg)",
                                self.color_ring_saturation_vector_rotated.slider.clone(),
                                RING_MEDIUM_PX,
                                saturation,
                                true,
                                look,
                            ),
                            render_variant(
                                "Raster saturation",
                                self.color_ring_saturation_raster.slider.clone(),
                                RING_MEDIUM_PX,
                                saturation,
                                false,
                                look,
                            ),
                            render_variant(
                                "Raster saturation (180deg)",
                                self.color_ring_saturation_raster_rotated.slider.clone(),
                                RING_MEDIUM_PX,
                                saturation,
                                false,
                                look,
                            ),
                        ]),
                        look,
                    )],
                    look,
                ),
                demo_section(
                    "Lightness Ring (Continuous)",
                    "Mirrored lightness delegates in vector and raster modes.",
                    vec![demo_card(
                        "Lightness Rings",
                        "Mirrored lightness delegates in vector and raster modes.",
                        WIDE_CARD,
                        variant_panel(vec![
                            render_variant(
                                "Vector lightness",
                                self.color_ring_lightness_vector.slider.clone(),
                                RING_MEDIUM_PX,
                                lightness,
                                true,
                                look,
                            ),
                            render_variant(
                                "Raster lightness",
                                self.color_ring_lightness_raster.slider.clone(),
                                RING_MEDIUM_PX,
                                lightness,
                                false,
                                look,
                            ),
                        ]),
                        look,
                    )],
                    look,
                ),
                demo_section(
                    "Disabled",
                    "Hue rings with interaction disabled.",
                    vec![demo_card(
                        "Disabled",
                        "Hue rings with pointer interaction disabled.",
                        WIDE_CARD,
                        variant_panel(vec![
                            render_variant(
                                "Vector (disabled)",
                                self.color_ring_disabled_vector.slider.clone(),
                                RING_MEDIUM_PX,
                                hue,
                                true,
                                look,
                            ),
                            render_variant(
                                "Raster (disabled)",
                                self.color_ring_disabled_raster.slider.clone(),
                                RING_MEDIUM_PX,
                                hue,
                                false,
                                look,
                            ),
                        ]),
                        look,
                    )],
                    look,
                ),
                demo_section(
                    "Ring Border Variants",
                    "Inner, outer, and foreground border combinations.",
                    vec![demo_card(
                        "Border Variants",
                        "Inner, outer, and foreground border combinations.",
                        WIDE_CARD,
                        variant_panel(vec![
                            render_variant("no border", self.ring_no_border.slider.clone(), 120.0, hue, true, look),
                            render_variant("inner border", self.ring_inner_border.slider.clone(), 120.0, hue, true, look),
                            render_variant("outer border", self.ring_outer_border.slider.clone(), 120.0, hue, true, look),
                            render_variant("both", self.ring_both_borders.slider.clone(), 120.0, hue, true, look),
                            render_variant(
                                "foreground both",
                                self.ring_foreground_borders.slider.clone(),
                                120.0,
                                hue,
                                true,
                                look,
                            ),
                        ]),
                        look,
                    )],
                    look,
                ),
                demo_section(
                    "Ring Sizes",
                    "XSmall through Large ring footprints.",
                    vec![demo_card(
                        "Sizes",
                        "XSmall through Large ring footprints.",
                        WIDE_CARD,
                        variant_panel(vec![
                            render_variant("XSmall", self.ring_size_xsmall.slider.clone(), 80.0, hue, true, look),
                            render_variant("Small", self.ring_size_small.slider.clone(), 120.0, hue, true, look),
                            render_variant("Medium", self.ring_size_medium.slider.clone(), RING_MEDIUM_PX, hue, true, look),
                            render_variant("Large", self.ring_size_large.slider.clone(), 280.0, hue, true, look),
                        ]),
                        look,
                    )],
                    look,
                ),
                demo_section(
                    "Ring Thickness Sizes",
                    "Medium ring with XSmall through Large track thickness.",
                    vec![demo_card(
                        "Thickness",
                        "Medium ring with XSmall through Large track thickness.",
                        WIDE_CARD,
                        variant_panel(vec![
                            render_variant("XSmall", self.ring_thickness_xsmall.slider.clone(), RING_MEDIUM_PX, hue, true, look),
                            render_variant("Small", self.ring_thickness_small.slider.clone(), RING_MEDIUM_PX, hue, true, look),
                            render_variant("Medium", self.ring_thickness_medium.slider.clone(), RING_MEDIUM_PX, hue, true, look),
                            render_variant("Large", self.ring_thickness_large.slider.clone(), RING_MEDIUM_PX, hue, true, look),
                        ]),
                        look,
                    )],
                    look,
                ),
            }
            .into_any_element(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.color_ring.slider, cx);
        notify_entity(&self.color_ring_vector_compare.slider, cx);
        notify_entity(&self.color_ring_raster.slider, cx);
        notify_entity(&self.color_ring_vector_compare_inner_target.slider, cx);
        notify_entity(&self.color_ring_raster_inner_target.slider, cx);
        notify_entity(&self.color_ring_saturation_vector.slider, cx);
        notify_entity(&self.color_ring_saturation_vector_rotated.slider, cx);
        notify_entity(&self.color_ring_saturation_raster.slider, cx);
        notify_entity(&self.color_ring_saturation_raster_rotated.slider, cx);
        notify_entity(&self.color_ring_lightness_vector.slider, cx);
        notify_entity(&self.color_ring_lightness_raster.slider, cx);
        notify_entity(&self.color_ring_disabled_vector.slider, cx);
        notify_entity(&self.color_ring_disabled_raster.slider, cx);
        notify_entity(&self.ring_no_border.slider, cx);
        notify_entity(&self.ring_inner_border.slider, cx);
        notify_entity(&self.ring_outer_border.slider, cx);
        notify_entity(&self.ring_both_borders.slider, cx);
        notify_entity(&self.ring_foreground_borders.slider, cx);
        notify_entity(&self.ring_size_xsmall.slider, cx);
        notify_entity(&self.ring_size_small.slider, cx);
        notify_entity(&self.ring_size_medium.slider, cx);
        notify_entity(&self.ring_size_large.slider, cx);
        notify_entity(&self.ring_thickness_xsmall.slider, cx);
        notify_entity(&self.ring_thickness_small.slider, cx);
        notify_entity(&self.ring_thickness_medium.slider, cx);
        notify_entity(&self.ring_thickness_large.slider, cx);
        notify_entity(&self.ring_saturation.slider, cx);
        notify_entity(&self.ring_lightness.slider, cx);
    }

    fn handle_hue_event(&mut self, event: &SliderEvent, cx: &mut Context<GalleryApp>) {
        if let Some(value) = primary_slider_value(event) {
            self.ring_event_name = match event {
                SliderEvent::Change { .. } => "Change",
                SliderEvent::Release { .. } => "Release",
                _ => self.ring_event_name,
            };
            self.ring_event_value = value;
            self.ring_hsl.h = value;
            self.sync_ring(cx, SyncSource::HueRing);
        }
    }

    fn handle_saturation_event(&mut self, event: &SliderEvent, cx: &mut Context<GalleryApp>) {
        if let Some(value) = primary_slider_value(event) {
            self.ring_saturation_event_name = match event {
                SliderEvent::Change { .. } => "Change",
                SliderEvent::Release { .. } => "Release",
                _ => self.ring_saturation_event_name,
            };
            self.ring_saturation_event_value = value;
            self.ring_hsl.s = value;
            self.sync_ring(cx, SyncSource::SaturationRing);
        }
    }

    fn handle_lightness_event(&mut self, event: &SliderEvent, cx: &mut Context<GalleryApp>) {
        if let Some(value) = primary_slider_value(event) {
            self.ring_lightness_event_name = match event {
                SliderEvent::Change { .. } => "Change",
                SliderEvent::Release { .. } => "Release",
                _ => self.ring_lightness_event_name,
            };
            self.ring_lightness_event_value = value;
            self.ring_hsl.l = value;
            self.sync_ring(cx, SyncSource::LightnessRing);
        }
    }

    fn sync_ring(&mut self, cx: &mut Context<GalleryApp>, source: SyncSource) {
        let hsl = self.ring_hsl;
        let hsv_value = hsv_value_for_saturation_ring(hsl);
        self.ring_color = hsl.to_hsla();
        let lightness = hsl.to_hsla().l;

        if !matches!(source, SyncSource::HueRing) {
            self.color_ring.set_value(hsl.h, cx);
        }

        for ring in [
            &self.color_ring_vector_compare,
            &self.color_ring_raster,
            &self.color_ring_vector_compare_inner_target,
            &self.color_ring_raster_inner_target,
            &self.color_ring_disabled_vector,
            &self.color_ring_disabled_raster,
            &self.ring_no_border,
            &self.ring_inner_border,
            &self.ring_outer_border,
            &self.ring_both_borders,
            &self.ring_foreground_borders,
            &self.ring_size_xsmall,
            &self.ring_size_small,
            &self.ring_size_medium,
            &self.ring_size_large,
            &self.ring_thickness_xsmall,
            &self.ring_thickness_small,
            &self.ring_thickness_medium,
            &self.ring_thickness_large,
        ] {
            ring.set_value(hsl.h, cx);
        }

        for ring in [
            &self.color_ring,
            &self.color_ring_vector_compare,
            &self.color_ring_vector_compare_inner_target,
            &self.color_ring_disabled_vector,
            &self.ring_no_border,
            &self.ring_inner_border,
            &self.ring_outer_border,
            &self.ring_both_borders,
            &self.ring_foreground_borders,
            &self.ring_size_xsmall,
            &self.ring_size_small,
            &self.ring_size_medium,
            &self.ring_size_large,
            &self.ring_thickness_xsmall,
            &self.ring_thickness_small,
            &self.ring_thickness_medium,
            &self.ring_thickness_large,
        ] {
            ring.sync_hue_vector(hsl.s, lightness, cx);
        }

        for ring in [
            &self.color_ring_raster,
            &self.color_ring_raster_inner_target,
            &self.color_ring_disabled_raster,
        ] {
            ring.sync_hue_raster(hsl.s, lightness, cx);
        }

        if !matches!(source, SyncSource::SaturationRing) {
            self.ring_saturation.set_value(hsl.s, cx);
        }
        self.ring_saturation.sync_saturation(hsl.h, hsv_value, cx);

        if !matches!(source, SyncSource::LightnessRing) {
            self.ring_lightness.set_value(hsl.l, cx);
        }
        self.ring_lightness.sync_lightness(hsl.h, hsl.s, cx);

        for ring in [&self.color_ring_saturation_vector, &self.color_ring_saturation_vector_rotated] {
            ring.set_value(hsl.s, cx);
            ring.sync_saturation(hsl.h, hsv_value, cx);
        }
        for ring in [&self.color_ring_saturation_raster, &self.color_ring_saturation_raster_rotated] {
            ring.set_value(hsl.s, cx);
            ring.sync_saturation(hsl.h, 1.0, cx);
        }
        for ring in [&self.color_ring_lightness_vector, &self.color_ring_lightness_raster] {
            ring.set_value(hsl.l, cx);
            ring.sync_lightness(hsl.h, hsl.s, cx);
        }

        cx.notify();
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy)]
enum SyncSource {
    HueRing,
    SaturationRing,
    LightnessRing,
}

fn hsv_value_for_saturation_ring(hsl: Hsl) -> f32 {
    Hsv::from_hsla_ext(hsl.to_hsla()).v
}

fn ring_event_overlay(
    ring: Entity<SliderControl>,
    event_name: &'static str,
    event_value: f32,
    look: &ShadcnLook,
) -> impl IntoElement {
    let overlay_style = look.typography_scale(ShadcnTextSize::Xs);
    let value_style = look.typography_scale(ShadcnTextSize::Sm);

    div().relative().child(ring).child(
        div().absolute().inset_0().flex().items_center().justify_center().child(
            div()
                .w(px(84.0))
                .h(px(32.0))
                .rounded(px(4.0))
                .bg(black().opacity(0.45))
                .text_color(white())
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .child(div().typography_style(overlay_style).child(event_name))
                .child(div().typography_style(value_style).child(format_ring_value(event_value))),
        ),
    )
}

fn variant_panel(children: Vec<AnyElement>) -> AnyElement {
    let mut panel = wrappanel! { gap=24 align=center; };
    for child in children {
        panel = panel.child(child);
    }
    panel.into_any_element()
}

fn render_color_readout(color: gpui::Hsla, look: &ShadcnLook) -> AnyElement {
    vstack! {
        gap=8;
        detail_row("H", format!("{:.1}°", color.h * 360.0), look),
        detail_row("S", format!("{:.3}", color.s), look),
        detail_row("L", format!("{:.3}", color.l), look),
        detail_row("A", format!("{:.3}", color.a), look),
    }
    .into_any_element()
}

fn format_ring_value(value: f32) -> String {
    if value.abs() >= 10.0 {
        format!("{:.0}", value)
    } else {
        format!("{:.2}", value)
    }
}

fn ring_frame(frame_px: f32, ring: impl IntoElement, overlay: impl IntoElement) -> AnyElement {
    div()
        .relative()
        .size(px(frame_px))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .child(ring)
        .child(div().absolute().inset_0().flex().items_center().justify_center().child(overlay))
        .into_any_element()
}

fn render_variant(
    label: &'static str,
    circle: Entity<SliderControl>,
    frame_px: f32,
    value: f32,
    is_vector: bool,
    look: &ShadcnLook,
) -> AnyElement {
    let label_style = look.typography_scale(ShadcnTextSize::Xs);
    let display_label = if is_vector {
        format!("*{}", format_ring_value(value))
    } else {
        format_ring_value(value)
    };
    let overlay_style = look.typography_scale(ShadcnTextSize::Xs);

    vstack! {
        gap=8 align=center;
        div()
            .typography_style(label_style)
            .text_color(look.chrome().muted_text)
            .child(label),
        ring_frame(
            frame_px,
            circle,
            div()
                .w(px(48.0))
                .h(px(20.0))
                .rounded(px(4.0))
                .bg(black().opacity(0.45))
                .typography_style(overlay_style)
                .text_color(white())
                .flex()
                .items_center()
                .justify_center()
                .child(display_label),
        )
    }
    .into_any_element()
}
