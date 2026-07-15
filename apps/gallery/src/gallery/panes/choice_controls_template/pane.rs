#![allow(clippy::too_many_arguments)]

use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, FontWeight, IntoElement, Render, SharedString, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::{ButtonRenderModel, ButtonTemplate, ControlPresenter};
use gpui_luma::controls::button_family::{ButtonFamilyRole, ButtonSize};
use gpui_luma::theme::{InteractionState};
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use super::super::button::labeling::render_vertical_section_rail;
use super::super::shared::{gallery_pane_with_description, notify_entity};

const CHOICE_TEMPLATES_DESCRIPTION: &str = concat!(
    "Template matrix for choice controls. ",
    "Rows are interaction states; columns are control templates."
);

#[derive(Clone)]
pub(in crate::gallery) struct ChoiceControlsTemplatePane {
    state_preview: Entity<ChoiceControlsTemplatePreview>,
}

impl ChoiceControlsTemplatePane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        Self { state_preview: cx.new(|_| ChoiceControlsTemplatePreview::new(look)) }
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        gallery_pane_with_description(
            "Choice Templates",
            Some(CHOICE_TEMPLATES_DESCRIPTION),
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(16.0))
                .child(self.state_preview.clone())
                .into_any_element(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.state_preview, cx);
    }
}

#[derive(Clone)]
struct ChoiceControlsTemplatePreview {
    look: Arc<ShadcnLook>,
    radio_template: Arc<dyn ButtonTemplate<bool>>,
    checkbox_template: Arc<dyn ButtonTemplate<bool>>,
    switch_template: Arc<dyn ButtonTemplate<bool>>,
    toggle_template: Arc<dyn ButtonTemplate<bool>>,
}

#[derive(Clone, Copy)]
struct ChoiceTemplateStateSample {
    id: &'static str,
    label: &'static str,
    state: InteractionState,
}

#[derive(Clone, Copy)]
enum ChoiceTemplateControl {
    Radio,
    Checkbox,
    Switch,
    Toggle,
    ToggleIcon,
}

impl ChoiceTemplateControl {
    fn id(self) -> &'static str {
        match self {
            Self::Radio => "radio",
            Self::Checkbox => "checkbox",
            Self::Switch => "switch",
            Self::Toggle => "toggle",
            Self::ToggleIcon => "toggle-icon",
        }
    }

    fn header(self) -> &'static str {
        match self {
            Self::Radio => "Radio",
            Self::Checkbox => "Checkbox",
            Self::Switch => "Switch",
            Self::Toggle => "Toggle",
            Self::ToggleIcon => "Toggle Icon",
        }
    }

    fn content(self, selected: bool) -> ControlPresenter<ButtonRenderModel<bool>> {
        match self {
            Self::Radio => Arc::new(move |_, _| div().child("Radio").into_any_element()),
            Self::Checkbox => Arc::new(move |_, _| div().child("Checkbox").into_any_element()),
            Self::Switch => Arc::new(move |_, _| div().into_any_element()),
            Self::Toggle => Arc::new(move |_, _| div().child("Toggle").into_any_element()),
            Self::ToggleIcon => {
                let icon = if selected { LucideIcon::Check } else { LucideIcon::Plus };
                Arc::new(move |_, _| {
                    div()
                        .font_family("lucide")
                        .text_size(px(16.0))
                        .line_height(px(16.0))
                        .child(char::from(icon).to_string())
                        .into_any_element()
                })
            }
        }
    }

    fn round(self) -> bool {
        matches!(self, Self::ToggleIcon)
    }

    fn template(self, preview: &ChoiceControlsTemplatePreview) -> &Arc<dyn ButtonTemplate<bool>> {
        match self {
            Self::Radio => &preview.radio_template,
            Self::Checkbox => &preview.checkbox_template,
            Self::Switch => &preview.switch_template,
            Self::Toggle | Self::ToggleIcon => &preview.toggle_template,
        }
    }
}

impl ChoiceControlsTemplatePreview {
    fn new(look: Arc<ShadcnLook>) -> Self {
        Self {
            radio_template: look.radio_button_template(ShadcnButtonStyle::Primary),
            checkbox_template: look.checkbox_template(ShadcnButtonStyle::Primary),
            switch_template: look.switch_template(ShadcnButtonStyle::Primary),
            toggle_template: look.toggle_template(ShadcnButtonStyle::Secondary),
            look,
        }
    }
}

impl Render for ChoiceControlsTemplatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let states = [
            ChoiceTemplateStateSample { id: "default", label: "Default", state: InteractionState::default() },
            ChoiceTemplateStateSample {
                id: "hover",
                label: "Hover",
                state: InteractionState { hovered: true, ..InteractionState::default() },
            },
            ChoiceTemplateStateSample {
                id: "focused",
                label: "Focused",
                state: InteractionState { focused: true, ..InteractionState::default() },
            },
            ChoiceTemplateStateSample {
                id: "pressed",
                label: "Pressed",
                state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
            },
            ChoiceTemplateStateSample {
                id: "disabled",
                label: "Disabled",
                state: InteractionState { disabled: true, ..InteractionState::default() },
            },
        ];
        let controls = [
            ChoiceTemplateControl::Radio,
            ChoiceTemplateControl::Checkbox,
            ChoiceTemplateControl::Switch,
            ChoiceTemplateControl::Toggle,
            ChoiceTemplateControl::ToggleIcon,
        ];

        div()
            .flex()
            .flex_col()
            .items_start()
            .gap(px(14.0))
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(chrome.muted_text)
                    .child("Template matrix preview"),
            )
            .child(render_matrix(self, "Selected", true, &states, &controls, chrome.muted_text, window, cx))
            .child(render_matrix(self, "Unselected", false, &states, &controls, chrome.muted_text, window, cx))
    }
}

fn render_matrix(
    preview: &ChoiceControlsTemplatePreview,
    section_label: &'static str,
    selected: bool,
    states: &[ChoiceTemplateStateSample],
    controls: &[ChoiceTemplateControl],
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .items_stretch()
        .gap(px(8.0))
        .child(render_vertical_section_rail(section_label, label_color))
        .child(
            div()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(8.0))
                .child(render_header_row(controls, label_color))
                .children(
                    states
                        .iter()
                        .map(|state| render_state_row(preview, state, selected, controls, label_color, window, cx)),
                ),
        )
        .into_any_element()
}

fn render_header_row(controls: &[ChoiceTemplateControl], label_color: gpui::Hsla) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(div().w(px(96.0)))
        .children(controls.iter().map(|control| {
            div()
                .w(px(152.0))
                .flex()
                .justify_center()
                .text_xs()
                .line_height(px(15.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(label_color)
                .child(control.header())
        }))
        .into_any_element()
}

fn render_state_row(
    preview: &ChoiceControlsTemplatePreview,
    state_sample: &ChoiceTemplateStateSample,
    selected: bool,
    controls: &[ChoiceTemplateControl],
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(
            div()
                .w(px(96.0))
                .text_xs()
                .line_height(px(15.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(label_color)
                .child(state_sample.label),
        )
        .children(
            controls
                .iter()
                .map(|control| render_control_cell(preview, *control, selected, state_sample, window, cx)),
        )
        .into_any_element()
}

fn render_control_cell(
    preview: &ChoiceControlsTemplatePreview,
    control: ChoiceTemplateControl,
    selected: bool,
    state_sample: &ChoiceTemplateStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("choice-template-preview-{}-{}", state_sample.id, control.id()));
    let model = ButtonRenderModel {
        id,
        data: selected,
        content: control.content(selected),
        role: ButtonFamilyRole::Text,
        size: ButtonSize::Md,
        state: state_sample.state,
        round: control.round(),
        radius_override: std::cell::Cell::new(None),
        elevation: true,
        compact: false,
        look: None,
        ..Default::default()
    };

    div()
        .w(px(152.0))
        .flex()
        .items_center()
        .justify_center()
        .child(control.template(preview).render(&model, window, cx))
        .into_any_element()
}
