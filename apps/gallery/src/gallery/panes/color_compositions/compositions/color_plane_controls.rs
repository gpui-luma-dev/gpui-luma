use std::sync::Arc;

use gpui::{Context, Entity, IntoElement, Subscription, div, prelude::*, px};
use gpui_luma::controls::color::color_field::{ColorFieldEvent, ColorFieldState};
use gpui_luma::controls::color::color_slider::color_spec::Hsv;
use gpui_luma::controls::color::color_slider::{
    AlphaDelegate, ColorSliderBuilder, ColorSliderDomainRenderer, primary_slider_value, refresh_color_slider, sizing,
    update_domain_delegate,
};
use gpui_luma::controls::color::ColorSwatch;
use gpui_luma::controls::slider::{SliderControl, SliderEvent};
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::panes::shared::{format_compact_hsla, format_hex_color};
use crate::gallery::panes::color::common::{detail_row, notify_control};

const CONTROL_WIDTH: f32 = 260.0;
const SWATCH_HEIGHT: f32 = 44.0;

pub(in crate::gallery) struct ColorPickerState {
    look: Arc<ShadcnLook>,
    field: Entity<ColorFieldState>,
    hue_slider: Entity<SliderControl>,
    alpha_slider: Entity<SliderControl>,
    alpha_domain: Arc<ColorSliderDomainRenderer>,
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
        let hue_slider = ColorSliderBuilder::hue("color-picker-hue-slider", hsv.h)
            .size(ControlSize::Sm)
            .thumb_medium()
            .edge_to_edge()
            .spawn(cx);
        let alpha_builder = ColorSliderBuilder::alpha("color-picker-alpha-slider", hsv.a, hsv)
            .size(ControlSize::Sm)
            .thumb_medium()
            .edge_to_edge();
        let alpha_domain = alpha_builder.domain_renderer();
        let alpha_slider = alpha_builder.spawn(cx);

        let subscriptions = vec![
            cx.subscribe(&field, |this, _, event: &ColorFieldEvent, cx| {
                if this.suppress_sync {
                    return;
                }

                let hsv = match event {
                    ColorFieldEvent::Change(hsv) | ColorFieldEvent::Release(hsv) => *hsv,
                };
                this.hsv.h = hsv.h;
                this.hsv.s = hsv.s;
                this.hsv.v = hsv.v;
                this.sync_controls(cx, false);
                cx.notify();
            }),
            cx.subscribe(&hue_slider, |this, _, event: &SliderEvent, cx| {
                if this.suppress_sync {
                    return;
                }

                if let Some(hue) = primary_slider_value(event) {
                    this.hsv.h = hue;
                    this.sync_controls(cx, true);
                    cx.notify();
                }
            }),
            cx.subscribe(&alpha_slider, |this, _, event: &SliderEvent, cx| {
                if this.suppress_sync {
                    return;
                }

                if let Some(alpha) = primary_slider_value(event) {
                    this.hsv.a = alpha;
                    this.sync_controls(cx, true);
                    cx.notify();
                }
            }),
        ];

        Self {
            look,
            field,
            hue_slider,
            alpha_slider,
            alpha_domain,
            hsv,
            suppress_sync: false,
            _subscriptions: subscriptions,
        }
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
                field.set_hsv(hsv, cx);
            });
        }

        update_domain_delegate(&self.alpha_domain, Arc::new(AlphaDelegate { spec: hsv }), self.alpha_domain.context());
        self.hue_slider.update(cx, |slider, cx| {
            slider.set_value(hsv.h, cx);
        });
        self.alpha_slider.update(cx, |slider, cx| {
            slider.set_value(hsv.a, cx);
        });
        refresh_color_slider(&self.alpha_slider, cx);

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
