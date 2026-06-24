use std::sync::Arc;

use gpui::{Context, Entity, IntoElement, Subscription, div, hsla, prelude::*, px};
use gpui_luma::controls::color::color_arc::{
    ColorArcBuilder, ColorArcDomainRenderer, ColorArcRenderer, ColorArcTrackContext, RasterArcDelegate,
    refresh_color_arc, update_arc_delegate,
};
use gpui_luma::controls::color::color_slider::color_spec::Hsv;
use gpui_luma::controls::color::color_ring::{
    ColorRingBuilder, ColorRingDomainRenderer, ColorRingTrackContext, HueRingDelegate, primary_slider_value,
    refresh_color_ring, update_ring_delegate,
};
use gpui_luma::controls::color::style::Size;
use gpui_luma::controls::slider::{SliderControl, SliderEvent};
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::panes::shared::{format_compact_hsla, format_hex_color};

use crate::gallery::panes::color::common::{detail_row, notify_control};

pub(in crate::gallery) struct SplitRingState {
    look: Arc<ShadcnLook>,
    saturation_arc: Entity<SliderControl>,
    saturation_renderer: Arc<ColorArcDomainRenderer>,
    saturation_context: ColorArcTrackContext,
    lightness_arc: Entity<SliderControl>,
    lightness_renderer: Arc<ColorArcDomainRenderer>,
    lightness_context: ColorArcTrackContext,
    hue_ring: Entity<SliderControl>,
    hue_ring_renderer: Arc<ColorRingDomainRenderer>,
    hue_ring_context: ColorRingTrackContext,
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

    pub(in crate::gallery) fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let init_color = hsla(18.0 / 360.0, 0.85, 0.49, 1.0);
        let hue_degrees = init_color.h * 360.0;
        let saturation = init_color.s;
        let lightness = init_color.l;
        let hsv_value = Hsv::from_hsla_ext(init_color).v;

        let saturation_builder = ColorArcBuilder::saturation_with_renderer(
            "composition-split-ring-saturation-arc",
            saturation,
            hue_degrees,
            hsv_value,
            ColorArcRenderer::Raster,
        )
        .range(0.0..1.0)
        .step(0.001)
        .size(Size::Size(px(Self::OUTER_SIZE_PX)))
        .start_degrees(90.0 + Self::ARC_ROTATION_DEGREES + Self::ARC_GAP_DEGREES * 0.5)
        .sweep_degrees(Self::ARC_SWEEP_DEGREES)
        .arc_thickness(Self::TRACK_WIDTH_PX)
        .thumb_size(Self::TRACK_WIDTH_PX);
        let saturation_renderer = saturation_builder.domain_renderer();
        let saturation_context = saturation_builder.track_context();
        let saturation_arc = saturation_builder.spawn(cx);

        let lightness_builder = ColorArcBuilder::lightness_with_renderer(
            "composition-split-ring-lightness-arc",
            lightness,
            hue_degrees,
            saturation,
            ColorArcRenderer::Raster,
        )
        .range(0.0..1.0)
        .step(0.001)
        .size(Size::Size(px(Self::OUTER_SIZE_PX)))
        .start_degrees(270.0 + Self::ARC_ROTATION_DEGREES + Self::ARC_GAP_DEGREES * 0.5)
        .sweep_degrees(Self::ARC_SWEEP_DEGREES)
        .arc_thickness(Self::TRACK_WIDTH_PX)
        .thumb_size(Self::TRACK_WIDTH_PX);
        let lightness_renderer = lightness_builder.domain_renderer();
        let lightness_context = lightness_builder.track_context();
        let lightness_arc = lightness_builder.spawn(cx);
        let hue_ring_builder =
            ColorRingBuilder::hue("composition-split-ring-hue-ring", hue_degrees, saturation, lightness)
                .size(Size::Size(px(Self::ring_size_px())))
                .ring_thickness(Self::TRACK_WIDTH_PX)
                .thumb_size(Self::TRACK_WIDTH_PX);
        let hue_ring_renderer = hue_ring_builder.domain_renderer();
        let hue_ring_context = hue_ring_builder.track_context();
        let hue_ring = hue_ring_builder.spawn(cx);

        let subscriptions = vec![
            cx.subscribe(&hue_ring, |this, _, event: &SliderEvent, cx| {
                let Some(hue) = primary_slider_value(event) else {
                    return;
                };
                this.hue_degrees = hue;
                this.sync(cx, SyncSource::HueRing);
                cx.notify();
            }),
            cx.subscribe(&saturation_arc, |this, _, event: &SliderEvent, cx| {
                let saturation = match event {
                    SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } => *value,
                    _ => return,
                };
                this.saturation = saturation.clamp(0.0, 1.0);
                this.sync(cx, SyncSource::SaturationArc);
                cx.notify();
            }),
            cx.subscribe(&lightness_arc, |this, _, event: &SliderEvent, cx| {
                let lightness = match event {
                    SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } => *value,
                    _ => return,
                };
                this.lightness = lightness.clamp(0.0, 1.0);
                this.sync(cx, SyncSource::LightnessArc);
                cx.notify();
            }),
        ];

        let mut this = Self {
            look,
            saturation_arc,
            saturation_renderer,
            saturation_context,
            lightness_arc,
            lightness_renderer,
            lightness_context,
            hue_ring,
            hue_ring_renderer,
            hue_ring_context,
            hue_degrees,
            saturation,
            lightness,
            _subscriptions: subscriptions,
        };
        this.sync(cx, SyncSource::External);
        this
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<Self>) {
        notify_control(&self.saturation_arc, cx);
        notify_control(&self.lightness_arc, cx);
        notify_control(&self.hue_ring, cx);
    }

    fn sync(&mut self, cx: &mut Context<Self>, source: SyncSource) {
        let hue_degrees = self.hue_degrees;
        let saturation = self.saturation;
        let lightness = self.lightness;
        let hsv_value = Hsv::from_hsla_ext(hsla((hue_degrees / 360.0).rem_euclid(1.0), saturation, lightness, 1.0)).v;

        if matches!(source, SyncSource::External) {
            self.hue_ring.update(cx, |ring, cx| ring.set_value(hue_degrees, cx));
        }
        update_ring_delegate(
            &self.hue_ring_renderer,
            Arc::new(HueRingDelegate { saturation, lightness }),
            self.hue_ring_context.clone(),
        );
        refresh_color_ring(&self.hue_ring, cx);
        if matches!(source, SyncSource::External) {
            self.saturation_arc.update(cx, |arc, cx| arc.set_value(saturation, cx));
        }
        update_arc_delegate(
            &self.saturation_renderer,
            Arc::new(RasterArcDelegate::saturation(hue_degrees, hsv_value)),
            self.saturation_context.clone(),
        );
        refresh_color_arc(&self.saturation_arc, cx);

        if matches!(source, SyncSource::External) {
            self.lightness_arc.update(cx, |arc, cx| arc.set_value(lightness, cx));
        }
        update_arc_delegate(
            &self.lightness_renderer,
            Arc::new(RasterArcDelegate::lightness(hue_degrees, saturation)),
            self.lightness_context.clone(),
        );
        refresh_color_arc(&self.lightness_arc, cx);
    }
}

#[derive(Clone, Copy)]
enum SyncSource {
    External,
    HueRing,
    SaturationArc,
    LightnessArc,
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
                            .left(px(group_left + arc_offset + ring_offset))
                            .top(px(group_top + ring_offset))
                            .size(px(ring_size))
                            .child(self.hue_ring.clone()),
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
