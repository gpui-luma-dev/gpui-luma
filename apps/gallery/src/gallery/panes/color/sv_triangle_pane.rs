use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Hsla, IntoElement, Subscription, div, prelude::*, px};
use gpui_luma::controls::color::color_field::{ColorFieldEvent, ColorFieldModel2D, ColorFieldState, TriangleDomain};
use gpui_luma::controls::color::color_ring::{ColorRingEvent, ColorRingState, HueRingDelegate};
use gpui_luma::controls::color::color_slider::color_spec::Hsv;
use gpui_luma::controls::color::style::Size;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{format_compact_hsla, format_hex_color, notify_entity};

use super::common::{color_gallery_pane, detail_row, notify_control};

#[derive(Clone)]
pub(in crate::gallery) struct SvTrianglePane {
    state: Entity<SvTriangleState>,
}

impl SvTrianglePane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let state = cx.new(|cx| SvTriangleState::new(look, cx));
        Self { state }
    }

    pub(in crate::gallery) fn subscribe(&self, _cx: &mut Context<GalleryApp>, _subscriptions: &mut Vec<Subscription>) {}

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        color_gallery_pane(
            "SV Triangle",
            "Hue ring with an embedded Photoshop-style saturation/value triangle.",
            self.state.clone(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        self.state.update(cx, |state, cx| state.notify_controls(cx));
        notify_entity(&self.state, cx);
    }
}

struct SvTriangleState {
    look: Arc<ShadcnLook>,
    hsv: Hsv,
    ring: Entity<ColorRingState>,
    triangle: Entity<ColorFieldState>,
    _subscriptions: Vec<Subscription>,
}

impl SvTriangleState {
    fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let initial_hsv = Hsv { h: 317.0, s: 0.83, v: 0.84, a: 1.0 };

        let ring = cx.new(|cx| {
            ColorRingState::hue(
                "composition-sv-triangle-ring",
                initial_hsv.h,
                HueRingDelegate { saturation: 1.0, lightness: 0.5 },
                cx,
            )
            .size(Size::Size(px(300.0)))
            .ring_thickness_size(Size::Medium)
            .thumb_size(14.0)
        });

        let triangle = cx.new(|_| {
            let (white, black, hue) = sv_triangle_vertices();
            ColorFieldState::new(
                "composition-sv-triangle-field",
                initial_hsv,
                Arc::new(TriangleDomain { a: white, b: black, c: hue }),
                Arc::new(PhotoshopSvTriangleModel),
            )
            .thumb_size(14.0)
            .raster_image()
            .rounded(px(0.0))
            .no_border()
            .edge_to_edge()
        });

        let subscriptions = vec![
            cx.subscribe(&ring, |this, _, event: &ColorRingEvent, cx| {
                let hue = match event {
                    ColorRingEvent::Change(value) | ColorRingEvent::Release(value) => *value,
                };
                this.hsv.h = hue;
                this.ring.update(cx, |ring, cx| {
                    ring.set_value(hue, cx);
                });
                this.triangle.update(cx, |field, cx| {
                    field.set_hsv_components(this.hsv.h, this.hsv.s, this.hsv.v, cx);
                });
                cx.notify();
            }),
            cx.subscribe(&triangle, |this, _, event: &ColorFieldEvent, cx| {
                let hsv = match event {
                    ColorFieldEvent::Change(hsv) | ColorFieldEvent::Release(hsv) => *hsv,
                };
                this.hsv.s = hsv.s;
                this.hsv.v = hsv.v;
                cx.notify();
            }),
        ];

        Self { look, hsv: initial_hsv, ring, triangle, _subscriptions: subscriptions }
    }

    fn notify_controls(&self, cx: &mut Context<Self>) {
        notify_control(&self.ring, cx);
        notify_control(&self.triangle, cx);
    }
}

impl gpui::Render for SvTriangleState {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (ring_outer_size, triangle_size, triangle_half) = {
            let ring_state = self.ring.read(cx);
            let outer = match ring_state.size {
                Size::XSmall => 140.0,
                Size::Small => 180.0,
                Size::Medium => 220.0,
                Size::Large => 280.0,
                Size::Size(px) => px.as_f32(),
            };
            let inner = (outer - 2.0 * ring_state.ring_thickness_px()).max(0.0);
            let tri = (inner - 2.0).max(40.0);
            (outer, tri, tri * 0.5)
        };

        let swatch = self.hsv.to_hsla_ext();

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
                    .child(self.ring.clone())
                    .child(
                        div()
                            .absolute()
                            .left_1_2()
                            .top_1_2()
                            .ml(px(-triangle_half))
                            .mt(px(-triangle_half))
                            .size(px(triangle_size))
                            .child(self.triangle.clone()),
                    ),
            )
            .child(
                div()
                    .w(px(ring_outer_size))
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .child(
                        div()
                            .h(px(40.0))
                            .rounded(px(12.0))
                            .border_1()
                            .border_color(self.look.chrome().border)
                            .bg(swatch),
                    )
                    .child(detail_row("Hex", format_hex_color(swatch), &self.look))
                    .child(detail_row("HSLA", format_compact_hsla(swatch), &self.look)),
            )
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct PhotoshopSvTriangleModel;

impl ColorFieldModel2D for PhotoshopSvTriangleModel {
    fn apply_uv(&self, hsv: &mut Hsv, uv: (f32, f32)) {
        let x = uv.0.clamp(0.0, 1.0);
        let y = uv.1.clamp(0.0, 1.0);
        let (white, black, hue) = sv_triangle_vertices();
        let (_, black_weight, hue_weight) = barycentric((x, y), white, black, hue);
        let value = (1.0 - black_weight.clamp(0.0, 1.0)).clamp(0.0, 1.0);
        let saturation = if value <= f32::EPSILON {
            0.0
        } else {
            (hue_weight.clamp(0.0, 1.0) / value).clamp(0.0, 1.0)
        };

        hsv.s = saturation;
        hsv.v = value;
    }

    fn uv_from_hsv(&self, hsv: &Hsv) -> (f32, f32) {
        let saturation = hsv.s.clamp(0.0, 1.0);
        let value = hsv.v.clamp(0.0, 1.0);
        let hue_weight = (saturation * value).clamp(0.0, 1.0);
        let black_weight = (1.0 - value).clamp(0.0, 1.0);
        let white_weight = (value - hue_weight).clamp(0.0, 1.0);
        let (white, black, hue) = sv_triangle_vertices();

        (
            (white.0 * white_weight + black.0 * black_weight + hue.0 * hue_weight).clamp(0.0, 1.0),
            (white.1 * white_weight + black.1 * black_weight + hue.1 * hue_weight).clamp(0.0, 1.0),
        )
    }

    fn color_at_uv(&self, hsv: &Hsv, uv: (f32, f32)) -> Hsla {
        let x = uv.0.clamp(0.0, 1.0);
        let y = uv.1.clamp(0.0, 1.0);
        let (white, black, hue) = sv_triangle_vertices();
        let (_, black_weight, hue_weight) = barycentric((x, y), white, black, hue);
        let value = (1.0 - black_weight.clamp(0.0, 1.0)).clamp(0.0, 1.0);
        let saturation = if value <= f32::EPSILON {
            0.0
        } else {
            (hue_weight.clamp(0.0, 1.0) / value).clamp(0.0, 1.0)
        };

        Hsv { h: hsv.h, s: saturation, v: value, a: 1.0 }.to_hsla_ext()
    }
}

fn sv_triangle_vertices() -> ((f32, f32), (f32, f32), (f32, f32)) {
    ((0.25, 0.066_987_3), (0.25, 0.933_012_7), (1.0, 0.5))
}

fn barycentric(point: (f32, f32), a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> (f32, f32, f32) {
    let denom = (b.1 - c.1) * (a.0 - c.0) + (c.0 - b.0) * (a.1 - c.1);
    if denom.abs() <= f32::EPSILON {
        return (0.0, 0.0, 0.0);
    }

    let wa = ((b.1 - c.1) * (point.0 - c.0) + (c.0 - b.0) * (point.1 - c.1)) / denom;
    let wb = ((c.1 - a.1) * (point.0 - c.0) + (a.0 - c.0) * (point.1 - c.1)) / denom;
    let wc = 1.0 - wa - wb;
    (wa, wb, wc)
}
