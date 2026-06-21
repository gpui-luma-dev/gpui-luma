use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Subscription, div, prelude::*, px};
use gpui_luma::controls::color::color_field::{ColorFieldEvent, ColorFieldState};
use gpui_luma::controls::color::color_slider::color_spec::Hsv;
use gpui_luma::controls::color::color_slider::{AlphaDelegate, ColorSliderEvent, ColorSliderState, sizing};
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{format_compact_hsla, format_hex_color, gallery_pane_with_description, notify_entity};

const CONTROL_WIDTH: f32 = 260.0;
const SWATCH_HEIGHT: f32 = 44.0;

#[derive(Clone)]
pub(in crate::gallery) struct ColorPickerPane {
    field: Entity<ColorFieldState>,
    hue_slider: Entity<ColorSliderState>,
    alpha_slider: Entity<ColorSliderState>,
    hsv: Hsv,
    suppress_sync: bool,
}

impl ColorPickerPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, _look: Arc<ShadcnLook>) -> Self {
        let hsv = Hsv { h: 12.0, s: 0.78, v: 0.86, a: 0.92 };
        let field = cx.new(|_| {
            ColorFieldState::saturation_value("color-picker-field", hsv, sizing::THUMB_SIZE_MEDIUM)
                .rounded(px(12.0))
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

        Self { field, hue_slider, alpha_slider, hsv, suppress_sync: false }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.field, |app, _, event: &ColorFieldEvent, cx| {
            app.panes.color_picker.handle_field_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.hue_slider, |app, _, event: &ColorSliderEvent, cx| {
            app.panes.color_picker.handle_hue_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.alpha_slider, |app, _, event: &ColorSliderEvent, cx| {
            app.panes.color_picker.handle_alpha_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        let selected = self.hsv.to_hsla_ext();
        let chrome = look.chrome();

        gallery_pane_with_description(
            "Color Picker",
            Some("A composed picker built from the migrated field and slider primitives."),
            div()
                .w_full()
                .max_w(px(920.0))
                .flex()
                .flex_wrap()
                .justify_center()
                .items_start()
                .gap(px(28.0))
                .child(div().size(px(CONTROL_WIDTH)).child(self.field.clone()))
                .child(
                    div()
                        .w(px(CONTROL_WIDTH))
                        .flex()
                        .flex_col()
                        .gap(px(14.0))
                        .child(label_row("Hue"))
                        .child(self.hue_slider.clone())
                        .child(label_row("Alpha"))
                        .child(self.alpha_slider.clone())
                        .child(
                            div()
                                .mt(px(8.0))
                                .flex()
                                .flex_col()
                                .gap(px(10.0))
                                .child(
                                    div()
                                        .h(px(SWATCH_HEIGHT))
                                        .rounded(px(12.0))
                                        .border_1()
                                        .border_color(chrome.border)
                                        .bg(selected),
                                )
                                .child(detail_row("Hex", format_hex_color(selected)))
                                .child(detail_row("HSLA", format_compact_hsla(selected))),
                        ),
                )
                .into_any_element(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.field, cx);
        notify_entity(&self.hue_slider, cx);
        notify_entity(&self.alpha_slider, cx);
    }

    fn handle_field_event(&mut self, event: &ColorFieldEvent, cx: &mut Context<GalleryApp>) {
        if self.suppress_sync {
            return;
        }

        let hsv = match event {
            ColorFieldEvent::Change(hsv) | ColorFieldEvent::Release(hsv) => *hsv,
        };
        self.hsv = hsv;
        self.sync_controls(cx, false);
        cx.notify();
    }

    fn handle_hue_event(&mut self, event: &ColorSliderEvent, cx: &mut Context<GalleryApp>) {
        if self.suppress_sync {
            return;
        }

        let hue = match event {
            ColorSliderEvent::Change(value) | ColorSliderEvent::Release(value) => *value,
        };
        self.hsv.h = hue;
        self.sync_controls(cx, true);
        cx.notify();
    }

    fn handle_alpha_event(&mut self, event: &ColorSliderEvent, cx: &mut Context<GalleryApp>) {
        if self.suppress_sync {
            return;
        }

        let alpha = match event {
            ColorSliderEvent::Change(value) | ColorSliderEvent::Release(value) => *value,
        };
        self.hsv.a = alpha;
        self.sync_controls(cx, true);
        cx.notify();
    }

    fn sync_controls(&mut self, cx: &mut Context<GalleryApp>, sync_field: bool) {
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

fn label_row(label: &'static str) -> gpui::Div {
    div().text_sm().font_weight(gpui::FontWeight::MEDIUM).child(label)
}

fn detail_row(label: &'static str, value: String) -> gpui::Div {
    div()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.0))
        .child(div().text_sm().font_weight(gpui::FontWeight::MEDIUM).child(label))
        .child(div().text_sm().child(value))
}
