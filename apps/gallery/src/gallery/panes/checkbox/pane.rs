use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, IntoElement, Render, SharedString, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::prototypes::mod_button::{Button, ButtonEvent, ButtonRenderModel, ButtonTemplate};
use gpui_luma::controls::content_presenter::HasContent;
use gpui_luma::controls::button_family::{ButtonKind, ButtonSize};
use gpui_luma::theme::InteractionState;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct CheckboxPane {
    default_checkbox: Entity<Button<bool>>,
    state_preview: Entity<CheckboxStatePreview>,
    default_checked: bool,
}

impl CheckboxPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            default_checkbox: Button::new("checkbox-default")
                .data(true)
                .content(|_, _| div().child("As-is").into_any_element())
                .template(theme.checkbox_template())
                .spawn(cx),
            state_preview: cx.new(|_| CheckboxStatePreview::new(theme)),
            default_checked: true,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.default_checkbox, |app, _, event: &ButtonEvent, cx| {
            app.panes.checkbox.handle_event(event, cx);
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
                )
                .child(
                    div()
                        .text_size(px(12.0))
                        .line_height(px(16.0))
                        .text_color(chrome.body_text)
                        .child(format!("Checked: as-is={}", self.default_checked)),
                )
                .child(self.state_preview.clone())
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.default_checkbox, cx);
        notify_entity(&self.state_preview, cx);
    }

    fn handle_event(
        &mut self,
        event: &ButtonEvent,
        cx: &mut Context<GalleryApp>,
    ) {
        match event {
            ButtonEvent::Click => {
                self.default_checkbox.update(cx, |button, cx| {
                    let new_checked = !*button.data();
                    button.set_data(new_checked, cx);
                    self.default_checked = new_checked;
                });
                cx.notify();
            }
        }
    }
}

#[derive(Clone)]
struct CheckboxStatePreview {
    theme: GalleryThemePack,
    default_template: Arc<dyn ButtonTemplate<bool>>,
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
    }
}

fn render_presentation(
    template: &Arc<dyn ButtonTemplate<bool>>,
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
    template: &Arc<dyn ButtonTemplate<bool>>,
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
    template: &Arc<dyn ButtonTemplate<bool>>,
    presentation_id: &'static str,
    checked: bool,
    sample: &CheckboxStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("checkbox-preview-{}-{}-{}", presentation_id, checked, sample.id));
    let label = SharedString::from("Checkbox");
    let model = ButtonRenderModel {
        id,
        data: checked,
        content: Arc::new(move |_, _| div().child(label.clone()).into_any_element()),
        kind: ButtonKind::Standard,
        size: ButtonSize::Md,
        state: sample.state,
        round: false,
        radius_override: std::cell::Cell::new(None),
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}
