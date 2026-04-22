use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, IntoElement, Render, SharedString, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::toggle_button::{
    ToggleButton, ToggleButtonEvent, ToggleButtonKind, ToggleButtonRenderModel, ToggleButtonSize, ToggleButtonTemplate,
};
use gpui_luma::theme::InteractionState;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct ToggleButtonPane {
    toggle_button: Entity<ToggleButton>,
    state_preview: Entity<ToggleButtonStatePreview>,
    selected: bool,
}

impl ToggleButtonPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            toggle_button: ToggleButton::new("toggle-button-example")
                .label("Toggle")
                .selected(true)
                .template(theme.toggle_button_template())
                .spawn(cx),
            state_preview: cx.new(|_| ToggleButtonStatePreview::new(theme)),
            selected: true,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.toggle_button, |app, _, event: &ToggleButtonEvent, cx| {
            app.panes.toggle_button.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        gallery_pane_with_usage(
            "Toggle Button",
            "Toggle Button",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_5()
                .child(self.toggle_button.clone())
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
        notify_entity(&self.toggle_button, cx);
        notify_entity(&self.state_preview, cx);
    }

    fn handle_event(&mut self, event: &ToggleButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ToggleButtonEvent::Change { selected } => {
                self.selected = *selected;
                cx.notify();
            }
        }
    }
}

#[derive(Clone)]
struct ToggleButtonStatePreview {
    theme: GalleryThemePack,
    template: Arc<dyn ToggleButtonTemplate>,
}

struct ToggleButtonStateSample {
    id: &'static str,
    label: &'static str,
    state: InteractionState,
}

impl ToggleButtonStatePreview {
    fn new(theme: &GalleryThemePack) -> Self {
        Self { theme: theme.clone(), template: theme.toggle_button_template() }
    }
}

impl Render for ToggleButtonStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.theme.chrome();
        let samples = [
            ToggleButtonStateSample { id: "default", label: "Default", state: InteractionState::default() },
            ToggleButtonStateSample {
                id: "hover",
                label: "Hover",
                state: InteractionState { hovered: true, ..InteractionState::default() },
            },
            ToggleButtonStateSample {
                id: "focus",
                label: "Focus",
                state: InteractionState { focused: true, ..InteractionState::default() },
            },
            ToggleButtonStateSample {
                id: "active",
                label: "Active",
                state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
            },
            ToggleButtonStateSample {
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
    template: &Arc<dyn ToggleButtonTemplate>,
    row_label: &'static str,
    selected: bool,
    samples: &[ToggleButtonStateSample],
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
    template: &Arc<dyn ToggleButtonTemplate>,
    selected: bool,
    sample: &ToggleButtonStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("toggle-button-preview-{}-{}", selected, sample.id));
    let label = SharedString::from("Toggle");
    let model = ToggleButtonRenderModel {
        id: &id,
        label: &label,
        kind: ToggleButtonKind::Default,
        size: ToggleButtonSize::Md,
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
