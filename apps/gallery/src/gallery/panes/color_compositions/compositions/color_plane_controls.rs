use std::sync::Arc;

use gpui::{Context, Entity, IntoElement, Subscription, div, prelude::*, px};
use gpui_luma::controls::color::color_field::{ColorFieldEvent, ColorFieldState};
use gpui_luma::controls::color::color_slider::color_spec::Hsv;
use gpui_luma::controls::color::color_slider::{AlphaDelegate, ColorSliderEvent, ColorSliderState, sizing};
use gpui_luma::controls::color::ColorSwatch;
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::panes::shared::{format_compact_hsla, format_hex_color};
use crate::gallery::panes::color::common::{detail_row, notify_control};

const CONTROL_WIDTH: f32 = 260.0;
const SWATCH_HEIGHT: f32 = 44.0;

pub(in crate::gallery) struct ColorPickerState {
    look: Arc<ShadcnLook>,
    field: Entity<ColorFieldState>,
    hue_slider: Entity<ColorSliderState>,
    alpha_slider: Entity<ColorSliderState>,
    hsv: Hsv,
    suppress_sync: bool,
    _subscriptions: Vec<Subscription>,
}

impl ColorPickerState {
    pub(in crate::gallery) fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let hsv = Hsv { h: 12.0, s: 0.78, v: 0.86, a: 0.92 };
        let field = cx.new(|_| {
            ColorFieldState::saturation_value("color-picker-field", hsv, sizing::THUMB_SIZE_MEDIUM)
                .rounded(px(3.0))
                .vector()
        });
        let hue_slider = cx.new(|cx| {
            ColorSliderState::hue("color-picker-hue-slider", hsv.h, cx)
                .size(ControlSize::Sm)
                .thumb_medium()
                .edge_to_edge()
        });
        let alpha_slider = cx.new(|cx| {
            ColorSliderState::alpha("color-picker-alpha-slider", hsv.a, AlphaDelegate { spec: hsv }, cx)
                .size(ControlSize::Sm)
                .thumb_medium()
                .edge_to_edge()
        });

        let subscriptions = vec![
            cx.subscribe(&field, |this, _, event: &ColorFieldEvent, cx| {
                if this.suppress_sync {
                    return;
                }

                let hsv = match event {
                    ColorFieldEvent::Change(hsv) | ColorFieldEvent::Release(hsv) => *hsv,
                };
                this.hsv = hsv;
                this.sync_controls(cx, false);
                cx.notify();
            }),
            cx.subscribe(&hue_slider, |this, _, event: &ColorSliderEvent, cx| {
                if this.suppress_sync {
                    return;
                }

                let hue = match event {
                    ColorSliderEvent::Change(value) | ColorSliderEvent::Release(value) => *value,
                };
                this.hsv.h = hue;
                this.sync_controls(cx, true);
                cx.notify();
            }),
            cx.subscribe(&alpha_slider, |this, _, event: &ColorSliderEvent, cx| {
                if this.suppress_sync {
                    return;
                }

                let alpha = match event {
                    ColorSliderEvent::Change(value) | ColorSliderEvent::Release(value) => *value,
                };
                this.hsv.a = alpha;
                this.sync_controls(cx, true);
                cx.notify();
            }),
        ];

        Self { look, field, hue_slider, alpha_slider, hsv, suppress_sync: false, _subscriptions: subscriptions }
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<Self>) {
        notify_control(&self.field, cx);
        notify_control(&self.hue_slider, cx);
        notify_control(&self.alpha_slider, cx);
    }

    fn sync_controls(&mut self, cx: &mut Context<Self>, sync_field: bool) {
        self.suppress_sync = true;
        let hsv = self.hsv;

        if sync_field {
            self.field.update(cx, |field, cx| {
                field.set_hsv_components(hsv.h, hsv.s, hsv.v, cx);
            });
        }

        self.hue_slider.update(cx, |slider, cx| {
            slider.set_value(hsv.h, cx);
        });
        self.alpha_slider.update(cx, |slider, cx| {
            slider.set_value(hsv.a, cx);
            slider.set_delegate(Box::new(AlphaDelegate { spec: hsv }), cx);
        });

        self.suppress_sync = false;
    }
}

impl gpui::Render for ColorPickerState {
    fn render(&mut self, _window: &mut gpui::Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let selected = self.hsv.to_hsla_ext();

        div()
            .w_full()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(14.0))
            .child(div().size(px(CONTROL_WIDTH)).child(self.field.clone()))
            .child(div().w(px(CONTROL_WIDTH)).child(self.hue_slider.clone()))
            .child(div().w(px(CONTROL_WIDTH)).child(self.alpha_slider.clone()))
            .child(
                div()
                    .w(px(CONTROL_WIDTH))
                    .flex()
                    .flex_col()
                    .gap(px(10.0))
                    .child(ColorSwatch::new(selected).checkerboard(true).height(px(SWATCH_HEIGHT)).rounded(px(12.0)))
                    .child(detail_row("Hex", format_hex_color(selected), &self.look))
                    .child(detail_row("HSLA", format_compact_hsla(selected), &self.look)),
            )
    }
}
