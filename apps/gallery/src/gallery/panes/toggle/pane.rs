use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, IntoElement, Render, SharedString, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::toggle::{Toggle, ToggleEvent, ToggleKind, ToggleRenderModel, ToggleSize, ToggleTemplate};
use gpui_luma::theme::InteractionState;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct TogglePane {
    toggle: Entity<Toggle>,
    state_preview: Entity<ToggleStatePreview>,
    selected: bool,
}

impl TogglePane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            toggle: Toggle::new("toggle-example")
                .label("Toggle")
                .selected(true)
                .template(theme.toggle_template())
                .spawn(cx),
            state_preview: cx.new(|_| ToggleStatePreview::new(theme)),
            selected: true,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.toggle, |app, _, event: &ToggleEvent, cx| {
            app.panes.toggle.handle_toggle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        gallery_pane_with_usage(
            "Toggle",
            "Toggle",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_5()
                .child(div().flex().items_center().gap(px(12.0)).child(self.toggle.clone()))
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
        notify_entity(&self.toggle, cx);
        notify_entity(&self.state_preview, cx);
    }

    fn handle_toggle_event(&mut self, event: &ToggleEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ToggleEvent::Change { selected } => {
                self.selected = *selected;
                cx.notify();
            }
        }
    }
}

#[derive(Clone)]
struct ToggleStatePreview {
    theme: GalleryThemePack,
    template: Arc<dyn ToggleTemplate>,
}

struct ToggleStateSample {
    id: &'static str,
    label: &'static str,
    state: InteractionState,
}

impl ToggleStatePreview {
    fn new(theme: &GalleryThemePack) -> Self {
        Self { theme: theme.clone(), template: theme.toggle_template() }
    }
}

impl Render for ToggleStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.theme.chrome();
        let samples = [
            ToggleStateSample { id: "default", label: "Default", state: InteractionState::default() },
            ToggleStateSample {
                id: "hover",
                label: "Hover",
                state: InteractionState { hovered: true, ..InteractionState::default() },
            },
            ToggleStateSample {
                id: "focus",
                label: "Focus",
                state: InteractionState { focused: true, ..InteractionState::default() },
            },
            ToggleStateSample {
                id: "active",
                label: "Active",
                state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
            },
            ToggleStateSample {
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
    template: &Arc<dyn ToggleTemplate>,
    row_label: &'static str,
    selected: bool,
    samples: &[ToggleStateSample],
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
    template: &Arc<dyn ToggleTemplate>,
    selected: bool,
    sample: &ToggleStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("toggle-preview-{}-{}", selected, sample.id));
    let label = SharedString::from("Toggle");
    let model = ToggleRenderModel {
        id: &id,
        label: &label,
        kind: ToggleKind::Default,
        size: ToggleSize::Md,
        enabled: !sample.state.disabled,
        selected,
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
