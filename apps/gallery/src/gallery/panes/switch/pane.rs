use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Entity, FontWeight, IntoElement, Render, SharedString, Subscription, Window, div,
    prelude::*, px,
};
use gpui_luma::controls::command::button::{ButtonEvent, ButtonRenderModel, ButtonTemplate, HasPresenter};
use gpui_luma::controls::switch::Switch;
use gpui_luma::controls::button_family::{ButtonFamilyRole, ButtonSize};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma::theme::{InteractionState};
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};

use crate::gallery::control::GalleryApp;

use super::super::button::labeling::render_vertical_section_rail;
use super::super::shared::{gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct SwitchPane {
    secondary_switch: Switch,
    primary_switch: Switch,
    state_preview: Entity<SwitchStatePreview>,
    secondary_on: bool,
    primary_on: bool,
}

impl SwitchPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        Self {
            secondary_switch: look
                .secondary_switch("switch-secondary-example")
                .with_data(true)
                .content(|_, _| div().into_any_element())
                .spawn(cx),
            primary_switch: look
                .primary_switch("switch-primary-example")
                .with_data(false)
                .content(|_, _| div().into_any_element())
                .spawn(cx),
            state_preview: cx.new(|_| SwitchStatePreview::new(look)),
            secondary_on: true,
            primary_on: false,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.secondary_switch, |app, _, event: &ButtonEvent, cx| {
            app.panes.switch.handle_secondary_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.primary_switch, |app, _, event: &ButtonEvent, cx| {
            app.panes.switch.handle_primary_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        let chrome = look.chrome();

        gallery_pane_with_usage(
            "Switch",
            "Switch",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_5()
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .justify_center()
                        .gap(px(12.0))
                        .child(self.primary_switch.clone())
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.body_text)
                                .child(format!("Primary on: {}", self.primary_on)),
                        )
                        .child(self.secondary_switch.clone())
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.body_text)
                                .child(format!("Secondary on: {}", self.secondary_on)),
                        ),
                )
                .child(self.state_preview.clone())
                .into_any_element(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.secondary_switch, cx);
        notify_entity(&self.primary_switch, cx);
        notify_entity(&self.state_preview, cx);
    }

    fn flip_switch(switch: &Switch, on: &mut bool, cx: &mut Context<GalleryApp>) {
        switch.update(cx, |button, cx| {
            let new_on = !*button.data();
            button.set_data(new_on, cx);
            *on = new_on;
        });
    }

    fn handle_secondary_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
            Self::flip_switch(&self.secondary_switch, &mut self.secondary_on, cx);
            cx.notify();
        }
    }

    fn handle_primary_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
            Self::flip_switch(&self.primary_switch, &mut self.primary_on, cx);
            cx.notify();
        }
    }
}

#[derive(Clone)]
struct SwitchStatePreview {
    look: Arc<ShadcnLook>,
}

struct SwitchStateSample {
    id: &'static str,
    header: &'static str,
    state: InteractionState,
}

#[derive(Clone, Copy)]
enum SwitchTemplateVariant {
    TrackOff,
    TrackOn,
    LabeledOff,
    LabeledOn,
}

impl SwitchTemplateVariant {
    fn id(self) -> &'static str {
        match self {
            Self::TrackOff => "track-off",
            Self::TrackOn => "track-on",
            Self::LabeledOff => "labeled-off",
            Self::LabeledOn => "labeled-on",
        }
    }

    fn on(self) -> bool {
        matches!(self, Self::TrackOn | Self::LabeledOn)
    }

    fn content(self) -> Arc<dyn Fn(&ButtonRenderModel<bool>, &mut App) -> AnyElement + Send + Sync> {
        match self {
            Self::TrackOff | Self::TrackOn => Arc::new(move |_, _| div().into_any_element()),
            Self::LabeledOff | Self::LabeledOn => {
                let label = SharedString::from("Switch");
                Arc::new(move |_, _| div().child(label.clone()).into_any_element())
            }
        }
    }
}

impl SwitchStatePreview {
    fn new(look: Arc<ShadcnLook>) -> Self {
        Self { look }
    }
}

impl Render for SwitchStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let samples = [
            SwitchStateSample { id: "default", header: "default", state: InteractionState::default() },
            SwitchStateSample {
                id: "hover",
                header: "hover",
                state: InteractionState { hovered: true, ..InteractionState::default() },
            },
            SwitchStateSample {
                id: "focused",
                header: "focused",
                state: InteractionState { focused: true, ..InteractionState::default() },
            },
            SwitchStateSample {
                id: "pressed",
                header: "pressed",
                state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
            },
            SwitchStateSample {
                id: "disabled",
                header: "disabled",
                state: InteractionState { disabled: true, ..InteractionState::default() },
            },
        ];
        let variants = [
            SwitchTemplateVariant::TrackOff,
            SwitchTemplateVariant::TrackOn,
            SwitchTemplateVariant::LabeledOff,
            SwitchTemplateVariant::LabeledOn,
        ];

        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(16.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(chrome.muted_text)
                    .child("Template matrix preview"),
            )
            .child(div().flex().flex_col().items_start().gap(px(20.0)).children([
                render_section(
                    &self.look,
                    "Primary",
                    ShadcnButtonStyle::Primary,
                    &variants,
                    &samples,
                    chrome.muted_text,
                    window,
                    cx,
                ),
                render_section(
                    &self.look,
                    "Secondary",
                    ShadcnButtonStyle::Secondary,
                    &variants,
                    &samples,
                    chrome.muted_text,
                    window,
                    cx,
                ),
            ]))
    }
}

fn render_section(
    look: &Arc<ShadcnLook>,
    section_label: &'static str,
    style: ShadcnButtonStyle,
    variants: &[SwitchTemplateVariant],
    samples: &[SwitchStateSample],
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let template = look.switch_template(style);
    div()
        .flex()
        .items_start()
        .gap(px(8.0))
        .child(render_vertical_section_rail(section_label, label_color))
        .child(
            div()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(8.0))
                .child(render_header_row(samples, label_color))
                .children(
                    variants.iter().map(|variant| render_variant_row(&template, style, *variant, samples, window, cx)),
                ),
        )
        .into_any_element()
}

fn render_header_row(samples: &[SwitchStateSample], label_color: gpui::Hsla) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .children(samples.iter().map(|sample| {
            div()
                .w(px(116.0))
                .flex()
                .justify_center()
                .text_size(px(11.0))
                .line_height(px(15.0))
                .text_color(label_color)
                .child(sample.header)
        }))
        .into_any_element()
}

fn render_variant_row(
    template: &Arc<dyn ButtonTemplate<bool>>,
    style: ShadcnButtonStyle,
    variant: SwitchTemplateVariant,
    samples: &[SwitchStateSample],
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .children(samples.iter().map(|sample| render_state_sample(template, style, variant, sample, window, cx)))
        .into_any_element()
}

fn render_state_sample(
    template: &Arc<dyn ButtonTemplate<bool>>,
    style: ShadcnButtonStyle,
    variant: SwitchTemplateVariant,
    sample: &SwitchStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let on = variant.on();
    let id = SharedString::from(format!("switch-preview-{}-{}-{}", shadcn_style_id(style), variant.id(), sample.id));
    let model = ButtonRenderModel {
        id,
        data: on,
        content: variant.content(),
        role: ButtonFamilyRole::Text,
        size: ButtonSize::Md,
        state: sample.state,
        round: false,
        radius_override: std::cell::Cell::new(None),
        appearance: None,
    };

    div()
        .w(px(116.0))
        .flex()
        .justify_center()
        .items_center()
        .child(template.render(&model, window, cx))
        .into_any_element()
}

fn shadcn_style_id(style: ShadcnButtonStyle) -> &'static str {
    match style {
        ShadcnButtonStyle::Primary => "primary",
        ShadcnButtonStyle::Secondary => "secondary",
        ShadcnButtonStyle::Outline => "outline",
        ShadcnButtonStyle::Ghost => "ghost",
    }
}
