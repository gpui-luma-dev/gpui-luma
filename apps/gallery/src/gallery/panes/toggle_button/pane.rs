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
    default_toggle_button: Entity<ToggleButton>,
    primary_toggle_button: Entity<ToggleButton>,
    state_preview: Entity<ToggleButtonStatePreview>,
    default_selected: bool,
    primary_selected: bool,
}

impl ToggleButtonPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            default_toggle_button: ToggleButton::new("toggle-button-default-example")
                .label("Default")
                .kind(ToggleButtonKind::Default)
                .selected(true)
                .template(theme.toggle_button_template())
                .spawn(cx),
            primary_toggle_button: ToggleButton::new("toggle-button-primary-example")
                .label("Primary")
                .kind(ToggleButtonKind::Primary)
                .selected(true)
                .template(theme.toggle_button_template())
                .spawn(cx),
            state_preview: cx.new(|_| ToggleButtonStatePreview::new(theme)),
            default_selected: true,
            primary_selected: true,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.default_toggle_button, |app, _, event: &ToggleButtonEvent, cx| {
            app.panes.toggle_button.handle_default_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.primary_toggle_button, |app, _, event: &ToggleButtonEvent, cx| {
            app.panes.toggle_button.handle_primary_event(event, cx);
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
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(12.0))
                        .child(self.default_toggle_button.clone())
                        .child(self.primary_toggle_button.clone()),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.body_text)
                                .child(format!("Default selected: {}", self.default_selected)),
                        )
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.body_text)
                                .child(format!("Primary selected: {}", self.primary_selected)),
                        ),
                )
                .child(self.state_preview.clone())
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.default_toggle_button, cx);
        notify_entity(&self.primary_toggle_button, cx);
        notify_entity(&self.state_preview, cx);
    }

    fn handle_default_event(&mut self, event: &ToggleButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ToggleButtonEvent::Change { selected } => {
                self.default_selected = *selected;
                cx.notify();
            }
        }
    }

    fn handle_primary_event(&mut self, event: &ToggleButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ToggleButtonEvent::Change { selected } => {
                self.primary_selected = *selected;
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
            .child(render_variant_preview(
                &self.template,
                "Default Variant",
                ToggleButtonKind::Default,
                &samples,
                chrome.muted_text,
                window,
                cx,
            ))
            .child(render_variant_preview(
                &self.template,
                "Primary Variant",
                ToggleButtonKind::Primary,
                &samples,
                chrome.muted_text,
                window,
                cx,
            ))
    }
}

fn render_variant_preview(
    template: &Arc<dyn ToggleButtonTemplate>,
    title: &'static str,
    kind: ToggleButtonKind,
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
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(title))
        .child(render_state_row(template, "Unselected", kind, false, samples, label_color, window, cx))
        .child(render_state_row(template, "Selected", kind, true, samples, label_color, window, cx))
        .into_any_element()
}

fn render_state_row(
    template: &Arc<dyn ToggleButtonTemplate>,
    row_label: &'static str,
    kind: ToggleButtonKind,
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
                    .map(|sample| render_state_sample(template, kind, selected, sample, label_color, window, cx)),
            ),
        )
        .into_any_element()
}

fn render_state_sample(
    template: &Arc<dyn ToggleButtonTemplate>,
    kind: ToggleButtonKind,
    selected: bool,
    sample: &ToggleButtonStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("toggle-button-preview-{:?}-{}-{}", kind, selected, sample.id));
    let label = SharedString::from("Toggle");
    let model = ToggleButtonRenderModel {
        id: &id,
        label: &label,
        kind,
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
