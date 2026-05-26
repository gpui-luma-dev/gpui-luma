use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, FontWeight, IntoElement, Render, SharedString, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::{ButtonEvent, ButtonRenderModel, ButtonTemplate, HasPresenter};
use gpui_luma::controls::radio_button::{self, RadioButton, default_radio_button_template};
use gpui_luma::controls::button_family::{ButtonFamilyRole, ButtonKind, ButtonSize};
use gpui_luma::theme::InteractionState;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::button::labeling::render_vertical_section_rail;
use super::super::shared::{gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct RadioButtonPane {
    standard_radio: RadioButton,
    prominent_radio: RadioButton,
    state_preview: Entity<RadioButtonStatePreview>,
    standard_selected: bool,
    prominent_selected: bool,
}

impl RadioButtonPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            standard_radio: radio_button::new("radio-button-standard-example")
                .kind(ButtonKind::Standard)
                .with_data(true)
                .content(|_, _| div().child("Standard").into_any_element())
                .spawn(cx),
            prominent_radio: radio_button::new("radio-button-prominent-example")
                .kind(ButtonKind::Prominent)
                .with_data(false)
                .content(|_, _| div().child("Prominent").into_any_element())
                .spawn(cx),
            state_preview: cx.new(|_| RadioButtonStatePreview::new(theme)),
            standard_selected: true,
            prominent_selected: false,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.standard_radio, |app, _, event: &ButtonEvent, cx| {
            app.panes.radio_button.handle_standard_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.prominent_radio, |app, _, event: &ButtonEvent, cx| {
            app.panes.radio_button.handle_prominent_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

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
                        .child(self.standard_radio.clone())
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.body_text)
                                .child(format!("Standard selected: {}", self.standard_selected)),
                        )
                        .child(self.prominent_radio.clone())
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.body_text)
                                .child(format!("Prominent selected: {}", self.prominent_selected)),
                        ),
                )
                .child(self.state_preview.clone())
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.standard_radio, cx);
        notify_entity(&self.prominent_radio, cx);
        notify_entity(&self.state_preview, cx);
    }

    fn flip_radio(radio: &RadioButton, selected: &mut bool, cx: &mut Context<GalleryApp>) {
        radio.update(cx, |button, cx| {
            let new_selected = !*button.data();
            button.set_data(new_selected, cx);
            *selected = new_selected;
        });
    }

    fn handle_standard_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
            Self::flip_radio(&self.standard_radio, &mut self.standard_selected, cx);
            cx.notify();
        }
    }

    fn handle_prominent_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
            Self::flip_radio(&self.prominent_radio, &mut self.prominent_selected, cx);
            cx.notify();
        }
    }
}

#[derive(Clone)]
struct RadioButtonStatePreview {
    theme: GalleryThemePack,
    template: Arc<dyn ButtonTemplate<bool>>,
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
    fn new(theme: &GalleryThemePack) -> Self {
        Self { theme: theme.clone(), template: default_radio_button_template() }
    }
}

impl Render for RadioButtonStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.theme.chrome();
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
                    &self.template,
                    "Prominent",
                    ButtonKind::Prominent,
                    &variants,
                    &samples,
                    chrome.muted_text,
                    window,
                    cx,
                ),
                render_section(
                    &self.template,
                    "Standard",
                    ButtonKind::Standard,
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
    template: &Arc<dyn ButtonTemplate<bool>>,
    section_label: &'static str,
    kind: ButtonKind,
    variants: &[RadioButtonTemplateVariant],
    samples: &[RadioButtonStateSample],
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
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
                .children(variants.iter().map(|variant| {
                    render_variant_row(template, kind, *variant, samples, window, cx)
                })),
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
    kind: ButtonKind,
    variant: RadioButtonTemplateVariant,
    samples: &[RadioButtonStateSample],
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .children(samples.iter().map(|sample| render_state_sample(template, kind, variant, sample, window, cx)))
        .into_any_element()
}

fn render_state_sample(
    template: &Arc<dyn ButtonTemplate<bool>>,
    kind: ButtonKind,
    variant: RadioButtonTemplateVariant,
    sample: &RadioButtonStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let selected = variant.selected();
    let id = SharedString::from(format!("radio-button-preview-{}-{}-{}", button_kind_id(kind), variant.id(), sample.id));
    let model = ButtonRenderModel {
        id,
        data: selected,
        content: variant.content(),
        kind,
        role: ButtonFamilyRole::Text,
        size: ButtonSize::Md,
        state: sample.state,
        round: false,
        radius_override: std::cell::Cell::new(None),
    };

    div()
        .w(px(116.0))
        .flex()
        .justify_center()
        .items_center()
        .child(template.render(&model, window, cx))
        .into_any_element()
}

fn button_kind_id(kind: ButtonKind) -> &'static str {
    match kind {
        ButtonKind::Prominent => "prominent",
        ButtonKind::Subtle => "subtle",
        ButtonKind::Standard => "standard",
        ButtonKind::Ghost => "ghost",
    }
}
