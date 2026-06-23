use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, SharedString, Subscription, div, prelude::*, px};
use gpui_luma::controls::color::color_slider::color_spec::{Hsl, RgbaSpec};
use gpui_luma::controls::color::color_slider::{
    AlphaDelegate, ChannelDelegate, ColorInterpolation, ColorSliderBuilder, ColorSliderDomainRenderer,
    ColorSpecification, refresh_color_slider, update_domain_delegate,
};
use gpui_luma::controls::color::ColorSwatch;
use gpui_luma::controls::slider::{SliderControl, SliderEvent};
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{format_compact_hsla, format_hex_color, notify_entity};

use super::common::{color_gallery_pane, control_label, demo_card, demo_section, detail_row};

const WIDE_CARD: f32 = 420.0;
const NARROW_CARD: f32 = 320.0;

#[derive(Clone)]
pub(in crate::gallery) struct ColorSliderPane {
    hue_slider: Entity<SliderControl>,
    saturation_slider: Entity<SliderControl>,
    alpha_slider: Entity<SliderControl>,
    saturation_domain: Arc<ColorSliderDomainRenderer>,
    alpha_domain: Arc<ColorSliderDomainRenderer>,
    gradient_rgb: Entity<SliderControl>,
    gradient_hsl: Entity<SliderControl>,
    gradient_lab: Entity<SliderControl>,
    red_slider: Entity<SliderControl>,
    green_slider: Entity<SliderControl>,
    blue_slider: Entity<SliderControl>,
    hsl: Hsl,
    last_event: SharedString,
}

impl ColorSliderPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, _look: Arc<ShadcnLook>) -> Self {
        let hsl = Hsl { h: 210.0, s: 0.72, l: 0.52, a: 0.85 };

        let hue_slider =
            ColorSliderBuilder::hue("color-slider-hue", hsl.h).size(ControlSize::Sm).thumb_medium().spawn(cx);
        let saturation_builder = ColorSliderBuilder::channel("color-slider-saturation", hsl.s, hsl, Hsl::SATURATION)
            .expect("HSL saturation delegate should be valid")
            .size(ControlSize::Sm)
            .thumb_medium()
            .edge_to_edge();
        let saturation_domain = saturation_builder.domain_renderer();
        let saturation_slider = saturation_builder.spawn(cx);
        let alpha_builder = ColorSliderBuilder::alpha("color-slider-alpha", hsl.a, hsl)
            .size(ControlSize::Sm)
            .thumb_medium()
            .edge_to_edge();
        let alpha_domain = alpha_builder.domain_renderer();
        let alpha_slider = alpha_builder.spawn(cx);

        let red = gpui::hsla(0.0, 1.0, 0.5, 1.0);
        let blue = gpui::hsla(240.0 / 360.0, 1.0, 0.5, 1.0);
        let gradient_rgb = ColorSliderBuilder::gradient("color-slider-gradient-rgb", 0.5, vec![red, blue])
            .interpolation(ColorInterpolation::Rgb)
            .spawn(cx);
        let gradient_hsl = ColorSliderBuilder::gradient("color-slider-gradient-hsl", 0.5, vec![red, blue])
            .interpolation(ColorInterpolation::Hsl)
            .spawn(cx);
        let gradient_lab = ColorSliderBuilder::gradient("color-slider-gradient-lab", 0.5, vec![red, blue])
            .interpolation(ColorInterpolation::Lab)
            .spawn(cx);

        let rgba = RgbaSpec { r: 255.0, g: 128.0, b: 0.0, a: 1.0 };
        let red_slider = ColorSliderBuilder::channel("color-slider-red", rgba.r, rgba, RgbaSpec::RED)
            .expect("RGBA red delegate should be valid")
            .min(0.0)
            .max(255.0)
            .spawn(cx);
        let green_slider = ColorSliderBuilder::channel("color-slider-green", rgba.g, rgba, RgbaSpec::GREEN)
            .expect("RGBA green delegate should be valid")
            .min(0.0)
            .max(255.0)
            .spawn(cx);
        let blue_slider = ColorSliderBuilder::channel("color-slider-blue", rgba.b, rgba, RgbaSpec::BLUE)
            .expect("RGBA blue delegate should be valid")
            .min(0.0)
            .max(255.0)
            .spawn(cx);

        Self {
            hue_slider,
            saturation_slider,
            alpha_slider,
            saturation_domain,
            alpha_domain,
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
        subscriptions.push(cx.subscribe(&self.hue_slider, |app, _, event: &SliderEvent, cx| {
            if let Some((value, label)) = slider_event_label("Hue", event) {
                app.panes.color_slider.last_event = label;
                app.panes.color_slider.handle_hue_event(value, cx);
            }
        }));
        subscriptions.push(cx.subscribe(&self.saturation_slider, |app, _, event: &SliderEvent, cx| {
            if let Some((value, label)) = slider_event_label("Saturation", event) {
                app.panes.color_slider.last_event = label;
                app.panes.color_slider.handle_saturation_event(value, cx);
            }
        }));
        subscriptions.push(cx.subscribe(&self.alpha_slider, |app, _, event: &SliderEvent, cx| {
            if let Some((value, label)) = slider_event_label("Alpha", event) {
                app.panes.color_slider.last_event = label;
                app.panes.color_slider.handle_alpha_event(value, cx);
            }
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
                            .child(ColorSwatch::new(selected).checkerboard(true).height(px(44.0)).rounded(px(12.0)).into_any_element())
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

    fn handle_hue_event(&mut self, value: f32, cx: &mut Context<GalleryApp>) {
        self.hsl.h = value;
        self.sync_delegates(cx);
        cx.notify();
    }

    fn handle_saturation_event(&mut self, value: f32, cx: &mut Context<GalleryApp>) {
        self.hsl.s = value;
        self.sync_delegates(cx);
        cx.notify();
    }

    fn handle_alpha_event(&mut self, value: f32, cx: &mut Context<GalleryApp>) {
        self.hsl.a = value;
        self.sync_delegates(cx);
        cx.notify();
    }

    fn sync_delegates(&self, cx: &mut Context<GalleryApp>) {
        let hsl = self.hsl;
        update_domain_delegate(
            &self.saturation_domain,
            Arc::new(
                ChannelDelegate::new(hsl, Hsl::SATURATION.into()).expect("HSL saturation delegate should be valid"),
            ),
            self.saturation_domain.context(),
        );
        refresh_color_slider(&self.saturation_slider, cx);
        update_domain_delegate(&self.alpha_domain, Arc::new(AlphaDelegate { spec: hsl }), self.alpha_domain.context());
        refresh_color_slider(&self.alpha_slider, cx);
    }
}

fn slider_event_label(source: &str, event: &SliderEvent) -> Option<(f32, SharedString)> {
    match event {
        SliderEvent::Change { value, .. } => Some((*value, SharedString::from(format!("{source} Change {value:.3}")))),
        SliderEvent::Release { value, .. } => {
            Some((*value, SharedString::from(format!("{source} Release {value:.3}"))))
        }
        _ => None,
    }
}

fn slider_row(label: &'static str, slider: Entity<SliderControl>, look: &ShadcnLook) -> AnyElement {
    div()
        .w_full()
        .flex()
        .items_center()
        .gap(px(12.0))
        .child(div().w(px(74.0)).child(control_label(label, look)))
        .child(div().flex_1().child(slider))
        .into_any_element()
}
