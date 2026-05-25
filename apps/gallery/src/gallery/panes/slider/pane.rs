use std::sync::Arc;

use gpui::{
    AnyElement, App, Bounds, Context, DragMoveEvent, Entity, IntoElement, MouseDownEvent, MouseUpEvent, Pixels, Render,
    SharedString, Subscription, Window, div, prelude::*, px,
};
use gpui_luma::controls::slider::{
    self, Slider, SliderBoundsHandler, SliderDrag, SliderDragMoveHandler, SliderEvent, SliderHoverHandler,
    SliderMouseDownHandler, SliderMouseUpHandler, SliderRenderModel, SliderTemplate, SliderTemplateHandlers,
    default_slider_template,
};
use gpui_luma::controls::value::ControlRange;
use gpui_luma::theme::InteractionState;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct SliderPane {
    slider: Slider,
    state_preview: Entity<SliderStatePreview>,
    value: f32,
}

impl SliderPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            slider: slider::new("slider-example").range(1..100).step(10).value(41).spawn(cx),
            state_preview: cx.new(|_| SliderStatePreview::new(theme)),
            value: 41.0,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.slider, |app, _, event: &SliderEvent, cx| {
            app.panes.slider.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        gallery_pane_with_usage(
            "Slider",
            "Slider",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_5()
                .child(self.slider.clone())
                .child(
                    div()
                        .text_size(px(12.0))
                        .line_height(px(16.0))
                        .text_color(chrome.body_text)
                        .child(format!("Value: {:.0}", self.value)),
                )
                .child(self.state_preview.clone())
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.slider, cx);
        notify_entity(&self.state_preview, cx);
    }

    fn handle_event(&mut self, event: &SliderEvent, cx: &mut Context<GalleryApp>) {
        match event {
            SliderEvent::Change { value } => {
                self.value = *value;
                cx.notify();
            }
        }
    }
}

#[derive(Clone)]
struct SliderStatePreview {
    theme: GalleryThemePack,
    template: Arc<dyn SliderTemplate>,
}

struct SliderStateSample {
    id: &'static str,
    label: &'static str,
    state: InteractionState,
}

impl SliderStatePreview {
    fn new(theme: &GalleryThemePack) -> Self {
        Self { theme: theme.clone(), template: default_slider_template() }
    }
}

impl Render for SliderStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.theme.chrome();
        let samples = [
            SliderStateSample { id: "default", label: "Standard", state: InteractionState::default() },
            SliderStateSample {
                id: "hover",
                label: "Hover",
                state: InteractionState { hovered: true, ..InteractionState::default() },
            },
            SliderStateSample {
                id: "focus",
                label: "Focus",
                state: InteractionState { focused: true, ..InteractionState::default() },
            },
            SliderStateSample {
                id: "active",
                label: "Active",
                state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
            },
            SliderStateSample {
                id: "disabled",
                label: "Disabled",
                state: InteractionState { disabled: true, ..InteractionState::default() },
            },
        ];

        div()
            .flex()
            .flex_col()
            .items_center()
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
    template: &Arc<dyn SliderTemplate>,
    sample: SliderStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("slider-preview-{}", sample.id));
    let range = ControlRange::from(1..100);
    let value = 41.0;
    let model = SliderRenderModel {
        id: &id,
        range,
        step: 10.0,
        value,
        percentage: range.percentage(value),
        enabled: !sample.state.disabled,
        state: sample.state,
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, slider_preview_handlers(), window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}

fn slider_preview_handlers() -> SliderTemplateHandlers {
    SliderTemplateHandlers {
        track_bounds: Box::new(noop_bounds) as SliderBoundsHandler,
        hover: Box::new(noop_hover) as SliderHoverHandler,
        mouse_down: Box::new(noop_mouse_down) as SliderMouseDownHandler,
        mouse_up: Box::new(noop_mouse_up) as SliderMouseUpHandler,
        mouse_up_out: Box::new(noop_mouse_up) as SliderMouseUpHandler,
        drag_move: Box::new(noop_drag_move) as SliderDragMoveHandler,
    }
}

fn noop_bounds(_: &Bounds<Pixels>, _: &mut Window, _: &mut App) {}

fn noop_hover(_: &bool, _: &mut Window, _: &mut App) {}

fn noop_mouse_down(_: &MouseDownEvent, _: &mut Window, _: &mut App) {}

fn noop_mouse_up(_: &MouseUpEvent, _: &mut Window, _: &mut App) {}

fn noop_drag_move(_: &DragMoveEvent<SliderDrag>, _: &mut Window, _: &mut App) {}
