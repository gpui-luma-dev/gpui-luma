use std::sync::Arc;

use gpui::{Context, Entity, IntoElement, Subscription, div, hsla, prelude::*, px};
use gpui_luma::controls::color::composition::ColorCompositionSync;
use gpui_luma::controls::color::color_arc::{
    ColorArcBuilder, ColorArcDomainRenderer, ColorArcRenderer, ColorArcTrackContext, RasterArcDelegate,
};
use gpui_luma::controls::color::color_slider::color_spec::Hsv;
use gpui_luma::controls::color::color_ring::{
    ColorRingBuilder, ColorRingDomainRenderer, ColorRingTrackContext, HueRingDelegate, primary_slider_value,
};
use gpui_luma::controls::color::style::Size;
use gpui_luma::controls::slider::SliderControl;
use gpui_luma_look_shadcn::ShadcnLook;

use super::CompositionSize;
use crate::gallery::panes::shared::{format_compact_hsla, format_hex_color};

use crate::gallery::panes::color::common::{detail_row, notify_control};

pub(in crate::gallery) struct SplitRingState {
    look: Arc<ShadcnLook>,
    metrics: SplitRingMetrics,
    sync: ColorCompositionSync,
    saturation_arc: Entity<SliderControl>,
    saturation_renderer: Arc<ColorArcDomainRenderer>,
    saturation_context: ColorArcTrackContext,
    lightness_arc: Entity<SliderControl>,
    lightness_renderer: Arc<ColorArcDomainRenderer>,
    lightness_context: ColorArcTrackContext,
    hue_ring: Entity<SliderControl>,
    hue_ring_renderer: Arc<ColorRingDomainRenderer>,
    hue_ring_context: ColorRingTrackContext,
    color: SplitRingColorState,
    _subscriptions: Vec<Subscription>,
}

#[derive(Clone, Copy)]
struct SplitRingColorState {
    hue_degrees: f32,
    saturation: f32,
    lightness: f32,
}

#[derive(Clone, Copy)]
struct SplitRingMetrics {
    outer_size: f32,
    track_width: f32,
    arc_ring_gap: f32,
    ring_swatch_gap: f32,
    outer_padding: f32,
    border_gap: f32,
    arc_horizontal_offset: f32,
}

impl SplitRingMetrics {
    const ARC_GAP_DEGREES: f32 = 8.0;
    const ARC_ROTATION_DEGREES: f32 = 90.0;

    fn resolve(size: CompositionSize) -> Self {
        let outer_size = size.resolve_primary(220.0, 300.0, 380.0);
        let scale = outer_size / 300.0;
        Self {
            outer_size,
            track_width: (20.0 * scale).max(12.0),
            arc_ring_gap: (5.0 * scale).max(3.0),
            ring_swatch_gap: (14.0 * scale).max(8.0),
            outer_padding: (12.0 * scale).max(8.0),
            border_gap: (14.0 * scale).max(10.0),
            arc_horizontal_offset: (5.0 * scale).max(3.0),
        }
    }

    fn arc_sweep_degrees(self) -> f32 {
        180.0 - Self::ARC_GAP_DEGREES
    }

    fn ring_outer_radius(self) -> f32 {
        (self.outer_size * 0.5 - self.track_width - self.arc_ring_gap).max(0.0)
    }

    fn frame_size(self) -> f32 {
        self.outer_size + self.arc_horizontal_offset * 2.0
    }

    fn border_size(self) -> f32 {
        self.frame_size() + self.border_gap * 2.0
    }

    fn canvas_size(self) -> f32 {
        self.border_size() + self.outer_padding * 2.0
    }

    fn ring_size(self) -> f32 {
        self.ring_outer_radius() * 2.0
    }

    fn swatch_size(self) -> f32 {
        let swatch_radius = (self.ring_outer_radius() - self.track_width - self.ring_swatch_gap).max(0.0);
        swatch_radius * 2.0
    }
}

impl SplitRingState {
    #[allow(dead_code)]
    pub(in crate::gallery) fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        Self::with_size(look, CompositionSize::Md, cx)
    }

    pub(in crate::gallery) fn with_size(look: Arc<ShadcnLook>, size: CompositionSize, cx: &mut Context<Self>) -> Self {
        let init_color = hsla(18.0 / 360.0, 0.85, 0.49, 1.0);
        let hue_degrees = init_color.h * 360.0;
        let saturation = init_color.s;
        let lightness = init_color.l;
        let hsv_value = Hsv::from_hsla_ext(init_color).v;
        let metrics = SplitRingMetrics::resolve(size);

        let saturation_builder = ColorArcBuilder::saturation_with_renderer(
            "composition-split-ring-saturation-arc",
            saturation,
            hue_degrees,
            hsv_value,
            ColorArcRenderer::Raster,
        )
        .range(0.0..1.0)
        .step(0.001)
        .size(Size::Size(px(metrics.outer_size)))
        .start_degrees(90.0 + SplitRingMetrics::ARC_ROTATION_DEGREES + SplitRingMetrics::ARC_GAP_DEGREES * 0.5)
        .sweep_degrees(metrics.arc_sweep_degrees())
        .arc_thickness(metrics.track_width)
        .thumb_size(metrics.track_width);
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
        .size(Size::Size(px(metrics.outer_size)))
        .start_degrees(270.0 + SplitRingMetrics::ARC_ROTATION_DEGREES + SplitRingMetrics::ARC_GAP_DEGREES * 0.5)
        .sweep_degrees(metrics.arc_sweep_degrees())
        .arc_thickness(metrics.track_width)
        .thumb_size(metrics.track_width);
        let lightness_renderer = lightness_builder.domain_renderer();
        let lightness_context = lightness_builder.track_context();
        let lightness_arc = lightness_builder.spawn(cx);
        let hue_ring_builder =
            ColorRingBuilder::hue("composition-split-ring-hue-ring", hue_degrees, saturation, lightness)
                .size(Size::Size(px(metrics.ring_size())))
                .ring_thickness(metrics.track_width)
                .thumb_size(metrics.track_width);
        let hue_ring_renderer = hue_ring_builder.domain_renderer();
        let hue_ring_context = hue_ring_builder.track_context();
        let hue_ring = hue_ring_builder.spawn(cx);

        let subscriptions = vec![
            cx.subscribe(&hue_ring, |this, _, event, cx| {
                let Some(hue) = primary_slider_value(event) else {
                    return;
                };
                if !this.sync.begin_sync() {
                    return;
                }
                this.color.hue_degrees = hue;
                this.sync_controls(cx);
                this.sync.end_sync();
                cx.notify();
            }),
            cx.subscribe(&saturation_arc, |this, _, event, cx| {
                let Some(saturation) = primary_slider_value(event) else {
                    return;
                };
                if !this.sync.begin_sync() {
                    return;
                }
                this.color.saturation = saturation.clamp(0.0, 1.0);
                this.sync_controls(cx);
                this.sync.end_sync();
                cx.notify();
            }),
            cx.subscribe(&lightness_arc, |this, _, event, cx| {
                let Some(lightness) = primary_slider_value(event) else {
                    return;
                };
                if !this.sync.begin_sync() {
                    return;
                }
                this.color.lightness = lightness.clamp(0.0, 1.0);
                this.sync_controls(cx);
                this.sync.end_sync();
                cx.notify();
            }),
        ];

        let this = Self {
            look,
            metrics,
            sync: ColorCompositionSync::new(),
            saturation_arc,
            saturation_renderer,
            saturation_context,
            lightness_arc,
            lightness_renderer,
            lightness_context,
            hue_ring,
            hue_ring_renderer,
            hue_ring_context,
            color: SplitRingColorState { hue_degrees, saturation, lightness },
            _subscriptions: subscriptions,
        };
        this.sync_controls(cx);
        this
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<Self>) {
        notify_control(&self.saturation_arc, cx);
        notify_control(&self.lightness_arc, cx);
        notify_control(&self.hue_ring, cx);
    }

    fn sync_controls(&self, cx: &mut Context<Self>) {
        let hue_degrees = self.color.hue_degrees;
        let saturation = self.color.saturation;
        let lightness = self.color.lightness;
        let hsv_value = Hsv::from_hsla_ext(hsla((hue_degrees / 360.0).rem_euclid(1.0), saturation, lightness, 1.0)).v;

        self.sync.sync_color_ring(
            &self.hue_ring,
            &self.hue_ring_renderer,
            Arc::new(HueRingDelegate { saturation, lightness }),
            self.hue_ring_context.clone(),
            hue_degrees,
            cx,
        );
        self.sync.sync_color_arc(
            &self.saturation_arc,
            &self.saturation_renderer,
            Arc::new(RasterArcDelegate::saturation(hue_degrees, hsv_value)),
            self.saturation_context.clone(),
            saturation,
            cx,
        );
        self.sync.sync_color_arc(
            &self.lightness_arc,
            &self.lightness_renderer,
            Arc::new(RasterArcDelegate::lightness(hue_degrees, saturation)),
            self.lightness_context.clone(),
            lightness,
            cx,
        );
    }
}

impl gpui::Render for SplitRingState {
    fn render(&mut self, _window: &mut gpui::Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let swatch_color =
            hsla((self.color.hue_degrees / 360.0).rem_euclid(1.0), self.color.saturation, self.color.lightness, 1.0);
        let ring_size = self.metrics.ring_size();
        let swatch_size = self.metrics.swatch_size();
        let group_width = self.metrics.frame_size();
        let border_size = self.metrics.border_size();
        let canvas_size = self.metrics.canvas_size();
        let arc_offset = self.metrics.arc_horizontal_offset;

        let group_left = (canvas_size - group_width) * 0.5;
        let group_top = (canvas_size - self.metrics.outer_size) * 0.5;
        let border_left = (canvas_size - border_size) * 0.5;
        let border_top = (canvas_size - border_size) * 0.5;
        let ring_offset = (self.metrics.outer_size - ring_size) * 0.5;
        let swatch_offset = (self.metrics.outer_size - swatch_size) * 0.5;

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
                            .size(px(self.metrics.outer_size))
                            .child(self.saturation_arc.clone()),
                    )
                    .child(
                        div()
                            .absolute()
                            .left(px(group_left + arc_offset * 2.0))
                            .top(px(group_top))
                            .size(px(self.metrics.outer_size))
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
                    .w(px(self.metrics.outer_size))
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .child(detail_row("Hex", format_hex_color(swatch_color), &self.look))
                    .child(detail_row("HSLA", format_compact_hsla(swatch_color), &self.look)),
            )
    }
}
