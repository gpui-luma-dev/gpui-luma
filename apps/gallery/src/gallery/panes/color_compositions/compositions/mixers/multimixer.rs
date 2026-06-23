use std::sync::Arc;

use gpui::{Context, Entity, IntoElement, Subscription, div, prelude::*, px};
use gpui_luma::controls::color::color_slider::color_spec::{
    ColorChannel, ColorSpecification, Hsl, Hsv, HueAlpha, Lab, RgbaSpec, slider_step_for_channel,
};
use gpui_luma::controls::color::ColorSwatch;
use gpui_luma::controls::color::color_slider::{
    AlphaDelegate, ChannelDelegate, ColorSliderBuilder, ColorSliderDelegate, ColorSliderDomainRenderer, HueDelegate,
    primary_slider_value, refresh_color_slider, update_domain_delegate,
};
use gpui_luma::controls::slider::{SliderControl, SliderEvent};
use gpui_luma::theme::ControlSize;
use gpui_luma::wrappanel;
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook};

use crate::gallery::panes::shared::{format_compact_hsla, format_hex_color};
use crate::gallery::panes::color::common::{control_label, demo_card, detail_row, notify_control};

pub(in crate::gallery) struct MultiMixerState {
    look: Arc<ShadcnLook>,
    hue_alpha: Entity<ColorSpaceMixerState<HueAlpha>>,
    rgb: Entity<ColorSpaceMixerState<RgbaSpec>>,
    hsla: Entity<ColorSpaceMixerState<Hsl>>,
    hsva: Entity<ColorSpaceMixerState<Hsv>>,
    lab: Entity<ColorSpaceMixerState<Lab>>,
    lab_auto: Entity<ColorSpaceMixerState<Lab>>,
    lab_dynamic: Entity<ColorSpaceMixerState<Lab>>,
}

impl MultiMixerState {
    pub(in crate::gallery) fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
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

        Self { look, hue_alpha, rgb, hsla, hsva, lab, lab_auto, lab_dynamic }
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<Self>) {
        self.hue_alpha.update(cx, |state, cx| state.notify_controls(cx));
        self.rgb.update(cx, |state, cx| state.notify_controls(cx));
        self.hsla.update(cx, |state, cx| state.notify_controls(cx));
        self.hsva.update(cx, |state, cx| state.notify_controls(cx));
        self.lab.update(cx, |state, cx| state.notify_controls(cx));
        self.lab_auto.update(cx, |state, cx| state.notify_controls(cx));
        self.lab_dynamic.update(cx, |state, cx| state.notify_controls(cx));
    }
}

impl gpui::Render for MultiMixerState {
    fn render(&mut self, _window: &mut gpui::Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().w_full().flex().flex_col().gap(px(28.0)).child(
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
                                .typography_style(self.look.typography_role(gpui_luma_look_shadcn::ShadcnTextRole::H4))
                                .text_color(self.look.chrome().title_text)
                                .child("Mixer Modes"),
                        )
                        .child(
                            div()
                                .typography_style(self.look.typography_scale(gpui_luma_look_shadcn::ShadcnTextSize::Sm))
                                .text_color(self.look.chrome().muted_text)
                                .child("Representative mixers for the upstream mode set."),
                        ),
                )
                .child(
                    wrappanel! {
                        orientation=horizontal gap=16.0 align=start;
                        mixer_card("Hue + Alpha", self.hue_alpha.clone(), &self.look),
                        mixer_card("RGB", self.rgb.clone(), &self.look),
                        mixer_card("HSLA", self.hsla.clone(), &self.look),
                        mixer_card("HSVA", self.hsva.clone(), &self.look),
                        mixer_card("Lab", self.lab.clone(), &self.look),
                        mixer_card("Lab Auto-clamped", self.lab_auto.clone(), &self.look),
                        mixer_card("Lab Dynamic Range", self.lab_dynamic.clone(), &self.look)
                    }
                    .w_full(),
                ),
        )
    }
}

fn mixer_card(title: &'static str, mixer: impl IntoElement, look: &ShadcnLook) -> gpui::AnyElement {
    demo_card(title, "Interactive per-space channel mixer.", 340.0, mixer, look)
}

struct MixerSliderRow {
    channel: ColorChannel,
    slider: Entity<SliderControl>,
    domain_renderer: Arc<ColorSliderDomainRenderer>,
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
            let builder = if channel_meta.name == "hue" {
                ColorSliderBuilder::hue(format!("{}-{}", title, channel_meta.name), initial_value)
            } else if channel_meta.name == "alpha" {
                ColorSliderBuilder::alpha(format!("{}-{}", title, channel_meta.name), initial_value, spec)
            } else {
                ColorSliderBuilder::channel(
                    format!("{}-{}", title, channel_meta.name),
                    initial_value,
                    spec,
                    channel_meta.name,
                )
                .expect("color mixer channel delegate should be valid")
            };
            let domain_renderer = builder.domain_renderer();
            let slider = builder
                .size(ControlSize::Sm)
                .thumb_medium()
                .edge_to_edge()
                .range(channel_meta.min..channel_meta.max)
                .step(slider_step_for_channel(&channel_meta))
                .spawn(cx);
            sliders.push(MixerSliderRow { channel: channel_meta, slider, domain_renderer });
        }

        let subscriptions = sliders
            .iter()
            .map(|row| {
                let channel_name = row.channel.name;
                cx.subscribe(&row.slider, move |this, _, event: &SliderEvent, cx| {
                    if let Some(value) = primary_slider_value(event) {
                        this.handle_slider_event(channel_name, value, cx);
                    }
                })
            })
            .collect();

        let state = Self { subtitle, look, spec, sliders, _subscriptions: subscriptions };
        state.sync_sliders(cx);
        state
    }

    fn notify_controls(&self, cx: &mut Context<Self>) {
        for row in &self.sliders {
            notify_control(&row.slider, cx);
        }
    }

    fn handle_slider_event(&mut self, channel_name: &'static str, proposed: f32, cx: &mut Context<Self>) {
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
            let (min, max) = spec.channel_bounds(channel.name);
            row.slider.update(cx, |slider, cx| {
                slider.set_range(min..max, cx);
                if let Some((allowed_min, allowed_max)) = spec.channel_allowed_interval(channel.name) {
                    slider.set_allowed_intervals(vec![allowed_min..=allowed_max], cx);
                } else {
                    slider.clear_allowed_intervals(cx);
                }
                slider.set_value(spec.get_value(channel.name), cx);
            });
            let mut context = row.domain_renderer.context();
            context.range = min..max;
            update_domain_delegate(&row.domain_renderer, mixer_delegate(spec, channel.name), context);
            refresh_color_slider(&row.slider, cx);
        }
    }
}

fn render_constructed_color_swatch(color: gpui::Hsla, _look: &ShadcnLook) -> gpui::AnyElement {
    ColorSwatch::new(color).checkerboard(true).height(px(44.0)).rounded(px(12.0)).into_any_element()
}

fn mixer_delegate<S: ColorSpecification>(spec: S, channel_name: &'static str) -> Arc<dyn ColorSliderDelegate> {
    if channel_name == "hue" {
        Arc::new(HueDelegate)
    } else if channel_name == "alpha" {
        Arc::new(AlphaDelegate { spec })
    } else {
        Arc::new(ChannelDelegate::new(spec, channel_name.into()).expect("color mixer channel delegate should be valid"))
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
