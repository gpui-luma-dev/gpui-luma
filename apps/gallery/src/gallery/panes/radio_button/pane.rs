use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, IntoElement, Render, SharedString, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::button_family::{ButtonKind, ButtonSize};
use gpui_luma::controls::command::button::{ButtonEvent, ButtonRenderModel, ButtonTemplate};
use gpui_luma::controls::content_presenter::HasContent;
use gpui_luma::controls::radio_button::{self, RadioButton};
use gpui_luma::theme::InteractionState;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct RadioButtonPane {
    radio_button: RadioButton,
    state_preview: Entity<RadioButtonStatePreview>,
    selected: bool,
}

impl RadioButtonPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            radio_button: radio_button::new("radio-button-example")
                .data(false)
                .content(|_, _| div().child("Standalone").into_any_element())
                .template(theme.radio_button_template())
                .spawn(cx),
            state_preview: cx.new(|_| RadioButtonStatePreview::new(theme)),
            selected: false,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.radio_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.radio_button.handle_event(event, cx);
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
                .child(self.radio_button.clone())
                .child(
                    div()
                        .text_size(px(12.0))
                        .line_height(px(16.0))
                        .text_color(chrome.body_text)
                        .child(format!("Selected: {}", self.selected)),
                )
                .child(self.state_preview.clone())
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.radio_button, cx);
        notify_entity(&self.state_preview, cx);
    }

    fn handle_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ButtonEvent::Click => {
                self.radio_button.update(cx, |button, cx| {
                    let new_selected = !*button.data();
                    button.set_data(new_selected, cx);
                    self.selected = new_selected;
                });
                cx.notify();
            }
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
    label: &'static str,
    state: InteractionState,
}

impl RadioButtonStatePreview {
    fn new(theme: &GalleryThemePack) -> Self {
        Self { theme: theme.clone(), template: theme.radio_button_template() }
    }
}

impl Render for RadioButtonStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.theme.chrome();
        let samples = [
            RadioButtonStateSample { id: "default", label: "Standard", state: InteractionState::default() },
            RadioButtonStateSample {
                id: "hover",
                label: "Hover",
                state: InteractionState { hovered: true, ..InteractionState::default() },
            },
            RadioButtonStateSample {
                id: "focus",
                label: "Focus",
                state: InteractionState { focused: true, ..InteractionState::default() },
            },
            RadioButtonStateSample {
                id: "active",
                label: "Active",
                state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
            },
            RadioButtonStateSample {
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
            .child(render_state_row(&self.template, "Unselected", false, &samples, chrome.muted_text, window, cx))
            .child(render_state_row(&self.template, "Selected", true, &samples, chrome.muted_text, window, cx))
    }
}

fn render_state_row(
    template: &Arc<dyn ButtonTemplate<bool>>,
    row_label: &'static str,
    selected: bool,
    samples: &[RadioButtonStateSample],
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
            div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
                samples
                    .iter()
                    .map(|sample| render_state_sample(template, selected, sample, label_color, window, cx)),
            ),
        )
        .into_any_element()
}

fn render_state_sample(
    template: &Arc<dyn ButtonTemplate<bool>>,
    selected: bool,
    sample: &RadioButtonStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("radio-button-preview-{}-{}", selected, sample.id));
    let label = SharedString::from("Radio");
    let model = ButtonRenderModel {
        id,
        data: selected,
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
