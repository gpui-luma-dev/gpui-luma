use std::f32::consts::SQRT_2;
use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, IntoElement, Subscription, div, prelude::*, px};
use gpui_luma::controls::color::color_field::{ColorFieldEvent, ColorFieldState};
use gpui_luma::controls::color::color_ring::{ColorRingEvent, ColorRingState, HueRingDelegate};
use gpui_luma::controls::color::color_slider::color_spec::Hsv;
use gpui_luma::controls::color::ColorSwatch;
use gpui_luma::controls::color::style::Size;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{format_compact_hsla, format_hex_color, notify_entity};

use super::common::{color_gallery_pane, detail_row, notify_control};

#[derive(Clone)]
pub(in crate::gallery) struct HsvWheelPane {
    state: Entity<HsvWheelState>,
}

impl HsvWheelPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let state = cx.new(|cx| HsvWheelState::new(look, cx));
        Self { state }
    }

    pub(in crate::gallery) fn subscribe(&self, _cx: &mut Context<GalleryApp>, _subscriptions: &mut Vec<Subscription>) {}

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        color_gallery_pane(
            "HSV Wheel",
            "Hue ring with an embedded saturation/value square, mirroring the upstream wheel composition page.",
            self.state.clone(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        self.state.update(cx, |state, cx| state.notify_controls(cx));
        notify_entity(&self.state, cx);
    }
}

struct HsvWheelState {
    look: Arc<ShadcnLook>,
    color_ring: Entity<ColorRingState>,
    plane_sv: Entity<ColorFieldState>,
    hsv: Hsv,
    _subscriptions: Vec<Subscription>,
}

impl HsvWheelState {
    const RING_OUTER_SIZE_PX: f32 = 300.0;

    fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let hsv = Hsv { h: 220.0, s: 0.88, v: 0.6, a: 1.0 };
        let color_ring = cx.new(|cx| {
            ColorRingState::hue(
                "composition-hsv-wheel-ring",
                hsv.h,
                HueRingDelegate { saturation: 1.0, lightness: 0.5 },
                cx,
            )
            .size(Size::Size(px(Self::RING_OUTER_SIZE_PX)))
            .ring_thickness_size(Size::Medium)
            .thumb_size(16.0)
        });
        let plane_sv = cx.new(|_| {
            ColorFieldState::saturation_value("composition-hsv-wheel-plane", hsv, 16.0)
                .raster_image()
                .rounded(px(0.0))
                .no_border()
                .edge_to_edge()
        });

        let subscriptions = vec![
            cx.subscribe(&color_ring, |this, _, event: &ColorRingEvent, cx| {
                let hue = match event {
                    ColorRingEvent::Change(value) | ColorRingEvent::Release(value) => *value,
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

    fn notify_controls(&self, cx: &mut Context<Self>) {
        notify_control(&self.color_ring, cx);
        notify_control(&self.plane_sv, cx);
    }
}

impl gpui::Render for HsvWheelState {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (ring_outer_size, plane_half, plane_size) = {
            let ring_state = self.color_ring.read(cx);
            let ring_outer_size = match ring_state.size {
                Size::XSmall => 140.0,
                Size::Small => 180.0,
                Size::Medium => 220.0,
                Size::Large => 280.0,
                Size::Size(px) => px.as_f32(),
            };
            let ring_inner_diameter = (ring_outer_size - 2.0 * ring_state.ring_thickness_px()).max(0.0);
            let plane_size = (ring_inner_diameter / SQRT_2 - 8.0).max(40.0);
            let plane_half = plane_size / 2.0;
            (ring_outer_size, plane_half, plane_size)
        };

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
                    .child(ColorSwatch::new(color).height(px(40.0)).rounded(px(12.0)).disable_checkerboard())
                    .child(detail_row("Hex", format_hex_color(color), &self.look))
                    .child(detail_row("HSLA", format_compact_hsla(color), &self.look)),
            )
    }
}
