use std::f32::consts::SQRT_2;
use std::sync::Arc;

use gpui::{Context, Entity, IntoElement, Subscription, div, prelude::*, px};
use gpui_luma::controls::color::color_field::{ColorFieldEvent, ColorFieldState};
use gpui_luma::controls::color::color_ring::{ColorRingBuilder, primary_slider_value, sizing};
use gpui_luma::controls::color::color_slider::color_spec::Hsv;
use gpui_luma::controls::color::ColorSwatch;
use gpui_luma::controls::color::style::Size;
use gpui_luma::controls::slider::{SliderControl, SliderEvent};
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::panes::shared::{format_compact_hsla, format_hex_color};

use crate::gallery::panes::color::common::{detail_row, notify_control};

pub(in crate::gallery) struct HsvWheelState {
    look: Arc<ShadcnLook>,
    color_ring: Entity<SliderControl>,
    plane_sv: Entity<ColorFieldState>,
    hsv: Hsv,
    _subscriptions: Vec<Subscription>,
}

impl HsvWheelState {
    const RING_OUTER_SIZE_PX: f32 = 300.0;
    const RING_THICKNESS_PX: f32 = sizing::RING_THICKNESS_MEDIUM;

    pub(in crate::gallery) fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let hsv = Hsv { h: 220.0, s: 0.88, v: 0.6, a: 1.0 };
        let color_ring = ColorRingBuilder::hue("composition-hsv-wheel-ring", hsv.h, 1.0, 0.5)
            .size(Size::Size(px(Self::RING_OUTER_SIZE_PX)))
            .ring_thickness_size(Size::Medium)
            .thumb_size(16.0)
            .spawn(cx);
        let plane_sv = cx.new(|_| {
            ColorFieldState::saturation_value("composition-hsv-wheel-plane", hsv, 16.0)
                // DO NOT USE .raster_image()
                .rounded(px(0.0))
                .no_border()
                .edge_to_edge()
        });

        let subscriptions = vec![
            cx.subscribe(&color_ring, |this, _, event: &SliderEvent, cx| {
                let Some(hue) = primary_slider_value(event) else {
                    return;
                };
                this.hsv.h = hue;
                this.plane_sv.update(cx, |plane, cx| {
                    plane.set_hsv_components(hue, this.hsv.s, this.hsv.v, cx);
                });
                cx.notify();
            }),
            cx.subscribe(&plane_sv, |this, _, event: &ColorFieldEvent, cx| {
                let hsv = match event {
                    ColorFieldEvent::Change(hsv) | ColorFieldEvent::Release(hsv) => *hsv,
                };
                this.hsv.s = hsv.s;
                this.hsv.v = hsv.v;
                cx.notify();
            }),
        ];

        Self { look, color_ring, plane_sv, hsv, _subscriptions: subscriptions }
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<Self>) {
        notify_control(&self.color_ring, cx);
        notify_control(&self.plane_sv, cx);
    }
}

impl gpui::Render for HsvWheelState {
    fn render(&mut self, _window: &mut gpui::Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let ring_outer_size = Self::RING_OUTER_SIZE_PX;
        let ring_inner_diameter = (ring_outer_size - 2.0 * Self::RING_THICKNESS_PX).max(0.0);
        let plane_size = (ring_inner_diameter / SQRT_2 - 8.0).max(40.0);
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
