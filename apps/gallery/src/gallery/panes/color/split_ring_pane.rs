use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, IntoElement, Subscription, div, hsla, prelude::*, px};
use gpui_luma::controls::color::color_arc::{ColorArcEvent, ColorArcRenderer, ColorArcState};
use gpui_luma::controls::color::color_arc::raster::RasterArcDelegate;
use gpui_luma::controls::color::color_ring::{ColorRingEvent, ColorRingState, HueRingDelegate};
use gpui_luma::controls::color::style::{Size, Sizable};
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{format_compact_hsla, format_hex_color, notify_entity};

use super::common::{color_gallery_pane, detail_row, notify_control};

#[derive(Clone)]
pub(in crate::gallery) struct SplitRingPane {
    state: Entity<SplitRingState>,
}

impl SplitRingPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let state = cx.new(|cx| SplitRingState::new(look, cx));
        Self { state }
    }

    pub(in crate::gallery) fn subscribe(&self, _cx: &mut Context<GalleryApp>, _subscriptions: &mut Vec<Subscription>) {}

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        color_gallery_pane(
            "Split Ring",
            "The Pixagram-style split-ring composition with a hue ring plus separate saturation and lightness arcs.",
            self.state.clone(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        self.state.update(cx, |state, cx| state.notify_controls(cx));
        notify_entity(&self.state, cx);
    }
}

struct SplitRingState {
    look: Arc<ShadcnLook>,
    saturation_arc: Entity<ColorArcState>,
    lightness_arc: Entity<ColorArcState>,
    hue_ring: Entity<ColorRingState>,
    hue_degrees: f32,
    saturation: f32,
    lightness: f32,
    _subscriptions: Vec<Subscription>,
}

impl SplitRingState {
    const OUTER_SIZE_PX: f32 = 300.0;
    const TRACK_WIDTH_PX: f32 = 20.0;
    const ARC_RING_GAP_PX: f32 = 5.0;
    const RING_SWATCH_GAP_PX: f32 = 14.0;
    const OUTER_PADDING_PX: f32 = 12.0;
    const BORDER_GAP_PX: f32 = 14.0;
    const ARC_GAP_DEGREES: f32 = 8.0;
    const ARC_ROTATION_DEGREES: f32 = 90.0;
    const ARC_SWEEP_DEGREES: f32 = 180.0 - Self::ARC_GAP_DEGREES;
    const ARC_HORIZONTAL_OFFSET_PX: f32 = 5.0;

    fn ring_outer_radius_px() -> f32 {
        (Self::OUTER_SIZE_PX * 0.5 - Self::TRACK_WIDTH_PX - Self::ARC_RING_GAP_PX).max(0.0)
    }

    fn frame_size_px() -> f32 {
        Self::OUTER_SIZE_PX + Self::ARC_HORIZONTAL_OFFSET_PX * 2.0
    }

    fn border_size_px() -> f32 {
        Self::frame_size_px() + Self::BORDER_GAP_PX * 2.0
    }

    fn canvas_size_px() -> f32 {
        Self::border_size_px() + Self::OUTER_PADDING_PX * 2.0
    }

    fn ring_size_px() -> f32 {
        Self::ring_outer_radius_px() * 2.0
    }

    fn swatch_size_px() -> f32 {
        let swatch_radius = (Self::ring_outer_radius_px() - Self::TRACK_WIDTH_PX - Self::RING_SWATCH_GAP_PX).max(0.0);
        swatch_radius * 2.0
    }

    fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let init_color = hsla(18.0 / 360.0, 0.85, 0.49, 1.0);
        let hue_degrees = init_color.h * 360.0;
        let saturation = init_color.s;
        let lightness = init_color.l;

        let saturation_arc = cx.new(|cx| {
            ColorArcState::saturation_with_renderer(
                "composition-split-ring-saturation-arc",
                saturation,
                hue_degrees,
                1.0,
                ColorArcRenderer::Raster,
                cx,
            )
            .with_size(Size::Size(px(Self::OUTER_SIZE_PX)))
            .start_degrees(90.0 + Self::ARC_ROTATION_DEGREES + Self::ARC_GAP_DEGREES * 0.5)
            .sweep_degrees(Self::ARC_SWEEP_DEGREES)
            .arc_thickness(Self::TRACK_WIDTH_PX)
            .thumb_size(Self::TRACK_WIDTH_PX)
        });
        let lightness_arc = cx.new(|cx| {
            ColorArcState::lightness_with_renderer(
                "composition-split-ring-lightness-arc",
                lightness,
                hue_degrees,
                saturation,
                ColorArcRenderer::Raster,
                cx,
            )
            .with_size(Size::Size(px(Self::OUTER_SIZE_PX)))
            .start_degrees(270.0 + Self::ARC_ROTATION_DEGREES + Self::ARC_GAP_DEGREES * 0.5)
            .sweep_degrees(Self::ARC_SWEEP_DEGREES)
            .arc_thickness(Self::TRACK_WIDTH_PX)
            .thumb_size(Self::TRACK_WIDTH_PX)
        });
        let hue_ring = cx.new(|cx| {
            ColorRingState::hue(
                "composition-split-ring-hue-ring",
                hue_degrees,
                HueRingDelegate { saturation, lightness },
                cx,
            )
            .with_size(Size::Size(px(Self::ring_size_px())))
            .ring_thickness(Self::TRACK_WIDTH_PX)
            .thumb_size(Self::TRACK_WIDTH_PX)
        });

        let subscriptions = vec![
            cx.subscribe(&hue_ring, |this, _, event: &ColorRingEvent, cx| {
                let hue = match event {
                    ColorRingEvent::Change(value) | ColorRingEvent::Release(value) => *value,
                };
                this.hue_degrees = hue;
                this.sync(cx);
                cx.notify();
            }),
            cx.subscribe(&saturation_arc, |this, _, event: &ColorArcEvent, cx| {
                let saturation = match event {
                    ColorArcEvent::Change(value) | ColorArcEvent::Release(value) => *value,
                };
                this.saturation = saturation.clamp(0.0, 1.0);
                this.sync(cx);
                cx.notify();
            }),
            cx.subscribe(&lightness_arc, |this, _, event: &ColorArcEvent, cx| {
                let lightness = match event {
                    ColorArcEvent::Change(value) | ColorArcEvent::Release(value) => *value,
                };
                this.lightness = lightness.clamp(0.0, 1.0);
                this.sync(cx);
                cx.notify();
            }),
        ];

        let mut this = Self {
            look,
            saturation_arc,
            lightness_arc,
            hue_ring,
            hue_degrees,
            saturation,
            lightness,
            _subscriptions: subscriptions,
        };
        this.sync(cx);
        this
    }

    fn notify_controls(&self, cx: &mut Context<Self>) {
        notify_control(&self.saturation_arc, cx);
        notify_control(&self.lightness_arc, cx);
        notify_control(&self.hue_ring, cx);
    }

    fn sync(&mut self, cx: &mut Context<Self>) {
        let hue_degrees = self.hue_degrees;
        let saturation = self.saturation;
        let lightness = self.lightness;

        self.hue_ring.update(cx, |ring, cx| {
            ring.set_value(hue_degrees, cx);
            ring.set_delegate(Box::new(HueRingDelegate { saturation, lightness }), cx);
        });
        self.saturation_arc.update(cx, |arc, cx| {
            arc.set_value(saturation, cx);
            arc.set_delegate(Box::new(RasterArcDelegate::saturation(hue_degrees, 1.0)), cx);
        });
        self.lightness_arc.update(cx, |arc, cx| {
            arc.set_value(lightness, cx);
            arc.set_delegate(Box::new(RasterArcDelegate::lightness(hue_degrees, saturation)), cx);
        });
    }
}

impl gpui::Render for SplitRingState {
    fn render(&mut self, _window: &mut gpui::Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let swatch_color = hsla((self.hue_degrees / 360.0).rem_euclid(1.0), self.saturation, self.lightness, 1.0);
        let ring_size = Self::ring_size_px();
        let swatch_size = Self::swatch_size_px();
        let group_width = Self::frame_size_px();
        let border_size = Self::border_size_px();
        let canvas_size = Self::canvas_size_px();
        let arc_offset = Self::ARC_HORIZONTAL_OFFSET_PX;

        let group_left = (canvas_size - group_width) * 0.5;
        let group_top = (canvas_size - Self::OUTER_SIZE_PX) * 0.5;
        let border_left = (canvas_size - border_size) * 0.5;
        let border_top = (canvas_size - border_size) * 0.5;
        let ring_offset = (Self::OUTER_SIZE_PX - ring_size) * 0.5;
        let swatch_offset = (Self::OUTER_SIZE_PX - swatch_size) * 0.5;

        div()
            .w_full()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(14.0))
            .child(
                div()
                    .relative()
                    .size(px(canvas_size))
                    .child(
                        div()
                            .absolute()
                            .left(px(border_left))
                            .top(px(border_top))
                            .size(px(border_size))
                            .rounded_full()
                            .border_1()
                            .border_color(self.look.chrome().border),
                    )
                    .child(
                        div()
                            .absolute()
                            .left(px(group_left))
                            .top(px(group_top))
                            .size(px(Self::OUTER_SIZE_PX))
                            .child(self.saturation_arc.clone()),
                    )
                    .child(
                        div()
                            .absolute()
                            .left(px(group_left + arc_offset * 2.0))
                            .top(px(group_top))
                            .size(px(Self::OUTER_SIZE_PX))
                            .child(self.lightness_arc.clone()),
                    )
                    .child(
                        div()
                            .absolute()
                            .left(px(group_left + arc_offset + ring_offset))
                            .top(px(group_top + ring_offset))
                            .size(px(ring_size))
                            .child(self.hue_ring.clone()),
                    )
                    .child(
                        div()
                            .absolute()
                            .left(px(group_left + arc_offset + swatch_offset))
                            .top(px(group_top + swatch_offset))
                            .size(px(swatch_size))
                            .rounded_full()
                            .bg(swatch_color)
                            .border_1()
                            .border_color(self.look.chrome().border),
                    ),
            )
            .child(
                div()
                    .w(px(Self::OUTER_SIZE_PX))
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .child(detail_row("Hex", format_hex_color(swatch_color), &self.look))
                    .child(detail_row("HSLA", format_compact_hsla(swatch_color), &self.look)),
            )
    }
}
