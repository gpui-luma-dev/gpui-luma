use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, IntoElement, Render, SharedString, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::checkbox::{Checkbox, CheckboxEvent, CheckboxRenderModel, CheckboxTemplate};
use gpui_luma::theme::InteractionState;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct CheckboxPane {
    default_checkbox: Entity<Checkbox>,
    border_checkbox: Entity<Checkbox>,
    state_preview: Entity<CheckboxStatePreview>,
    default_checked: bool,
    border_checked: bool,
}

#[derive(Clone, Copy)]
enum CheckboxPresentation {
    Default,
    Border,
}

impl CheckboxPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            default_checkbox: Checkbox::new("checkbox-default")
                .label("As-is")
                .checked(true)
                .template(theme.checkbox_template())
                .spawn(cx),
            border_checkbox: Checkbox::new("checkbox-border")
                .label("Border")
                .template(theme.border_checkbox_template())
                .spawn(cx),
            state_preview: cx.new(|_| CheckboxStatePreview::new(theme)),
            default_checked: true,
            border_checked: false,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.default_checkbox, |app, _, event: &CheckboxEvent, cx| {
            app.panes.checkbox.handle_event(CheckboxPresentation::Default, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.border_checkbox, |app, _, event: &CheckboxEvent, cx| {
            app.panes.checkbox.handle_event(CheckboxPresentation::Border, event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        gallery_pane_with_usage(
            "Checkbox",
            "Checkbox",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_5()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(self.default_checkbox.clone())
                        .child(self.border_checkbox.clone()),
                )
                .child(
                    div()
                        .text_size(px(12.0))
                        .line_height(px(16.0))
                        .text_color(chrome.body_text)
                        .child(format!("Checked: as-is={}, border={}", self.default_checked, self.border_checked)),
                )
                .child(self.state_preview.clone())
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.default_checkbox, cx);
        notify_entity(&self.border_checkbox, cx);
        notify_entity(&self.state_preview, cx);
    }

    fn handle_event(
        &mut self,
        presentation: CheckboxPresentation,
        event: &CheckboxEvent,
        cx: &mut Context<GalleryApp>,
    ) {
        match event {
            CheckboxEvent::Change { checked } => {
                match presentation {
                    CheckboxPresentation::Default => {
                        self.default_checked = *checked;
                    }
                    CheckboxPresentation::Border => {
                        self.border_checked = *checked;
                    }
                }
                cx.notify();
            }
        }
    }
}

#[derive(Clone)]
struct CheckboxStatePreview {
    theme: GalleryThemePack,
    default_template: Arc<dyn CheckboxTemplate>,
    border_template: Arc<dyn CheckboxTemplate>,
}

struct CheckboxStateSample {
    id: &'static str,
    label: &'static str,
    state: InteractionState,
}

impl CheckboxStatePreview {
    fn new(theme: &GalleryThemePack) -> Self {
        Self {
            theme: theme.clone(),
            default_template: theme.checkbox_template(),
            border_template: theme.border_checkbox_template(),
        }
    }
}

impl Render for CheckboxStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.theme.chrome();
        let samples = [
            CheckboxStateSample { id: "default", label: "Standard", state: InteractionState::default() },
            CheckboxStateSample {
                id: "hover",
                label: "Hover",
                state: InteractionState { hovered: true, ..InteractionState::default() },
            },
            CheckboxStateSample {
                id: "focus",
                label: "Focus",
                state: InteractionState { focused: true, ..InteractionState::default() },
            },
            CheckboxStateSample {
                id: "active",
                label: "Active",
                state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
            },
            CheckboxStateSample {
                id: "disabled",
                label: "Disabled",
                state: InteractionState { disabled: true, ..InteractionState::default() },
            },
        ];

        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(14.0))
            .child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(16.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(chrome.muted_text)
                    .child("Template state preview"),
            )
            .child(render_presentation(
                &self.default_template,
                "default",
                "Standard",
                &samples,
                chrome.muted_text,
                window,
                cx,
            ))
            .child(render_presentation(
                &self.border_template,
                "border",
                "Border",
                &samples,
                chrome.muted_text,
                window,
                cx,
            ))
    }
}

fn render_presentation(
    template: &Arc<dyn CheckboxTemplate>,
    presentation_id: &'static str,
    presentation_label: &'static str,
    samples: &[CheckboxStateSample],
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(8.0))
        .child(
            div()
                .text_size(px(11.0))
                .line_height(px(15.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(label_color)
                .child(presentation_label),
        )
        .child(render_state_row(template, presentation_id, "Unchecked", false, samples, label_color, window, cx))
        .child(render_state_row(template, presentation_id, "Checked", true, samples, label_color, window, cx))
        .into_any_element()
}

fn render_state_row(
    template: &Arc<dyn CheckboxTemplate>,
    presentation_id: &'static str,
    row_label: &'static str,
    checked: bool,
    samples: &[CheckboxStateSample],
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(8.0))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(row_label))
        .child(
            div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(samples.iter().map(
                |sample| render_state_sample(template, presentation_id, checked, sample, label_color, window, cx),
            )),
        )
        .into_any_element()
}

fn render_state_sample(
    template: &Arc<dyn CheckboxTemplate>,
    presentation_id: &'static str,
    checked: bool,
    sample: &CheckboxStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("checkbox-preview-{}-{}-{}", presentation_id, checked, sample.id));
    let label = SharedString::from("Checkbox");
    let model =
        CheckboxRenderModel { id, label, checked, enabled: !sample.state.disabled, state: sample.state };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}
