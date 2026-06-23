use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, IntoElement, Subscription, div, prelude::*, px};
use gpui_luma::controls::color::color_field::{ColorFieldEvent, ColorFieldState};
use gpui_luma::controls::color::color_slider::color_spec::Hsv;
use gpui_luma::controls::color::color_slider::{
    ChannelDelegate, ColorSliderBuilder, ColorSliderDomainRenderer, primary_slider_value, refresh_color_slider, sizing,
    update_domain_delegate,
};
use gpui_luma::controls::color::ColorSwatch;
use gpui_luma::controls::slider::{SliderControl, SliderEvent};
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::panes::shared::{format_compact_hsla, format_hex_color};

use crate::gallery::panes::color::common::{detail_row, notify_control};

pub(in crate::gallery) struct HsvPlaneState {
    look: Arc<ShadcnLook>,
    hsv: Hsv,
    plane: Entity<ColorFieldState>,
    slider_h: Entity<SliderControl>,
    slider_s: Entity<SliderControl>,
    slider_v: Entity<SliderControl>,
    slider_s_domain: Arc<ColorSliderDomainRenderer>,
    slider_v_domain: Arc<ColorSliderDomainRenderer>,
    _subscriptions: Vec<Subscription>,
}

impl HsvPlaneState {
    pub(in crate::gallery) fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let initial_hsv = Hsv { h: 266.0, s: 0.78, v: 0.76, a: 1.0 };

        let plane = cx.new(|_| {
            ColorFieldState::hue_saturation_value("composition-hsv-plane", initial_hsv, sizing::THUMB_SIZE_MEDIUM)
                .raster_image()
                .rounded(px(0.0))
        });
        let slider_h = ColorSliderBuilder::hue("composition-hsv-plane-h", initial_hsv.h)
            .horizontal()
            .rounded(px(0.0))
            .thumb_small()
            .thumb_square()
            .size(ControlSize::Sm)
            .spawn(cx);
        let slider_s_builder =
            ColorSliderBuilder::channel("composition-hsv-plane-s", initial_hsv.s, initial_hsv, Hsv::SATURATION)
                .expect("HSV saturation delegate should be valid")
                .horizontal()
                .rounded(px(0.0))
                .thumb_small()
                .thumb_square()
                .size(ControlSize::Sm);
        let slider_s_domain = slider_s_builder.domain_renderer();
        let slider_s = slider_s_builder.spawn(cx);
        let slider_v_builder =
            ColorSliderBuilder::channel("composition-hsv-plane-v", initial_hsv.v, initial_hsv, Hsv::VALUE)
                .expect("HSV value delegate should be valid")
                .horizontal()
                .rounded(px(0.0))
                .thumb_small()
                .thumb_square()
                .size(ControlSize::Sm);
        let slider_v_domain = slider_v_builder.domain_renderer();
        let slider_v = slider_v_builder.spawn(cx);

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
            cx.subscribe(&slider_h, |this, _, event: &SliderEvent, cx| {
                if let Some(value) = primary_slider_value(event) {
                    this.hsv.h = value;
                    this.sync_controls(cx);
                    cx.notify();
                }
            }),
            cx.subscribe(&slider_s, |this, _, event: &SliderEvent, cx| {
                if let Some(value) = primary_slider_value(event) {
                    this.hsv.s = value;
                    this.sync_controls(cx);
                    cx.notify();
                }
            }),
            cx.subscribe(&slider_v, |this, _, event: &SliderEvent, cx| {
                if let Some(value) = primary_slider_value(event) {
                    this.hsv.v = value;
                    this.sync_controls(cx);
                    cx.notify();
                }
            }),
        ];

        Self {
            look,
            hsv: initial_hsv,
            plane,
            slider_h,
            slider_s,
            slider_v,
            slider_s_domain,
            slider_v_domain,
            _subscriptions: subscriptions,
        }
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<Self>) {
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
        update_domain_delegate(
            &self.slider_s_domain,
            Arc::new(
                ChannelDelegate::new(hsv, Hsv::SATURATION.into()).expect("HSV saturation delegate should be valid"),
            ),
            self.slider_s_domain.context(),
        );
        update_domain_delegate(
            &self.slider_v_domain,
            Arc::new(ChannelDelegate::new(hsv, Hsv::VALUE.into()).expect("HSV value delegate should be valid")),
            self.slider_v_domain.context(),
        );
        self.slider_h.update(cx, |slider, cx| slider.set_value(hsv.h, cx));
        self.slider_s.update(cx, |slider, cx| slider.set_value(hsv.s, cx));
        self.slider_v.update(cx, |slider, cx| slider.set_value(hsv.v, cx));
        refresh_color_slider(&self.slider_s, cx);
        refresh_color_slider(&self.slider_v, cx);
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
                    .child(ColorSwatch::new(hsla).checkerboard(false).height(px(40.0)).rounded(px(12.0)))
                    .child(detail_row("Hex", format_hex_color(hsla), &self.look))
                    .child(detail_row("HSLA", format_compact_hsla(hsla), &self.look)),
            )
    }
}

fn slider_row(label: &'static str, slider: Entity<SliderControl>) -> AnyElement {
    div()
        .w_full()
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(div().w(px(14.0)).text_xs().child(label))
        .child(div().flex_1().child(slider))
        .into_any_element()
}
