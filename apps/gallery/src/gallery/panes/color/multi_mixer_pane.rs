use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, IntoElement, Subscription, div, prelude::*, px};
use gpui_luma::controls::color::color_slider::color_spec::{
    ColorChannel, ColorSpecification, Hsl, Hsv, HueAlpha, Lab, RgbaSpec,
};
use gpui_luma::controls::color::ColorSwatch;
use gpui_luma::wrappanel;
use gpui_luma::controls::color::color_slider::{
    AlphaDelegate, ChannelDelegate, ColorSliderDelegate, ColorSliderEvent, ColorSliderState, HueDelegate,
};
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook};

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{format_compact_hsla, format_hex_color, notify_entity};

use super::common::{color_gallery_pane, control_label, demo_card, detail_row, notify_control};

#[derive(Clone)]
pub(in crate::gallery) struct MultiMixerPane {
    hue_alpha: Entity<ColorSpaceMixerState<HueAlpha>>,
    rgb: Entity<ColorSpaceMixerState<RgbaSpec>>,
    hsla: Entity<ColorSpaceMixerState<Hsl>>,
    hsva: Entity<ColorSpaceMixerState<Hsv>>,
    lab: Entity<ColorSpaceMixerState<Lab>>,
    lab_auto: Entity<ColorSpaceMixerState<Lab>>,
    lab_dynamic: Entity<ColorSpaceMixerState<Lab>>,
}

impl MultiMixerPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let initial = gpui::hsla(0.55, 1.0, 0.5, 1.0);
        let hue_alpha =
            cx.new(|cx| ColorSpaceMixerState::new("Hue + Alpha", None, look.clone(), HueAlpha::from_hsla(initial), cx));
        let rgb = cx.new(|cx| ColorSpaceMixerState::new("RGB", None, look.clone(), RgbaSpec::from_hsla(initial), cx));
        let hsla = cx.new(|cx| ColorSpaceMixerState::new("HSLA", None, look.clone(), Hsl::from_hsla(initial), cx));
        let hsva = cx.new(|cx| ColorSpaceMixerState::new("HSVA", None, look.clone(), Hsv::from_hsla(initial), cx));
        let lab =
            cx.new(|cx| ColorSpaceMixerState::new("Lab", Some("Unclamped"), look.clone(), Lab::from_hsla(initial), cx));
        let lab_auto = cx.new(|cx| {
            let mut spec = Lab::from_hsla(initial);
            spec.set_auto_clamp(true);
            ColorSpaceMixerState::new("Lab", Some("Auto-clamped"), look.clone(), spec, cx)
        });
        let lab_dynamic = cx.new(|cx| {
            let mut spec = Lab::from_hsla(initial);
            spec.set_dynamic_range(true);
            ColorSpaceMixerState::new("Lab", Some("Dynamic range"), look.clone(), spec, cx)
        });

        Self { hue_alpha, rgb, hsla, hsva, lab, lab_auto, lab_dynamic }
    }

    pub(in crate::gallery) fn subscribe(&self, _cx: &mut Context<GalleryApp>, _subscriptions: &mut Vec<Subscription>) {}

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        color_gallery_pane(
            "Multi Mixer",
            "The original mixer page explored multiple color spaces. This port keeps each space as its own interactive card so you can compare the slider primitives directly.",
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(28.0))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .child(
                                    div()
                                        .typography_style(
                                            look.typography_role(gpui_luma_look_shadcn::ShadcnTextRole::H4),
                                        )
                                        .text_color(look.chrome().title_text)
                                        .child("Mixer Modes"),
                                )
                                .child(
                                    div()
                                        .typography_style(
                                            look.typography_scale(gpui_luma_look_shadcn::ShadcnTextSize::Sm),
                                        )
                                        .text_color(look.chrome().muted_text)
                                        .child("Representative mixers for the upstream mode set."),
                                ),
                        )
                        .child(
                            wrappanel! {
                                orientation=horizontal gap=16.0 align=start;
                                mixer_card("Hue + Alpha", self.hue_alpha.clone(), look),
                                mixer_card("RGB", self.rgb.clone(), look),
                                mixer_card("HSLA", self.hsla.clone(), look),
                                mixer_card("HSVA", self.hsva.clone(), look),
                                mixer_card("Lab", self.lab.clone(), look),
                                mixer_card("Lab Auto-clamped", self.lab_auto.clone(), look),
                                mixer_card("Lab Dynamic Range", self.lab_dynamic.clone(), look)
                            }
                            .w_full(),
                        ),
                )
                .into_any_element(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        self.hue_alpha.update(cx, |state, cx| state.notify_controls(cx));
        self.rgb.update(cx, |state, cx| state.notify_controls(cx));
        self.hsla.update(cx, |state, cx| state.notify_controls(cx));
        self.hsva.update(cx, |state, cx| state.notify_controls(cx));
        self.lab.update(cx, |state, cx| state.notify_controls(cx));
        self.lab_auto.update(cx, |state, cx| state.notify_controls(cx));
        self.lab_dynamic.update(cx, |state, cx| state.notify_controls(cx));

        notify_entity(&self.hue_alpha, cx);
        notify_entity(&self.rgb, cx);
        notify_entity(&self.hsla, cx);
        notify_entity(&self.hsva, cx);
        notify_entity(&self.lab, cx);
        notify_entity(&self.lab_auto, cx);
        notify_entity(&self.lab_dynamic, cx);
    }
}

fn mixer_card(title: &'static str, mixer: impl IntoElement, look: &ShadcnLook) -> AnyElement {
    demo_card(title, "Interactive per-space channel mixer.", 340.0, mixer, look)
}

struct MixerSliderRow {
    channel: ColorChannel,
    slider: Entity<ColorSliderState>,
}

struct ColorSpaceMixerState<S: ColorSpecification> {
    subtitle: Option<&'static str>,
    look: Arc<ShadcnLook>,
    spec: S,
    sliders: Vec<MixerSliderRow>,
    _subscriptions: Vec<Subscription>,
}

impl<S: ColorSpecification> ColorSpaceMixerState<S> {
    fn new(
        title: &'static str,
        subtitle: Option<&'static str>,
        look: Arc<ShadcnLook>,
        spec: S,
        cx: &mut Context<Self>,
    ) -> Self {
        let channels = spec.channels().to_vec();
        let mut sliders = Vec::with_capacity(channels.len());

        for channel in &channels {
            let channel_meta = *channel;
            let initial_value = spec.get_value(channel_meta.name);
            let slider = cx.new(|cx| {
                let mut slider = if channel_meta.name == "hue" {
                    ColorSliderState::hue(format!("{}-{}", title, channel_meta.name), initial_value, cx)
                } else if channel_meta.name == "alpha" {
                    ColorSliderState::alpha(
                        format!("{}-{}", title, channel_meta.name),
                        initial_value,
                        AlphaDelegate { spec },
                        cx,
                    )
                } else {
                    let initial_delegate = ChannelDelegate::new(spec, channel_meta.name.into())
                        .expect("color mixer channel delegate should be valid");
                    ColorSliderState::channel(
                        format!("{}-{}", title, channel_meta.name),
                        initial_value,
                        initial_delegate,
                        cx,
                    )
                }
                .size(ControlSize::Sm)
                .thumb_medium()
                .edge_to_edge();
                slider.set_range(channel_meta.min, channel_meta.max, cx);
                slider
            });
            sliders.push(MixerSliderRow { channel: channel_meta, slider });
        }

        let subscriptions = sliders
            .iter()
            .map(|row| {
                let channel_name = row.channel.name;
                cx.subscribe(&row.slider, move |this, _, event: &ColorSliderEvent, cx| {
                    this.handle_slider_event(channel_name, event, cx);
                })
            })
            .collect();

        Self { subtitle, look, spec, sliders, _subscriptions: subscriptions }
    }

    fn notify_controls(&self, cx: &mut Context<Self>) {
        for row in &self.sliders {
            notify_control(&row.slider, cx);
        }
    }

    fn handle_slider_event(&mut self, channel_name: &'static str, event: &ColorSliderEvent, cx: &mut Context<Self>) {
        let proposed = match event {
            ColorSliderEvent::Change(value) | ColorSliderEvent::Release(value) => *value,
        };
        let clamped = self.spec.clamp_channel_to_gamut(channel_name, proposed);
        self.spec.set_value(channel_name, clamped);
        self.spec.clamp_spec_to_gamut();
        self.sync_sliders(cx);
        cx.notify();
    }

    fn sync_sliders(&self, cx: &mut Context<Self>) {
        let spec = self.spec;
        for row in &self.sliders {
            let channel = row.channel;
            row.slider.update(cx, |slider, cx| {
                let (min, max) = spec.channel_bounds(channel.name);
                slider.set_range(min, max, cx);
                slider.set_value(spec.get_value(channel.name), cx);
                slider.set_delegate(mixer_delegate(spec, channel.name), cx);
            });
        }
    }
}

fn render_constructed_color_swatch(color: gpui::Hsla, _look: &ShadcnLook) -> AnyElement {
    ColorSwatch::new(color).height(px(44.0)).rounded(px(12.0)).into_any_element()
}

fn mixer_delegate<S: ColorSpecification>(spec: S, channel_name: &'static str) -> Box<dyn ColorSliderDelegate> {
    if channel_name == "hue" {
        Box::new(HueDelegate)
    } else if channel_name == "alpha" {
        Box::new(AlphaDelegate { spec })
    } else {
        Box::new(ChannelDelegate::new(spec, channel_name.into()).expect("color mixer channel delegate should be valid"))
    }
}

impl<S: ColorSpecification> gpui::Render for ColorSpaceMixerState<S> {
    fn render(&mut self, _window: &mut gpui::Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let color = self.spec.to_hsla();

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .when_some(self.subtitle, |container, subtitle| {
                container.child(div().text_xs().text_color(self.look.chrome().muted_text).child(subtitle))
            })
            .child(render_constructed_color_swatch(color, &self.look))
            .child(detail_row("Hex", format_hex_color(color), &self.look))
            .child(detail_row("HSLA", format_compact_hsla(color), &self.look))
            .child(detail_row("Spec", self.spec.summary(), &self.look))
            .when(self.spec.is_out_of_gamut(), |div| {
                div.child(detail_row("Gamut", "Out of gamut".to_string(), &self.look))
            })
            .children(self.sliders.iter().map(|row| {
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    .child(div().w(px(82.0)).child(control_label(row.channel.label, &self.look)))
                    .child(div().flex_1().child(row.slider.clone()))
                    .into_any_element()
            }))
    }
}
