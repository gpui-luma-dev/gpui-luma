use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, IntoElement, Subscription, div, prelude::*, px};
use gpui_luma::controls::color::color_field::{ColorFieldEvent, ColorFieldState};
use gpui_luma::controls::color::color_slider::color_spec::Hsv;
use gpui_luma::controls::color::color_slider::{ChannelDelegate, ColorSliderEvent, ColorSliderState, sizing};
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{format_compact_hsla, format_hex_color, notify_entity};

use super::common::{color_gallery_pane, detail_row, notify_control};

#[derive(Clone)]
pub(in crate::gallery) struct HsvPlanePane {
    state: Entity<HsvPlaneState>,
}

impl HsvPlanePane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let state = cx.new(|cx| HsvPlaneState::new(look, cx));
        Self { state }
    }

    pub(in crate::gallery) fn subscribe(&self, _cx: &mut Context<GalleryApp>, _subscriptions: &mut Vec<Subscription>) {}

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        color_gallery_pane(
            "HSV Plane",
            "The Photoshop-style HSV plane composition with dedicated H, S, and V sliders.",
            self.state.clone(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        self.state.update(cx, |state, cx| state.notify_controls(cx));
        notify_entity(&self.state, cx);
    }
}

struct HsvPlaneState {
    look: Arc<ShadcnLook>,
    hsv: Hsv,
    plane: Entity<ColorFieldState>,
    slider_h: Entity<ColorSliderState>,
    slider_s: Entity<ColorSliderState>,
    slider_v: Entity<ColorSliderState>,
    _subscriptions: Vec<Subscription>,
}

impl HsvPlaneState {
    fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let initial_hsv = Hsv { h: 266.0, s: 0.78, v: 0.76, a: 1.0 };

        let plane = cx.new(|_| {
            ColorFieldState::hue_saturation_value("composition-hsv-plane", initial_hsv, sizing::THUMB_SIZE_MEDIUM)
                .raster_image()
                .rounded(px(0.0))
        });
        let slider_h = cx.new(|cx| {
            let mut slider = ColorSliderState::hue("composition-hsv-plane-h", initial_hsv.h, cx)
                .horizontal()
                .rounded(px(0.0))
                .thumb_small()
                .thumb_square();
            slider.set_size(ControlSize::Sm, cx);
            slider
        });
        let slider_s = cx.new(|cx| {
            let mut slider = ColorSliderState::channel(
                "composition-hsv-plane-s",
                initial_hsv.s,
                ChannelDelegate::new(initial_hsv, Hsv::SATURATION.into())
                    .expect("HSV saturation delegate should be valid"),
                cx,
            )
            .horizontal()
            .rounded(px(0.0))
            .thumb_small()
            .thumb_square();
            slider.set_size(ControlSize::Sm, cx);
            slider
        });
        let slider_v = cx.new(|cx| {
            let mut slider = ColorSliderState::channel(
                "composition-hsv-plane-v",
                initial_hsv.v,
                ChannelDelegate::new(initial_hsv, Hsv::VALUE.into()).expect("HSV value delegate should be valid"),
                cx,
            )
            .horizontal()
            .rounded(px(0.0))
            .thumb_small()
            .thumb_square();
            slider.set_size(ControlSize::Sm, cx);
            slider
        });

        let subscriptions = vec![
            cx.subscribe(&plane, |this, _, event: &ColorFieldEvent, cx| {
                let hsv = match event {
                    ColorFieldEvent::Change(hsv) | ColorFieldEvent::Release(hsv) => *hsv,
                };
                this.hsv.h = hsv.h;
                this.hsv.s = hsv.s;
                this.hsv.v = hsv.v;
                this.sync_controls(cx);
                cx.notify();
            }),
            cx.subscribe(&slider_h, |this, _, event: &ColorSliderEvent, cx| {
                let value = match event {
                    ColorSliderEvent::Change(value) | ColorSliderEvent::Release(value) => *value,
                };
                this.hsv.h = value;
                this.sync_controls(cx);
                cx.notify();
            }),
            cx.subscribe(&slider_s, |this, _, event: &ColorSliderEvent, cx| {
                let value = match event {
                    ColorSliderEvent::Change(value) | ColorSliderEvent::Release(value) => *value,
                };
                this.hsv.s = value;
                this.sync_controls(cx);
                cx.notify();
            }),
            cx.subscribe(&slider_v, |this, _, event: &ColorSliderEvent, cx| {
                let value = match event {
                    ColorSliderEvent::Change(value) | ColorSliderEvent::Release(value) => *value,
                };
                this.hsv.v = value;
                this.sync_controls(cx);
                cx.notify();
            }),
        ];

        Self { look, hsv: initial_hsv, plane, slider_h, slider_s, slider_v, _subscriptions: subscriptions }
    }

    fn notify_controls(&self, cx: &mut Context<Self>) {
        notify_control(&self.plane, cx);
        notify_control(&self.slider_h, cx);
        notify_control(&self.slider_s, cx);
        notify_control(&self.slider_v, cx);
    }

    fn sync_controls(&self, cx: &mut Context<Self>) {
        let hsv = self.hsv;
        self.plane.update(cx, |plane, cx| {
            plane.set_hsv_components(hsv.h, hsv.s, hsv.v, cx);
        });
        self.slider_h.update(cx, |slider, cx| slider.set_value(hsv.h, cx));
        self.slider_s.update(cx, |slider, cx| {
            slider.set_value(hsv.s, cx);
            slider.set_delegate(
                Box::new(
                    ChannelDelegate::new(hsv, Hsv::SATURATION.into()).expect("HSV saturation delegate should be valid"),
                ),
                cx,
            );
        });
        self.slider_v.update(cx, |slider, cx| {
            slider.set_value(hsv.v, cx);
            slider.set_delegate(
                Box::new(ChannelDelegate::new(hsv, Hsv::VALUE.into()).expect("HSV value delegate should be valid")),
                cx,
            );
        });
    }
}

impl gpui::Render for HsvPlaneState {
    fn render(&mut self, _window: &mut gpui::Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let hsla = self.hsv.to_hsla_ext();
        let plane_size = 280.0;

        div()
            .w(px(plane_size))
            .max_w_full()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .child(div().w(px(plane_size)).h(px(plane_size)).overflow_hidden().child(self.plane.clone()))
            .child(slider_row("H", self.slider_h.clone()))
            .child(slider_row("S", self.slider_s.clone()))
            .child(slider_row("V", self.slider_v.clone()))
            .child(
                div()
                    .w(px(plane_size))
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .w(px(plane_size))
                            .h(px(40.0))
                            .rounded(px(12.0))
                            .bg(hsla)
                            .border_1()
                            .border_color(self.look.chrome().border),
                    )
                    .child(detail_row("Hex", format_hex_color(hsla), &self.look))
                    .child(detail_row("HSLA", format_compact_hsla(hsla), &self.look)),
            )
    }
}

fn slider_row(label: &'static str, slider: Entity<ColorSliderState>) -> AnyElement {
    div()
        .w_full()
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(div().w(px(14.0)).text_xs().child(label))
        .child(div().flex_1().child(slider))
        .into_any_element()
}
