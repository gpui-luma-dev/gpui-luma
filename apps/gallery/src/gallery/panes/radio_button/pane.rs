use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Entity, FontWeight, IntoElement, Render, SharedString, Subscription, Window, div,
    prelude::*, px,
};
use gpui_luma::controls::command::button::{ButtonEvent, ButtonRenderModel, ButtonTemplate, HasPresenter};
use gpui_luma::controls::radio_button::RadioButton;
use gpui_luma::controls::button_family::{ButtonFamilyRole, ButtonSize};
use gpui_luma_theme_radix::prelude::*;
use gpui_luma::theme::{InteractionState};
use gpui_luma_theme_radix::{RadixButtonStyle, RadixTheme};

use crate::gallery::control::GalleryApp;

use super::super::button::labeling::render_vertical_section_rail;
use super::super::shared::{gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct RadioButtonPane {
    secondary_radio: RadioButton,
    primary_radio: RadioButton,
    state_preview: Entity<RadioButtonStatePreview>,
    secondary_selected: bool,
    primary_selected: bool,
}

impl RadioButtonPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, radix_theme: Arc<RadixTheme>) -> Self {
        Self {
            secondary_radio: radix_theme
                .secondary_radio("radio-button-secondary-example")
                .with_data(true)
                .content(|_, _| div().child("Secondary").into_any_element())
                .spawn(cx),
            primary_radio: radix_theme
                .primary_radio("radio-button-primary-example")
                .with_data(false)
                .content(|_, _| div().child("Primary").into_any_element())
                .spawn(cx),
            state_preview: cx.new(|_| RadioButtonStatePreview::new(radix_theme)),
            secondary_selected: true,
            primary_selected: false,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.secondary_radio, |app, _, event: &ButtonEvent, cx| {
            app.panes.radio_button.handle_secondary_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.primary_radio, |app, _, event: &ButtonEvent, cx| {
            app.panes.radio_button.handle_primary_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, radix_theme: &RadixTheme) -> AnyElement {
        let chrome = radix_theme.chrome();

        gallery_pane_with_usage(
            "Radio Button",
            "Radio Button",
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
                        .child(self.primary_radio.clone())
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.body_text)
                                .child(format!("Primary selected: {}", self.primary_selected)),
                        )
                        .child(self.secondary_radio.clone())
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.body_text)
                                .child(format!("Secondary selected: {}", self.secondary_selected)),
                        ),
                )
                .child(self.state_preview.clone())
                .into_any_element(),
            radix_theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.secondary_radio, cx);
        notify_entity(&self.primary_radio, cx);
        notify_entity(&self.state_preview, cx);
    }

    fn flip_radio(radio: &RadioButton, selected: &mut bool, cx: &mut Context<GalleryApp>) {
        radio.update(cx, |button, cx| {
            let new_selected = !*button.data();
            button.set_data(new_selected, cx);
            *selected = new_selected;
        });
    }

    fn handle_secondary_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
            Self::flip_radio(&self.secondary_radio, &mut self.secondary_selected, cx);
            cx.notify();
        }
    }

    fn handle_primary_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
            Self::flip_radio(&self.primary_radio, &mut self.primary_selected, cx);
            cx.notify();
        }
    }
}

#[derive(Clone)]
struct RadioButtonStatePreview {
    radix_theme: Arc<RadixTheme>,
}

struct RadioButtonStateSample {
    id: &'static str,
    header: &'static str,
    state: InteractionState,
}

#[derive(Clone, Copy)]
enum RadioButtonTemplateVariant {
    IndicatorUnselected,
    IndicatorSelected,
    LabeledUnselected,
    LabeledSelected,
}

impl RadioButtonTemplateVariant {
    fn id(self) -> &'static str {
        match self {
            Self::IndicatorUnselected => "indicator-unselected",
            Self::IndicatorSelected => "indicator-selected",
            Self::LabeledUnselected => "labeled-unselected",
            Self::LabeledSelected => "labeled-selected",
        }
    }

    fn selected(self) -> bool {
        matches!(self, Self::IndicatorSelected | Self::LabeledSelected)
    }

    fn content(self) -> Arc<dyn Fn(&ButtonRenderModel<bool>, &mut App) -> AnyElement + Send + Sync> {
        match self {
            Self::IndicatorUnselected | Self::IndicatorSelected => Arc::new(move |_, _| div().into_any_element()),
            Self::LabeledUnselected | Self::LabeledSelected => {
                let label = SharedString::from("Radio");
                Arc::new(move |_, _| div().child(label.clone()).into_any_element())
            }
        }
    }
}

impl RadioButtonStatePreview {
    fn new(radix_theme: Arc<RadixTheme>) -> Self {
        Self { radix_theme }
    }
}

impl Render for RadioButtonStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.radix_theme.chrome();
        let samples = [
            RadioButtonStateSample { id: "default", header: "default", state: InteractionState::default() },
            RadioButtonStateSample {
                id: "hover",
                header: "hover",
                state: InteractionState { hovered: true, ..InteractionState::default() },
            },
            RadioButtonStateSample {
                id: "focused",
                header: "focused",
                state: InteractionState { focused: true, ..InteractionState::default() },
            },
            RadioButtonStateSample {
                id: "pressed",
                header: "pressed",
                state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
            },
            RadioButtonStateSample {
                id: "disabled",
                header: "disabled",
                state: InteractionState { disabled: true, ..InteractionState::default() },
            },
        ];
        let variants = [
            RadioButtonTemplateVariant::IndicatorUnselected,
            RadioButtonTemplateVariant::IndicatorSelected,
            RadioButtonTemplateVariant::LabeledUnselected,
            RadioButtonTemplateVariant::LabeledSelected,
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
                    &self.radix_theme,
                    "Primary",
                    RadixButtonStyle::Primary,
                    &variants,
                    &samples,
                    chrome.muted_text,
                    window,
                    cx,
                ),
                render_section(
                    &self.radix_theme,
                    "Secondary",
                    RadixButtonStyle::Secondary,
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
    radix_theme: &Arc<RadixTheme>,
    section_label: &'static str,
    style: RadixButtonStyle,
    variants: &[RadioButtonTemplateVariant],
    samples: &[RadioButtonStateSample],
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let template = radix_theme.radio_button_template(style);
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

fn render_header_row(samples: &[RadioButtonStateSample], label_color: gpui::Hsla) -> AnyElement {
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
    style: RadixButtonStyle,
    variant: RadioButtonTemplateVariant,
    samples: &[RadioButtonStateSample],
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
    style: RadixButtonStyle,
    variant: RadioButtonTemplateVariant,
    sample: &RadioButtonStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let selected = variant.selected();
    let id =
        SharedString::from(format!("radio-button-preview-{}-{}-{}", radix_style_id(style), variant.id(), sample.id));
    let model = ButtonRenderModel {
        id,
        data: selected,
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

fn radix_style_id(style: RadixButtonStyle) -> &'static str {
    match style {
        RadixButtonStyle::Primary => "primary",
        RadixButtonStyle::Secondary => "secondary",
        RadixButtonStyle::Outline => "outline",
        RadixButtonStyle::Ghost => "ghost",
    }
}
