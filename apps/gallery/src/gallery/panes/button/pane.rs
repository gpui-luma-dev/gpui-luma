use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, IntoElement, Render, SharedString, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::button::{Button, ButtonEvent, ButtonKind, ButtonRenderModel, ButtonSize, ButtonTemplate};
use gpui_luma::theme::InteractionState;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct ButtonPane {
    button: Entity<Button>,
    state_preview: Entity<ButtonStatePreview>,
    clicks: usize,
}

impl ButtonPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            button: Button::new("button-example")
                .label("Click me")
                .kind(ButtonKind::Primary)
                .template(theme.button_template())
                .spawn(cx),
            state_preview: cx.new(|_| ButtonStatePreview::new(theme)),
            clicks: 0,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.button, |app, _, event: &ButtonEvent, cx| {
            app.panes.button.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        gallery_pane_with_usage(
            "Button",
            "Button",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_5()
                .child(self.button.clone())
                .child(self.state_preview.clone())
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.button, cx);
        notify_entity(&self.state_preview, cx);
    }

    fn handle_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ButtonEvent::Click => {
                self.clicks += 1;
                let label = format!("Clicked {}", self.clicks);

                self.button.update(cx, |button, cx| {
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
            .child(
                div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
                    samples
                        .into_iter()
                        .map(|sample| render_state_sample(&self.template, sample, chrome.muted_text, window, cx)),
                ),
            )
    }
}

fn render_state_sample(
    template: &Arc<dyn ButtonTemplate>,
    sample: ButtonStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("button-preview-{}", sample.id));
    let label = SharedString::from("Button");
    let model = ButtonRenderModel {
        id: &id,
        label: &label,
        kind: ButtonKind::Primary,
        size: ButtonSize::Md,
        state: sample.state,
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
