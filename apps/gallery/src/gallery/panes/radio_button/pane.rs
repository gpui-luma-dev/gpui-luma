#![allow(clippy::too_many_arguments)]

use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Entity, FontWeight, IntoElement, Render, SharedString, Subscription, Window, div,
    prelude::*, px,
};
use gpui_luma::controls::button_family::{ButtonFamilyRole, ButtonSize};
use gpui_luma::controls::command::button::{ButtonRenderModel, ButtonTemplate, HasPresenter};
use gpui_luma::controls::radio_button::{RadioButton, RadioButtonEvent};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma::theme::{InteractionState};
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};

use crate::gallery::control::GalleryApp;

use super::inspector_tree::build_radio_button_inspect_tree;
use super::super::button::labeling::render_vertical_section_rail;
use super::super::shared::inspector::{ColorInspectorShell, spawn_color_inspector_tree};
use super::super::shared::{gallery_pane_with_inspector, notify_entity, InspectorToggleRegistry};

type ChoiceContentRenderer = dyn Fn(&ButtonRenderModel<bool>, &mut App) -> AnyElement + Send + Sync;

#[derive(Clone)]
pub(in crate::gallery) struct RadioButtonPane {
    secondary_radio: RadioButton,
    primary_radio: RadioButton,
    state_preview: Entity<RadioButtonStatePreview>,
    inspector: Entity<ColorInspectorShell>,
    secondary_selected: bool,
    primary_selected: bool,
}

impl RadioButtonPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let tree = spawn_color_inspector_tree(
            "radio-button-inspector-tree",
            look.clone(),
            build_radio_button_inspect_tree,
            cx,
        );
        let inspector = cx.new(|cx| {
            ColorInspectorShell::new(
                look.clone(),
                tree,
                "radio-button-inspector",
                "radio-button-inspector-split",
                "radio-button-inspector-detail",
                build_radio_button_inspect_tree,
                cx,
            )
        });

        Self {
            secondary_radio: look
                .secondary_radio("radio-button-secondary-example")
                .with_data(true)
                .content(|_, _| div().child("Secondary").into_any_element())
                .spawn(cx),
            primary_radio: look
                .primary_radio("radio-button-primary-example")
                .with_data(false)
                .content(|_, _| div().child("Primary").into_any_element())
                .spawn(cx),
            state_preview: cx.new(|_| RadioButtonStatePreview::new(look)),
            inspector,
            secondary_selected: true,
            primary_selected: false,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.secondary_radio, |app, _, event: &RadioButtonEvent, cx| {
            app.panes.radio_button.handle_secondary_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.primary_radio, |app, _, event: &RadioButtonEvent, cx| {
            app.panes.radio_button.handle_primary_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook, toggles: &InspectorToggleRegistry) -> AnyElement {
        let chrome = look.chrome();

        gallery_pane_with_inspector(
            "radio-button",
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
            self.inspector.clone(),
            toggles,
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.secondary_radio, cx);
        notify_entity(&self.primary_radio, cx);
        notify_entity(&self.state_preview, cx);
        notify_entity(&self.inspector, cx);
        notify_entity(&self.inspector.read(cx).tree(), cx);
        notify_entity(&self.inspector.read(cx).detail(), cx);
        notify_entity(&self.inspector.read(cx).split(), cx);
    }

    fn handle_secondary_event(&mut self, event: &RadioButtonEvent, cx: &mut Context<GalleryApp>) {
        if let RadioButtonEvent::Change { selected } = event {
            self.secondary_selected = *selected;
            cx.notify();
        }
    }

    fn handle_primary_event(&mut self, event: &RadioButtonEvent, cx: &mut Context<GalleryApp>) {
        if let RadioButtonEvent::Change { selected } = event {
            self.primary_selected = *selected;
            cx.notify();
        }
    }
}

#[derive(Clone)]
struct RadioButtonStatePreview {
    look: Arc<ShadcnLook>,
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

    fn content(self) -> Arc<ChoiceContentRenderer> {
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
    fn new(look: Arc<ShadcnLook>) -> Self {
        Self { look }
    }
}

impl Render for RadioButtonStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
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
    variants: &[RadioButtonTemplateVariant],
    samples: &[RadioButtonStateSample],
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let template = look.radio_button_template(style);
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
    style: ShadcnButtonStyle,
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
    style: ShadcnButtonStyle,
    variant: RadioButtonTemplateVariant,
    sample: &RadioButtonStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let selected = variant.selected();
    let id =
        SharedString::from(format!("radio-button-preview-{}-{}-{}", shadcn_style_id(style), variant.id(), sample.id));
    let model = ButtonRenderModel {
        id,
        data: selected,
        content: variant.content(),
        role: ButtonFamilyRole::Text,
        size: ButtonSize::Md,
        state: sample.state,
        round: false,
        radius_override: std::cell::Cell::new(None),
        elevation: true,
        compact: false,
        look: None,
        ..Default::default()
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
        ShadcnButtonStyle::ContentOnly => "content-only",
    }
}
