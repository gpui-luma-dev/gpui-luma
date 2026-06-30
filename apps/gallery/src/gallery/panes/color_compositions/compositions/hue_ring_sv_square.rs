use std::f32::consts::SQRT_2;
use std::sync::Arc;

use gpui::{Context, Entity, IntoElement, Subscription, div, prelude::*, px};
use gpui_luma::controls::color::composition::ColorCompositionSync;
use gpui_luma::controls::color::color_field::{ColorFieldEvent, ColorFieldState};
use gpui_luma::controls::color::color_ring::{ColorRingBuilder, primary_slider_value, sizing};
use gpui_luma::controls::color::color_slider::color_spec::Hsv;
use gpui_luma::controls::color::ColorSwatch;
use gpui_luma::controls::color::style::Size;
use gpui_luma::controls::slider::SliderControl;
use gpui_luma_look_shadcn::ShadcnLook;

use super::CompositionSize;
use crate::gallery::panes::shared::{format_compact_hsla, format_hex_color};

use crate::gallery::panes::color::common::{detail_row, notify_control};

pub(in crate::gallery) struct HsvWheelState {
    look: Arc<ShadcnLook>,
    metrics: HsvWheelMetrics,
    sync: ColorCompositionSync,
    color_ring: Entity<SliderControl>,
    plane_sv: Entity<ColorFieldState>,
    hsv: Hsv,
    _subscriptions: Vec<Subscription>,
}

#[derive(Clone, Copy)]
struct HsvWheelMetrics {
    ring_outer_size: f32,
    ring_thickness: f32,
    ring_thumb_size: f32,
    inner_gap: f32,
}

impl HsvWheelMetrics {
    fn resolve(size: CompositionSize) -> Self {
        let ring_outer_size = size.resolve_primary(220.0, 300.0, 380.0);
        let scale = ring_outer_size / 300.0;
        Self {
            ring_outer_size,
            ring_thickness: sizing::RING_THICKNESS_MEDIUM * scale,
            ring_thumb_size: (16.0 * scale).max(12.0),
            inner_gap: (8.0 * scale).max(6.0),
        }
    }

    fn plane_size(self) -> f32 {
        let ring_inner_diameter = (self.ring_outer_size - 2.0 * self.ring_thickness).max(0.0);
        (ring_inner_diameter / SQRT_2 - self.inner_gap).max(40.0)
    }
}

impl HsvWheelState {
    #[allow(dead_code)]
    pub(in crate::gallery) fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        Self::with_size(look, CompositionSize::Md, cx)
    }

    pub(in crate::gallery) fn with_size(look: Arc<ShadcnLook>, size: CompositionSize, cx: &mut Context<Self>) -> Self {
        let hsv = Hsv { h: 220.0, s: 0.88, v: 0.6, a: 1.0 };
        let metrics = HsvWheelMetrics::resolve(size);
        let color_ring = ColorRingBuilder::hue("composition-hsv-wheel-ring", hsv.h, 1.0, 0.5)
            .size(Size::Size(px(metrics.ring_outer_size)))
            .ring_thickness(metrics.ring_thickness)
            .thumb_size(metrics.ring_thumb_size)
            .spawn(cx);
        let plane_sv = cx.new(|_| {
            ColorFieldState::saturation_value("composition-hsv-wheel-plane", hsv, metrics.ring_thumb_size)
                // DO NOT USE .raster_image()
                .rounded(px(0.0))
                .no_border()
                .edge_to_edge()
        });

        let subscriptions = vec![
            cx.subscribe(&color_ring, |this, _, event, cx| {
                let Some(hue) = primary_slider_value(event) else {
                    return;
                };
                if !this.sync.begin_sync() {
                    return;
                }
                this.hsv.h = hue;
                this.sync_controls(cx);
                this.sync.end_sync();
                cx.notify();
            }),
            cx.subscribe(&plane_sv, |this, _, event: &ColorFieldEvent, cx| {
                if !this.sync.begin_sync() {
                    return;
                }
                let hsv = match event {
                    ColorFieldEvent::Change(hsv) | ColorFieldEvent::Release(hsv) => *hsv,
                };
                this.hsv.s = hsv.s;
                this.hsv.v = hsv.v;
                this.sync_controls(cx);
                this.sync.end_sync();
                cx.notify();
            }),
        ];

        Self {
            look,
            metrics,
            sync: ColorCompositionSync::new(),
            color_ring,
            plane_sv,
            hsv,
            _subscriptions: subscriptions,
        }
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<Self>) {
        notify_control(&self.color_ring, cx);
        notify_control(&self.plane_sv, cx);
    }

    fn sync_controls(&self, cx: &mut Context<Self>) {
        let hsv = self.hsv;
        self.sync.sync_slider_value(&self.color_ring, hsv.h, cx);
        self.plane_sv.update(cx, |plane, cx| {
            plane.set_hsv_components(hsv.h, hsv.s, hsv.v, cx);
        });
    }
}

impl gpui::Render for HsvWheelState {
    fn render(&mut self, _window: &mut gpui::Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let ring_outer_size = self.metrics.ring_outer_size;
        let plane_size = self.metrics.plane_size();
        let plane_half = plane_size / 2.0;

        let color = self.hsv.to_hsla_ext();

        div()
            .w_full()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(14.0))
            .child(
                div()
                    .size(px(ring_outer_size))
                    .relative()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(self.color_ring.clone())
                    .child(
                        div()
                            .absolute()
                            .top_1_2()
                            .left_1_2()
                            .mt(px(-plane_half))
                            .ml(px(-plane_half))
                            .size(px(plane_size))
                            .child(self.plane_sv.clone()),
                    ),
            )
            .child(
                div()
                    .w(px(ring_outer_size))
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .child(ColorSwatch::new(color).checkerboard(false).height(px(40.0)).rounded(px(12.0)))
                    .child(detail_row("Hex", format_hex_color(color), &self.look))
                    .child(detail_row("HSLA", format_compact_hsla(color), &self.look)),
            )
    }
}
