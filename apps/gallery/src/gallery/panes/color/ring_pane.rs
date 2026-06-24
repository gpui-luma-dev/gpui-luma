use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Subscription, black, div, hsla, prelude::*, px, white};
use gpui_luma::controls::color::color_ring::{
    ColorRingEvent, ColorRingModel, ColorRingRasterState, ColorRingRenderer, ColorRingState, HueRingDelegate,
    LightnessRingDelegate, SaturationRingDelegate,
};
use gpui_luma::controls::color::color_slider::color_spec::{Hsl, Hsv};
use gpui_luma::controls::color::color_slider::ColorSpecification;
use gpui_luma::controls::color::style::Size;
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
pub(in crate::gallery) struct ColorRingPane {
    color_ring: Entity<ColorRingState>,
    color_ring_vector_compare: Entity<ColorRingState>,
    color_ring_raster: Entity<ColorRingRasterState>,
    color_ring_vector_compare_inner_target: Entity<ColorRingState>,
    color_ring_raster_inner_target: Entity<ColorRingRasterState>,
    color_ring_saturation_vector: Entity<ColorRingState>,
    color_ring_saturation_vector_rotated: Entity<ColorRingState>,
    color_ring_saturation_raster: Entity<ColorRingRasterState>,
    color_ring_saturation_raster_rotated: Entity<ColorRingRasterState>,
    color_ring_lightness_vector: Entity<ColorRingState>,
    color_ring_lightness_raster: Entity<ColorRingRasterState>,
    color_ring_disabled_vector: Entity<ColorRingState>,
    color_ring_disabled_raster: Entity<ColorRingRasterState>,
    ring_no_border: Entity<ColorRingState>,
    ring_inner_border: Entity<ColorRingState>,
    ring_outer_border: Entity<ColorRingState>,
    ring_both_borders: Entity<ColorRingState>,
    ring_foreground_borders: Entity<ColorRingState>,
    ring_size_xsmall: Entity<ColorRingState>,
    ring_size_small: Entity<ColorRingState>,
    ring_size_medium: Entity<ColorRingState>,
    ring_size_large: Entity<ColorRingState>,
    ring_thickness_xsmall: Entity<ColorRingState>,
    ring_thickness_small: Entity<ColorRingState>,
    ring_thickness_medium: Entity<ColorRingState>,
    ring_thickness_large: Entity<ColorRingState>,
    ring_saturation: Entity<ColorRingState>,
    ring_lightness: Entity<ColorRingState>,
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

        let color_ring = cx.new(|cx| {
            ColorRingModel::new("color_ring", hsl.h, Box::new(HueRingDelegate { saturation: hsl.s, lightness: hsl.l }))
                .max(360.0)
                .size(Size::Medium)
                .allow_inner_target(true)
                .build(cx)
        });

        let color_ring_vector_compare = cx.new(|cx| {
            ColorRingState::hue_with_renderer(
                "color_ring_vector_compare",
                hsl.h,
                hsl.s,
                hsl.l,
                ColorRingRenderer::Vector,
                cx,
            )
            .size(Size::Medium)
        });
        let color_ring_raster = cx.new(|cx| {
            ColorRingState::hue_with_renderer("color_ring_raster", hsl.h, hsl.s, hsl.l, ColorRingRenderer::Raster, cx)
                .size(Size::Medium)
        });
        let color_ring_vector_compare_inner_target = cx.new(|cx| {
            ColorRingState::hue_with_renderer(
                "color_ring_vector_compare_inner_target",
                hsl.h,
                hsl.s,
                hsl.l,
                ColorRingRenderer::Vector,
                cx,
            )
            .size(Size::Medium)
            .allow_inner_target(true)
        });
        let color_ring_raster_inner_target = cx.new(|cx| {
            ColorRingState::hue_with_renderer(
                "color_ring_raster_inner_target",
                hsl.h,
                hsl.s,
                hsl.l,
                ColorRingRenderer::Raster,
                cx,
            )
            .size(Size::Medium)
            .allow_inner_target(true)
        });

        let color_ring_saturation_vector = cx.new(|cx| {
            ColorRingState::saturation_with_renderer(
                "color_ring_saturation_vector",
                hsl.s,
                hsl.h,
                hsv_value,
                ColorRingRenderer::Vector,
                cx,
            )
            .size(Size::Medium)
        });
        let color_ring_saturation_vector_rotated = cx.new(|cx| {
            ColorRingState::saturation_with_renderer(
                "color_ring_saturation_vector_rotated",
                hsl.s,
                hsl.h,
                hsv_value,
                ColorRingRenderer::Vector,
                cx,
            )
            .size(Size::Medium)
            .rotation_degrees(180.0)
        });
        let color_ring_saturation_raster = cx.new(|cx| {
            ColorRingState::saturation_with_renderer(
                "color_ring_saturation_raster",
                hsl.s,
                hsl.h,
                1.0,
                ColorRingRenderer::Raster,
                cx,
            )
            .size(Size::Medium)
        });
        let color_ring_saturation_raster_rotated = cx.new(|cx| {
            ColorRingState::saturation_with_renderer(
                "color_ring_saturation_raster_rotated",
                hsl.s,
                hsl.h,
                1.0,
                ColorRingRenderer::Raster,
                cx,
            )
            .size(Size::Medium)
            .rotation_degrees(180.0)
        });

        let color_ring_lightness_vector = cx.new(|cx| {
            ColorRingState::lightness_with_renderer(
                "color_ring_lightness_vector",
                hsl.l,
                hsl.h,
                hsl.s,
                ColorRingRenderer::Vector,
                cx,
            )
            .size(Size::Medium)
        });
        let color_ring_lightness_raster = cx.new(|cx| {
            ColorRingState::lightness_with_renderer(
                "color_ring_lightness_raster",
                hsl.l,
                hsl.h,
                hsl.s,
                ColorRingRenderer::Raster,
                cx,
            )
            .size(Size::Medium)
        });

        let color_ring_disabled_vector = cx.new(|cx| {
            ColorRingState::hue_with_renderer(
                "color_ring_disabled_vector",
                hsl.h,
                hsl.s,
                hsl.l,
                ColorRingRenderer::Vector,
                cx,
            )
            .size(Size::Medium)
            .enabled(false)
        });
        let color_ring_disabled_raster = cx.new(|cx| {
            ColorRingState::hue_with_renderer(
                "color_ring_disabled_raster",
                hsl.h,
                hsl.s,
                hsl.l,
                ColorRingRenderer::Raster,
                cx,
            )
            .size(Size::Medium)
            .enabled(false)
        });

        let ring_no_border = cx.new(|cx| {
            ColorRingState::hue("ring_no_border", hsl.h, HueRingDelegate { saturation: hsl.s, lightness: hsl.l }, cx)
                .size(Size::Small)
                .ring_inner_border(false)
                .ring_outer_border(false)
        });
        let ring_inner_border = cx.new(|cx| {
            ColorRingState::hue("ring_inner_border", hsl.h, HueRingDelegate { saturation: hsl.s, lightness: hsl.l }, cx)
                .size(Size::Small)
                .ring_inner_border(true)
                .ring_outer_border(false)
        });
        let ring_outer_border = cx.new(|cx| {
            ColorRingState::hue("ring_outer_border", hsl.h, HueRingDelegate { saturation: hsl.s, lightness: hsl.l }, cx)
                .size(Size::Small)
                .ring_inner_border(false)
                .ring_outer_border(true)
        });
        let ring_both_borders = cx.new(|cx| {
            ColorRingState::hue("ring_both_borders", hsl.h, HueRingDelegate { saturation: hsl.s, lightness: hsl.l }, cx)
                .size(Size::Small)
                .ring_inner_border(true)
                .ring_outer_border(true)
        });
        let ring_foreground_borders = cx.new(|cx| {
            ColorRingState::hue(
                "ring_foreground_borders",
                hsl.h,
                HueRingDelegate { saturation: hsl.s, lightness: hsl.l },
                cx,
            )
            .size(Size::Small)
            .ring_inner_border(true)
            .ring_outer_border(true)
            .ring_border_color(contrast_border)
        });

        let ring_size_xsmall = cx.new(|cx| {
            ColorRingState::hue("ring_size_xsmall", hsl.h, HueRingDelegate { saturation: hsl.s, lightness: hsl.l }, cx)
                .size(Size::XSmall)
        });
        let ring_size_small = cx.new(|cx| {
            ColorRingState::hue("ring_size_small", hsl.h, HueRingDelegate { saturation: hsl.s, lightness: hsl.l }, cx)
                .size(Size::Small)
        });
        let ring_size_medium = cx.new(|cx| {
            ColorRingState::hue("ring_size_medium", hsl.h, HueRingDelegate { saturation: hsl.s, lightness: hsl.l }, cx)
                .size(Size::Medium)
        });
        let ring_size_large = cx.new(|cx| {
            ColorRingState::hue("ring_size_large", hsl.h, HueRingDelegate { saturation: hsl.s, lightness: hsl.l }, cx)
                .size(Size::Large)
        });

        let ring_thickness_xsmall = cx.new(|cx| {
            ColorRingState::hue(
                "ring_thickness_xsmall",
                hsl.h,
                HueRingDelegate { saturation: hsl.s, lightness: hsl.l },
                cx,
            )
            .size(Size::Medium)
            .ring_thickness_size(Size::XSmall)
        });
        let ring_thickness_small = cx.new(|cx| {
            ColorRingState::hue(
                "ring_thickness_small",
                hsl.h,
                HueRingDelegate { saturation: hsl.s, lightness: hsl.l },
                cx,
            )
            .size(Size::Medium)
            .ring_thickness_size(Size::Small)
        });
        let ring_thickness_medium = cx.new(|cx| {
            ColorRingState::hue(
                "ring_thickness_medium",
                hsl.h,
                HueRingDelegate { saturation: hsl.s, lightness: hsl.l },
                cx,
            )
            .size(Size::Medium)
            .ring_thickness_size(Size::Medium)
        });
        let ring_thickness_large = cx.new(|cx| {
            ColorRingState::hue(
                "ring_thickness_large",
                hsl.h,
                HueRingDelegate { saturation: hsl.s, lightness: hsl.l },
                cx,
            )
            .size(Size::Medium)
            .ring_thickness_size(Size::Large)
        });

        let ring_saturation = cx.new(|cx| {
            ColorRingModel::new("ring_saturation", hsl.s, Box::new(SaturationRingDelegate { hue: hsl.h, hsv_value }))
                .size(Size::Medium)
                .allow_inner_target(true)
                .build(cx)
        });

        let ring_lightness = cx.new(|cx| {
            ColorRingModel::new(
                "ring_lightness",
                hsl.l,
                Box::new(LightnessRingDelegate { hue: hsl.h, saturation: hsl.s }),
            )
            .size(Size::Medium)
            .allow_inner_target(true)
            .build(cx)
        });

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
        subscriptions.push(cx.subscribe(&self.color_ring, |app, _, event: &ColorRingEvent, cx| {
            let (event_name, value) = match event {
                ColorRingEvent::Change(value) => ("Change", *value),
                ColorRingEvent::Release(value) => ("Release", *value),
            };
            app.panes.color_ring.ring_event_name = event_name;
            app.panes.color_ring.ring_event_value = value;
            app.panes.color_ring.ring_hsl.h = value;
            app.panes.color_ring.sync_ring_from_hue_ring(cx);
        }));
        subscriptions.push(cx.subscribe(&self.ring_saturation, |app, _, event: &ColorRingEvent, cx| {
            let (event_name, value) = match event {
                ColorRingEvent::Change(value) => ("Change", *value),
                ColorRingEvent::Release(value) => ("Release", *value),
            };
            app.panes.color_ring.ring_saturation_event_name = event_name;
            app.panes.color_ring.ring_saturation_event_value = value;
            app.panes.color_ring.ring_hsl.s = value;
            app.panes.color_ring.sync_ring_from_saturation_ring(cx);
        }));
        subscriptions.push(cx.subscribe(&self.ring_lightness, |app, _, event: &ColorRingEvent, cx| {
            let (event_name, value) = match event {
                ColorRingEvent::Change(value) => ("Change", *value),
                ColorRingEvent::Release(value) => ("Release", *value),
            };
            app.panes.color_ring.ring_lightness_event_name = event_name;
            app.panes.color_ring.ring_lightness_event_value = value;
            app.panes.color_ring.ring_hsl.l = value;
            app.panes.color_ring.sync_ring_from_lightness_ring(cx);
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
                    self.color_ring.clone(),
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
                            self.ring_saturation.clone(),
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
                            self.ring_lightness.clone(),
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
                            render_thumb_variant(
                                "Vector (paths)",
                                self.color_ring_vector_compare.clone(),
                                RING_MEDIUM_PX,
                                hue,
                                look,
                            ),
                            render_thumb_variant(
                                "Vector (paths, inner target)",
                                self.color_ring_vector_compare_inner_target.clone(),
                                RING_MEDIUM_PX,
                                hue,
                                look,
                            ),
                            render_raster_variant(
                                "Raster (pre-imaged)",
                                self.color_ring_raster.clone(),
                                RING_MEDIUM_PX,
                                hue,
                                look,
                            ),
                            render_raster_variant(
                                "Raster (pre-imaged, inner target)",
                                self.color_ring_raster_inner_target.clone(),
                                RING_MEDIUM_PX,
                                hue,
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
                            render_thumb_variant(
                                "Vector saturation",
                                self.color_ring_saturation_vector.clone(),
                                RING_MEDIUM_PX,
                                saturation,
                                look,
                            ),
                            render_thumb_variant(
                                "Vector saturation (180deg)",
                                self.color_ring_saturation_vector_rotated.clone(),
                                RING_MEDIUM_PX,
                                saturation,
                                look,
                            ),
                            render_raster_variant(
                                "Raster saturation",
                                self.color_ring_saturation_raster.clone(),
                                RING_MEDIUM_PX,
                                saturation,
                                look,
                            ),
                            render_raster_variant(
                                "Raster saturation (180deg)",
                                self.color_ring_saturation_raster_rotated.clone(),
                                RING_MEDIUM_PX,
                                saturation,
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
                            render_thumb_variant(
                                "Vector lightness",
                                self.color_ring_lightness_vector.clone(),
                                RING_MEDIUM_PX,
                                lightness,
                                look,
                            ),
                            render_raster_variant(
                                "Raster lightness",
                                self.color_ring_lightness_raster.clone(),
                                RING_MEDIUM_PX,
                                lightness,
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
                            render_thumb_variant(
                                "Vector (disabled)",
                                self.color_ring_disabled_vector.clone(),
                                RING_MEDIUM_PX,
                                hue,
                                look,
                            ),
                            render_raster_variant(
                                "Raster (disabled)",
                                self.color_ring_disabled_raster.clone(),
                                RING_MEDIUM_PX,
                                hue,
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
                            render_thumb_variant("no border", self.ring_no_border.clone(), 120.0, hue, look),
                            render_thumb_variant("inner border", self.ring_inner_border.clone(), 120.0, hue, look),
                            render_thumb_variant("outer border", self.ring_outer_border.clone(), 120.0, hue, look),
                            render_thumb_variant("both", self.ring_both_borders.clone(), 120.0, hue, look),
                            render_thumb_variant(
                                "foreground both",
                                self.ring_foreground_borders.clone(),
                                120.0,
                                hue,
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
                            render_thumb_variant("XSmall", self.ring_size_xsmall.clone(), 80.0, hue, look),
                            render_thumb_variant("Small", self.ring_size_small.clone(), 120.0, hue, look),
                            render_thumb_variant("Medium", self.ring_size_medium.clone(), RING_MEDIUM_PX, hue, look),
                            render_thumb_variant("Large", self.ring_size_large.clone(), 280.0, hue, look),
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
                            render_thumb_variant("XSmall", self.ring_thickness_xsmall.clone(), RING_MEDIUM_PX, hue, look),
                            render_thumb_variant("Small", self.ring_thickness_small.clone(), RING_MEDIUM_PX, hue, look),
                            render_thumb_variant("Medium", self.ring_thickness_medium.clone(), RING_MEDIUM_PX, hue, look),
                            render_thumb_variant("Large", self.ring_thickness_large.clone(), RING_MEDIUM_PX, hue, look),
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
        notify_entity(&self.color_ring, cx);
        notify_entity(&self.color_ring_vector_compare, cx);
        notify_entity(&self.color_ring_raster, cx);
        notify_entity(&self.color_ring_vector_compare_inner_target, cx);
        notify_entity(&self.color_ring_raster_inner_target, cx);
        notify_entity(&self.color_ring_saturation_vector, cx);
        notify_entity(&self.color_ring_saturation_vector_rotated, cx);
        notify_entity(&self.color_ring_saturation_raster, cx);
        notify_entity(&self.color_ring_saturation_raster_rotated, cx);
        notify_entity(&self.color_ring_lightness_vector, cx);
        notify_entity(&self.color_ring_lightness_raster, cx);
        notify_entity(&self.color_ring_disabled_vector, cx);
        notify_entity(&self.color_ring_disabled_raster, cx);
        notify_entity(&self.ring_no_border, cx);
        notify_entity(&self.ring_inner_border, cx);
        notify_entity(&self.ring_outer_border, cx);
        notify_entity(&self.ring_both_borders, cx);
        notify_entity(&self.ring_foreground_borders, cx);
        notify_entity(&self.ring_size_xsmall, cx);
        notify_entity(&self.ring_size_small, cx);
        notify_entity(&self.ring_size_medium, cx);
        notify_entity(&self.ring_size_large, cx);
        notify_entity(&self.ring_thickness_xsmall, cx);
        notify_entity(&self.ring_thickness_small, cx);
        notify_entity(&self.ring_thickness_medium, cx);
        notify_entity(&self.ring_thickness_large, cx);
        notify_entity(&self.ring_saturation, cx);
        notify_entity(&self.ring_lightness, cx);
    }

    fn sync_ring_from_hue_ring(&mut self, cx: &mut Context<GalleryApp>) {
        self.sync_ring(cx, SyncSource::HueRing);
    }

    fn sync_ring_from_saturation_ring(&mut self, cx: &mut Context<GalleryApp>) {
        self.sync_ring(cx, SyncSource::SaturationRing);
    }

    fn sync_ring_from_lightness_ring(&mut self, cx: &mut Context<GalleryApp>) {
        self.sync_ring(cx, SyncSource::LightnessRing);
    }

    fn sync_ring(&mut self, cx: &mut Context<GalleryApp>, source: SyncSource) {
        let hsl = self.ring_hsl;
        let hsv_value = hsv_value_for_saturation_ring(hsl);
        self.ring_color = hsl.to_hsla();

        let hue_delegate = || HueRingDelegate { saturation: hsl.s, lightness: hsl.l };

        self.color_ring.update(cx, |ring, cx| {
            ring.set_delegate(Box::new(hue_delegate()), cx);
            if !matches!(source, SyncSource::HueRing) {
                ring.set_value(hsl.h, cx);
            }
        });

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
            ring.update(cx, |ring, cx| {
                ring.set_delegate(Box::new(hue_delegate()), cx);
                ring.set_value(hsl.h, cx);
            });
        }

        self.ring_saturation.update(cx, |ring, cx| {
            let delegate = SaturationRingDelegate { hue: hsl.h, hsv_value };
            ring.set_delegate(Box::new(delegate), cx);
            if !matches!(source, SyncSource::SaturationRing) {
                ring.set_value(hsl.s, cx);
            }
        });
        self.ring_lightness.update(cx, |ring, cx| {
            let delegate = LightnessRingDelegate { hue: hsl.h, saturation: hsl.s };
            ring.set_delegate(Box::new(delegate), cx);
            if !matches!(source, SyncSource::LightnessRing) {
                ring.set_value(hsl.l, cx);
            }
        });

        for ring in [&self.color_ring_saturation_vector, &self.color_ring_saturation_vector_rotated] {
            let delegate = SaturationRingDelegate { hue: hsl.h, hsv_value };
            ring.update(cx, |ring, cx| {
                ring.set_delegate(Box::new(delegate), cx);
                ring.set_value(hsl.s, cx);
            });
        }
        for ring in [&self.color_ring_saturation_raster, &self.color_ring_saturation_raster_rotated] {
            let delegate = SaturationRingDelegate { hue: hsl.h, hsv_value: 1.0 };
            ring.update(cx, |ring, cx| {
                ring.set_delegate(Box::new(delegate), cx);
                ring.set_value(hsl.s, cx);
            });
        }
        for ring in [&self.color_ring_lightness_vector, &self.color_ring_lightness_raster] {
            let delegate = LightnessRingDelegate { hue: hsl.h, saturation: hsl.s };
            ring.update(cx, |ring, cx| {
                ring.set_delegate(Box::new(delegate), cx);
                ring.set_value(hsl.l, cx);
            });
        }

        cx.notify();
    }
}

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
    ring: Entity<ColorRingState>,
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

fn render_ring_with_value(
    circle: Entity<ColorRingState>,
    frame_px: f32,
    value: f32,
    is_vector: bool,
    look: &ShadcnLook,
) -> AnyElement {
    let label = if is_vector {
        format!("*{}", format_ring_value(value))
    } else {
        format_ring_value(value)
    };
    let overlay_style = look.typography_scale(ShadcnTextSize::Xs);

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
            .child(label),
    )
}

fn render_raster_ring_with_value(
    circle: Entity<ColorRingRasterState>,
    frame_px: f32,
    value: f32,
    look: &ShadcnLook,
) -> AnyElement {
    let overlay_style = look.typography_scale(ShadcnTextSize::Xs);

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
            .child(format_ring_value(value)),
    )
}

fn render_thumb_variant(
    label: &'static str,
    circle: Entity<ColorRingState>,
    frame_px: f32,
    value: f32,
    look: &ShadcnLook,
) -> AnyElement {
    let label_style = look.typography_scale(ShadcnTextSize::Xs);

    vstack! {
        gap=8 align=center;
        div()
            .typography_style(label_style)
            .text_color(look.chrome().muted_text)
            .child(label),
        render_ring_with_value(circle, frame_px, value, true, look),
    }
    .into_any_element()
}

fn render_raster_variant(
    label: &'static str,
    circle: Entity<ColorRingRasterState>,
    frame_px: f32,
    value: f32,
    look: &ShadcnLook,
) -> AnyElement {
    let label_style = look.typography_scale(ShadcnTextSize::Xs);

    vstack! {
        gap=8 align=center;
        div()
            .typography_style(label_style)
            .text_color(look.chrome().muted_text)
            .child(label),
        render_raster_ring_with_value(circle, frame_px, value, look),
    }
    .into_any_element()
}
