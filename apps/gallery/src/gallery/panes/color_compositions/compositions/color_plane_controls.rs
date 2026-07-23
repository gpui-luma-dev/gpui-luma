use std::sync::Arc;

use gpui::{Context, Entity, IntoElement, Subscription, div, prelude::*, px};
use gpui_luma::controls::color::composition::ColorCompositionSync;
use gpui_luma::controls::color::color_field::{ColorFieldEvent, ColorFieldState};
use gpui_luma::controls::color::color_slider::color_spec::Hsv;
use gpui_luma::controls::color::color_slider::{
    AlphaDelegate, ColorSliderBuilder, ColorSliderDomainRenderer, primary_slider_value, sizing,
};
use gpui_luma::controls::color::ColorSwatch;
use gpui_luma::controls::slider::SliderControl;
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::ShadcnLook;

use super::CompositionSize;
use crate::gallery::panes::shared::{format_compact_hsla, format_hex_color};
use crate::gallery::panes::color::common::{detail_row, notify_control};

pub(in crate::gallery) struct ColorPickerState {
    look: Arc<ShadcnLook>,
    metrics: ColorPickerMetrics,
    sync: ColorCompositionSync,
    field: Entity<ColorFieldState>,
    hue_slider: Entity<SliderControl>,
    alpha_slider: Entity<SliderControl>,
    alpha_domain: Arc<ColorSliderDomainRenderer>,
    hsv: Hsv,
    _subscriptions: Vec<Subscription>,
}

#[derive(Clone, Copy)]
struct ColorPickerMetrics {
    control_width: f32,
    swatch_height: f32,
}

impl ColorPickerMetrics {
    fn resolve(size: CompositionSize) -> Self {
        let control_width = size.resolve_primary(220.0, 260.0, 320.0);
        let scale = control_width / 260.0;
        Self { control_width, swatch_height: (44.0 * scale).max(32.0) }
    }
}

impl ColorPickerState {
    #[allow(dead_code)]
    pub(in crate::gallery) fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        Self::with_size(look, CompositionSize::Md, cx)
    }

    pub(in crate::gallery) fn with_size(look: Arc<ShadcnLook>, size: CompositionSize, cx: &mut Context<Self>) -> Self {
        let hsv = Hsv { h: 12.0, s: 0.78, v: 0.86, a: 0.92 };
        let metrics = ColorPickerMetrics::resolve(size);
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
                let hsv = match event {
                    ColorFieldEvent::Change(hsv) | ColorFieldEvent::Release(hsv) => *hsv,
                    _ => return,
                };
                let Some(_sync_guard) = this.sync.begin_guard() else {
                    return;
                };
                this.hsv.h = hsv.h;
                this.hsv.s = hsv.s;
                this.hsv.v = hsv.v;
                this.sync_controls(cx, false);
                cx.notify();
            }),
            cx.subscribe(&hue_slider, |this, _, event, cx| {
                let Some(hue) = primary_slider_value(event) else {
                    return;
                };

                let Some(_sync_guard) = this.sync.begin_guard() else {
                    return;
                };
                this.hsv.h = hue;
                this.sync_controls(cx, true);
                cx.notify();
            }),
            cx.subscribe(&alpha_slider, |this, _, event, cx| {
                let Some(alpha) = primary_slider_value(event) else {
                    return;
                };

                let Some(_sync_guard) = this.sync.begin_guard() else {
                    return;
                };
                this.hsv.a = alpha;
                this.sync_controls(cx, true);
                cx.notify();
            }),
        ];

        Self {
            look,
            metrics,
            field,
            hue_slider,
            alpha_slider,
            alpha_domain,
            sync: ColorCompositionSync::new(),
            hsv,
            _subscriptions: subscriptions,
        }
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<Self>) {
        notify_control(&self.field, cx);
        notify_control(&self.hue_slider, cx);
        notify_control(&self.alpha_slider, cx);
    }

    fn sync_controls(&self, cx: &mut Context<Self>, sync_field: bool) {
        let hsv = self.hsv;

        if sync_field {
            self.field.update(cx, |field, cx| {
                field.set_hsv(hsv, cx);
            });
        }

        self.sync.sync_slider_value(&self.hue_slider, hsv.h, cx);
        self.sync.sync_color_slider(
            &self.alpha_slider,
            &self.alpha_domain,
            Arc::new(AlphaDelegate { spec: hsv }),
            self.alpha_domain.context(),
            hsv.a,
            cx,
        );
    }
}

impl gpui::Render for ColorPickerState {
    fn render(&mut self, _window: &mut gpui::Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let selected = self.hsv.to_hsla_ext();
        let control_width = self.metrics.control_width;
        let swatch_height = self.metrics.swatch_height;

        div()
            .w_full()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(14.0))
            .child(div().size(px(control_width)).child(self.field.clone()))
            .child(div().w(px(control_width)).child(self.hue_slider.clone()))
            .child(div().w(px(control_width)).child(self.alpha_slider.clone()))
            .child(
                div()
                    .w(px(control_width))
                    .flex()
                    .flex_col()
                    .gap(px(10.0))
                    .child(ColorSwatch::new(selected).checkerboard(true).height(px(swatch_height)).rounded(px(12.0)))
                    .child(detail_row("Hex", format_hex_color(selected), &self.look))
                    .child(detail_row("HSLA", format_compact_hsla(selected), &self.look)),
            )
    }
}
