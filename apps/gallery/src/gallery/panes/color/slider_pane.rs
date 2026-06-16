use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, SharedString, Subscription, div, prelude::*, px};
use gpui_luma::controls::color::color_slider::color_spec::{Hsl, RgbaSpec};
use gpui_luma::controls::color::color_slider::{
    AlphaDelegate, ChannelDelegate, ColorInterpolation, ColorSliderEvent, ColorSliderState, ColorSpecification,
};
use gpui_luma::controls::color::style::Size;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{format_compact_hsla, format_hex_color, notify_entity};

use super::common::{color_gallery_pane, control_label, demo_card, demo_section, detail_row};

const WIDE_CARD: f32 = 420.0;
const NARROW_CARD: f32 = 320.0;

#[derive(Clone)]
pub(in crate::gallery) struct ColorSliderPane {
    hue_slider: Entity<ColorSliderState>,
    saturation_slider: Entity<ColorSliderState>,
    alpha_slider: Entity<ColorSliderState>,
    gradient_rgb: Entity<ColorSliderState>,
    gradient_hsl: Entity<ColorSliderState>,
    gradient_lab: Entity<ColorSliderState>,
    red_slider: Entity<ColorSliderState>,
    green_slider: Entity<ColorSliderState>,
    blue_slider: Entity<ColorSliderState>,
    hsl: Hsl,
    last_event: SharedString,
}

impl ColorSliderPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, _look: Arc<ShadcnLook>) -> Self {
        let hsl = Hsl { h: 210.0, s: 0.72, l: 0.52, a: 0.85 };

        let hue_slider =
            cx.new(|cx| ColorSliderState::hue("color-slider-hue", hsl.h, cx).size(Size::Small).thumb_medium());
        let saturation_slider = cx.new(|cx| {
            ColorSliderState::channel(
                "color-slider-saturation",
                hsl.s,
                ChannelDelegate::new(hsl, Hsl::SATURATION.into()).expect("HSL saturation delegate should be valid"),
                cx,
            )
            .size(Size::Small)
            .thumb_medium()
            .edge_to_edge()
        });
        let alpha_slider = cx.new(|cx| {
            ColorSliderState::alpha("color-slider-alpha", hsl.a, AlphaDelegate { spec: hsl }, cx)
                .size(Size::Small)
                .thumb_medium()
                .edge_to_edge()
        });

        let red = gpui::hsla(0.0, 1.0, 0.5, 1.0);
        let blue = gpui::hsla(240.0 / 360.0, 1.0, 0.5, 1.0);
        let gradient_rgb = cx.new(|cx| {
            ColorSliderState::gradient("color-slider-gradient-rgb", 0.5, vec![red, blue], cx)
                .interpolation(ColorInterpolation::Rgb)
        });
        let gradient_hsl = cx.new(|cx| {
            ColorSliderState::gradient("color-slider-gradient-hsl", 0.5, vec![red, blue], cx)
                .interpolation(ColorInterpolation::Hsl)
        });
        let gradient_lab = cx.new(|cx| {
            ColorSliderState::gradient("color-slider-gradient-lab", 0.5, vec![red, blue], cx)
                .interpolation(ColorInterpolation::Lab)
        });

        let rgba = RgbaSpec { r: 255.0, g: 128.0, b: 0.0, a: 1.0 };
        let red_slider = cx.new(|cx| {
            ColorSliderState::channel(
                "color-slider-red",
                rgba.r,
                ChannelDelegate::new(rgba, RgbaSpec::RED.into()).expect("RGBA red delegate should be valid"),
                cx,
            )
            .min(0.0)
            .max(255.0)
        });
        let green_slider = cx.new(|cx| {
            ColorSliderState::channel(
                "color-slider-green",
                rgba.g,
                ChannelDelegate::new(rgba, RgbaSpec::GREEN.into()).expect("RGBA green delegate should be valid"),
                cx,
            )
            .min(0.0)
            .max(255.0)
        });
        let blue_slider = cx.new(|cx| {
            ColorSliderState::channel(
                "color-slider-blue",
                rgba.b,
                ChannelDelegate::new(rgba, RgbaSpec::BLUE.into()).expect("RGBA blue delegate should be valid"),
                cx,
            )
            .min(0.0)
            .max(255.0)
        });

        Self {
            hue_slider,
            saturation_slider,
            alpha_slider,
            gradient_rgb,
            gradient_hsl,
            gradient_lab,
            red_slider,
            green_slider,
            blue_slider,
            hsl,
            last_event: SharedString::from("Release"),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.hue_slider, |app, _, event: &ColorSliderEvent, cx| {
            app.panes.color_slider.handle_hue_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.saturation_slider, |app, _, event: &ColorSliderEvent, cx| {
            app.panes.color_slider.handle_saturation_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.alpha_slider, |app, _, event: &ColorSliderEvent, cx| {
            app.panes.color_slider.handle_alpha_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        let selected = self.hsl.to_hsla();

        color_gallery_pane(
            "Color Slider",
            "Color-slider primitives split out from the original project: direct hue or alpha delegates, interpolation comparisons, and channel sliders.",
            div()
                .w_full()
                .max_w(px(1120.0))
                .flex()
                .flex_col()
                .gap(px(28.0))
                .child(demo_section(
                    "Core Delegates",
                    "Primary interactive color sliders backed by hue, saturation, and alpha delegates.",
                    vec![demo_card(
                        "Interactive Set",
                        "The same core controls used to build larger pickers.",
                        WIDE_CARD,
                        div()
                            .w_full()
                            .flex()
                            .flex_col()
                            .gap(px(12.0))
                            .child(slider_row("Hue", self.hue_slider.clone(), look))
                            .child(slider_row("Saturation", self.saturation_slider.clone(), look))
                            .child(slider_row("Alpha", self.alpha_slider.clone(), look))
                            .child(
                                div()
                                    .mt(px(4.0))
                                    .h(px(44.0))
                                    .rounded(px(12.0))
                                    .border_1()
                                    .border_color(look.chrome().border)
                                    .bg(selected),
                            )
                            .child(detail_row("Hex", format_hex_color(selected), look))
                            .child(detail_row("HSLA", format_compact_hsla(selected), look))
                            .child(detail_row("Last Event", self.last_event.to_string(), look)),
                        look,
                    )],
                    look,
                ))
                .child(demo_section(
                    "Interpolation",
                    "Gradient interpolation paths from the upstream slider demos.",
                    vec![demo_card(
                        "Gradient Compare",
                        "RGB, HSL, and Lab interpolation over the same endpoints.",
                        NARROW_CARD,
                        div()
                            .w_full()
                            .flex()
                            .flex_col()
                            .gap(px(12.0))
                            .child(slider_row("RGB", self.gradient_rgb.clone(), look))
                            .child(slider_row("HSL", self.gradient_hsl.clone(), look))
                            .child(slider_row("Lab", self.gradient_lab.clone(), look)),
                        look,
                    )],
                    look,
                ))
                .child(demo_section(
                    "Channels",
                    "Representative RGB channel delegates, matching the spirit of the original page without porting every variant yet.",
                    vec![demo_card(
                        "RGBA Channels",
                        "Direct red, green, and blue channel sliders.",
                        NARROW_CARD,
                        div()
                            .w_full()
                            .flex()
                            .flex_col()
                            .gap(px(12.0))
                            .child(slider_row("Red", self.red_slider.clone(), look))
                            .child(slider_row("Green", self.green_slider.clone(), look))
                            .child(slider_row("Blue", self.blue_slider.clone(), look)),
                        look,
                    )],
                    look,
                ))
                .into_any_element(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        for entity in [
            &self.hue_slider,
            &self.saturation_slider,
            &self.alpha_slider,
            &self.gradient_rgb,
            &self.gradient_hsl,
            &self.gradient_lab,
            &self.red_slider,
            &self.green_slider,
            &self.blue_slider,
        ] {
            notify_entity(entity, cx);
        }
    }

    fn handle_hue_event(&mut self, event: &ColorSliderEvent, cx: &mut Context<GalleryApp>) {
        let value = self.capture_event("Hue", event);
        self.hsl.h = value;
        self.sync_delegates(cx);
        cx.notify();
    }

    fn handle_saturation_event(&mut self, event: &ColorSliderEvent, cx: &mut Context<GalleryApp>) {
        let value = self.capture_event("Saturation", event);
        self.hsl.s = value;
        self.sync_delegates(cx);
        cx.notify();
    }

    fn handle_alpha_event(&mut self, event: &ColorSliderEvent, cx: &mut Context<GalleryApp>) {
        let value = self.capture_event("Alpha", event);
        self.hsl.a = value;
        self.sync_delegates(cx);
        cx.notify();
    }

    fn capture_event(&mut self, source: &str, event: &ColorSliderEvent) -> f32 {
        match event {
            ColorSliderEvent::Change(value) => {
                self.last_event = SharedString::from(format!("{source} Change {:.3}", value));
                *value
            }
            ColorSliderEvent::Release(value) => {
                self.last_event = SharedString::from(format!("{source} Release {:.3}", value));
                *value
            }
        }
    }

    fn sync_delegates(&self, cx: &mut Context<GalleryApp>) {
        let hsl = self.hsl;
        self.saturation_slider.update(cx, |slider, cx| {
            slider.set_delegate(
                Box::new(
                    ChannelDelegate::new(hsl, Hsl::SATURATION.into()).expect("HSL saturation delegate should be valid"),
                ),
                cx,
            );
        });
        self.alpha_slider.update(cx, |slider, cx| {
            slider.set_delegate(Box::new(AlphaDelegate { spec: hsl }), cx);
        });
    }
}

fn slider_row(label: &'static str, slider: Entity<ColorSliderState>, look: &ShadcnLook) -> AnyElement {
    div()
        .w_full()
        .flex()
        .items_center()
        .gap(px(12.0))
        .child(div().w(px(74.0)).child(control_label(label, look)))
        .child(div().flex_1().child(slider))
        .into_any_element()
}
