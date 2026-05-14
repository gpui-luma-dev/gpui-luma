use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, IntoElement, Render, SharedString, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::{ButtonEvent, ButtonRenderModel, ButtonTemplate};
use gpui_luma::controls::switch::{self, Switch};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::button_family::{ButtonKind, ButtonSize};
use gpui_luma::controls::button_family::ButtonFamilyRole;
use gpui_luma::theme::InteractionState;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct SwitchPane {
    switch: Switch,
    state_preview: Entity<SwitchStatePreview>,
    on: bool,
}

impl SwitchPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            switch: switch::new("switch-example")
                .data(true)
                .content(|_, _| div().into_any_element())
                .template(theme.switch_template())
                .spawn(cx),
            state_preview: cx.new(|_| SwitchStatePreview::new(theme)),
            on: true,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.switch, |app, _, event: &ButtonEvent, cx| {
            app.panes.switch.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        gallery_pane_with_usage(
            "Switch",
            "Switch",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_5()
                .child(self.switch.clone())
                .child(
                    div()
                        .text_size(px(12.0))
                        .line_height(px(16.0))
                        .text_color(chrome.body_text)
                        .child(format!("On: {}", self.on)),
                )
                .child(self.state_preview.clone())
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.switch, cx);
        notify_entity(&self.state_preview, cx);
    }

    fn handle_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ButtonEvent::Click => {
                self.switch.update(cx, |button, cx| {
                    let new_on = !*button.data();
                    button.set_data(new_on, cx);
                    self.on = new_on;
                });
                cx.notify();
            }
        }
    }
}

#[derive(Clone)]
struct SwitchStatePreview {
    theme: GalleryThemePack,
    template: Arc<dyn ButtonTemplate<bool>>,
}

struct SwitchStateSample {
    id: &'static str,
    label: &'static str,
    state: InteractionState,
}

impl SwitchStatePreview {
    fn new(theme: &GalleryThemePack) -> Self {
        Self { theme: theme.clone(), template: theme.switch_template() }
    }
}

impl Render for SwitchStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.theme.chrome();
        let samples = [
            SwitchStateSample { id: "default", label: "Standard", state: InteractionState::default() },
            SwitchStateSample {
                id: "hover",
                label: "Hover",
                state: InteractionState { hovered: true, ..InteractionState::default() },
            },
            SwitchStateSample {
                id: "focus",
                label: "Focus",
                state: InteractionState { focused: true, ..InteractionState::default() },
            },
            SwitchStateSample {
                id: "pressed",
                label: "Pressed",
                state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
            },
            SwitchStateSample {
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
            .child(render_state_row(&self.template, "Off", false, &samples, chrome.muted_text, window, cx))
            .child(render_state_row(&self.template, "On", true, &samples, chrome.muted_text, window, cx))
    }
}

fn render_state_row(
    template: &Arc<dyn ButtonTemplate<bool>>,
    row_label: &'static str,
    on: bool,
    samples: &[SwitchStateSample],
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
                samples.iter().map(|sample| render_state_sample(template, on, sample, label_color, window, cx)),
            ),
        )
        .into_any_element()
}

fn render_state_sample(
    template: &Arc<dyn ButtonTemplate<bool>>,
    on: bool,
    sample: &SwitchStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("switch-preview-{}-{}", on, sample.id));
    let model = ButtonRenderModel {
        id,
        data: on,
        content: Arc::new(move |_, _| div().into_any_element()),
        kind: ButtonKind::Standard,
        role: ButtonFamilyRole::Text,
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
