use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, IntoElement, Render, SharedString, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::{Button, ButtonEvent, ButtonKind, ButtonRenderModel, ButtonSize, ButtonTemplate};
use gpui_luma::theme::InteractionState;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct ButtonPane {
    default_button: Entity<Button>,
    primary_button: Entity<Button>,
    state_preview: Entity<ButtonStatePreview>,
    default_clicks: usize,
    primary_clicks: usize,
}

impl ButtonPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            default_button: Button::new("button-default-example")
                .label("Default")
                .kind(ButtonKind::Default)
                .template(theme.button_template())
                .spawn(cx),
            primary_button: Button::new("button-primary-example")
                .label("Primary")
                .kind(ButtonKind::Primary)
                .template(theme.button_template())
                .spawn(cx),
            state_preview: cx.new(|_| ButtonStatePreview::new(theme)),
            default_clicks: 0,
            primary_clicks: 0,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.default_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.button.handle_default_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.primary_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.button.handle_primary_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        gallery_pane_with_usage(
            "Command (Text)",
            "Button",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_5()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(12.0))
                        .child(self.default_button.clone())
                        .child(self.primary_button.clone()),
                )
                .child(self.state_preview.clone())
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.default_button, cx);
        notify_entity(&self.primary_button, cx);
        notify_entity(&self.state_preview, cx);
    }

    fn handle_default_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ButtonEvent::Click => {
                self.default_clicks += 1;
                let label = format!("Default {}", self.default_clicks);

                self.default_button.update(cx, |button, cx| {
                    button.set_label(label, cx);
                });
            }
        }
    }

    fn handle_primary_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ButtonEvent::Click => {
                self.primary_clicks += 1;
                let label = format!("Primary {}", self.primary_clicks);

                self.primary_button.update(cx, |button, cx| {
                    button.set_label(label, cx);
                });
            }
        }
    }
}

#[derive(Clone)]
struct ButtonStatePreview {
    theme: GalleryThemePack,
    template: Arc<dyn ButtonTemplate>,
}

struct ButtonStateSample {
    id: &'static str,
    label: &'static str,
    state: InteractionState,
}

impl ButtonStatePreview {
    fn new(theme: &GalleryThemePack) -> Self {
        Self { theme: theme.clone(), template: theme.button_template() }
    }
}

impl Render for ButtonStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.theme.chrome();
        let samples = [
            ButtonStateSample { id: "default", label: "Default", state: InteractionState::default() },
            ButtonStateSample {
                id: "hover",
                label: "Hover",
                state: InteractionState { hovered: true, ..InteractionState::default() },
            },
            ButtonStateSample {
                id: "focus",
                label: "Focus",
                state: InteractionState { focused: true, ..InteractionState::default() },
            },
            ButtonStateSample {
                id: "active",
                label: "Active",
                state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
            },
            ButtonStateSample {
                id: "disabled",
                label: "Disabled",
                state: InteractionState { disabled: true, ..InteractionState::default() },
            },
        ];

        div()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(16.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(chrome.muted_text)
                    .child("Template state preview"),
            )
            .child(render_state_row(
                &self.template,
                "Default variant",
                ButtonKind::Default,
                &samples,
                chrome.muted_text,
                window,
                cx,
            ))
            .child(render_state_row(
                &self.template,
                "Primary variant",
                ButtonKind::Primary,
                &samples,
                chrome.muted_text,
                window,
                cx,
            ))
    }
}

fn render_state_row(
    template: &Arc<dyn ButtonTemplate>,
    row_label: &'static str,
    kind: ButtonKind,
    samples: &[ButtonStateSample],
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
        .child(div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
            samples.iter().map(|sample| render_state_sample(template, kind, sample, label_color, window, cx)),
        ))
        .into_any_element()
}

fn render_state_sample(
    template: &Arc<dyn ButtonTemplate>,
    kind: ButtonKind,
    sample: &ButtonStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("button-preview-{:?}-{}", kind, sample.id));
    let label = SharedString::from("Button");
    let model = ButtonRenderModel { id: &id, label: &label, kind, size: ButtonSize::Md, state: sample.state };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}
