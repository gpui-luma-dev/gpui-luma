use gpui::{
    App, Bounds, Context, DragMoveEvent, Empty, EventEmitter, Focusable, IntoElement, MouseDownEvent, MouseUpEvent,
    Pixels, Point, Render, SharedString, Window, div, prelude::*, px,
};

use super::{SliderBuilder, SliderRenderModel, SliderTemplateHandlers};
use crate::controls::interaction::ControlInteraction;
use crate::controls::slider::model::SliderModel;
use crate::controls::value::{ControlRange, value_from_input};
use crate::keyhandling::{
    ControlKeyProfile, DecreaseValue, DecreaseValueLarge, IncreaseValue, IncreaseValueLarge, MoveToEnd, MoveToStart,
};

#[derive(Clone, Debug)]
pub enum SliderEvent {
    Change { value: f32 },
}

#[derive(Clone, Debug)]
pub struct SliderDrag {
    id: SharedString,
}

impl SliderDrag {
    pub(crate) fn new(id: SharedString) -> Self {
        Self { id }
    }
}

impl Render for SliderDrag {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

pub struct SliderControl {
    model: SliderModel,
    interaction: ControlInteraction,
    track_bounds: Option<Bounds<Pixels>>,
}

impl EventEmitter<SliderEvent> for SliderControl {}

impl SliderControl {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> SliderBuilder {
        SliderBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: SliderBuilder, cx: &mut Context<Self>) -> Self {
        let enabled = builder.model.enabled;

        Self { model: builder.model, interaction: ControlInteraction::new(enabled, cx), track_bounds: None }
    }

    pub fn value(&self) -> f32 {
        self.model.value
    }

    pub fn range(&self) -> ControlRange {
        self.model.range
    }

    pub fn set_value(&mut self, value: impl Into<f64>, cx: &mut Context<Self>) {
        let value = value_from_input(value);
        self.set_value_internal(value, false, cx);
    }

    pub fn set_range(&mut self, range: impl Into<ControlRange>, cx: &mut Context<Self>) {
        self.model.range = range.into();
        self.model.value = self.model.range.snap(self.model.value, self.model.step);
        cx.notify();
    }

    pub fn set_step(&mut self, step: impl Into<f64>, cx: &mut Context<Self>) {
        self.model.step = crate::controls::value::normalized_step(value_from_input(step));
        self.model.value = self.model.range.snap(self.model.value, self.model.step);
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.enabled = enabled;
        self.interaction.set_enabled(enabled);
        cx.notify();
    }

    fn render_model<'a>(&'a self, window: &Window) -> SliderRenderModel<'a> {
        SliderRenderModel {
            id: &self.model.id,
            range: self.model.range,
            step: self.model.step,
            value: self.model.value,
            percentage: self.model.range.percentage(self.model.value),
            enabled: self.model.enabled,
            state: self.interaction.render_state(self.model.enabled, window),
        }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> SliderTemplateHandlers {
        SliderTemplateHandlers {
            track_bounds: Box::new(cx.listener(Self::handle_track_bounds)),
            hover: Box::new(cx.listener(Self::handle_hover)),
            mouse_down: Box::new(cx.listener(Self::handle_mouse_down)),
            mouse_up: Box::new(cx.listener(Self::handle_mouse_up)),
            mouse_up_out: Box::new(cx.listener(Self::handle_mouse_up)),
            drag_move: Box::new(cx.listener(Self::handle_drag_move)),
        }
    }

    fn set_value_internal(&mut self, value: f32, emit: bool, cx: &mut Context<Self>) -> bool {
        let value = self.model.range.snap(value, self.model.step);

        if (self.model.value - value).abs() <= f32::EPSILON {
            return false;
        }

        self.model.value = value;

        if emit {
            cx.emit(SliderEvent::Change { value });
        }

        cx.notify();
        true
    }

    fn set_value_from_position(&mut self, position: Point<Pixels>, emit: bool, cx: &mut Context<Self>) -> bool {
        if !self.model.enabled {
            return false;
        }

        let Some(bounds) = self.track_bounds else {
            return false;
        };

        if bounds.size.width <= px(0.0) {
            return false;
        }

        let percentage = ((position.x - bounds.left()) / bounds.size.width).clamp(0.0, 1.0);
        let value = self.model.range.value_at(percentage);

        self.set_value_internal(value, emit, cx)
    }

    fn handle_track_bounds(&mut self, bounds: &Bounds<Pixels>, _window: &mut Window, _cx: &mut Context<Self>) {
        self.track_bounds = Some(*bounds);
    }

    fn handle_hover(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.handle_hover(*hovered) {
            cx.notify();
        }
    }

    fn handle_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let interaction_changed = self.interaction.handle_mouse_down(self.model.enabled, window, cx);
        let value_changed = self.set_value_from_position(event.position, true, cx);

        if interaction_changed || value_changed {
            cx.notify();
        }
    }

    fn handle_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.handle_mouse_up() {
            cx.notify();
        }
    }

    fn handle_drag_move(&mut self, event: &DragMoveEvent<SliderDrag>, _window: &mut Window, cx: &mut Context<Self>) {
        if event.drag(cx).id != self.model.id {
            return;
        }

        self.track_bounds = Some(event.bounds);
        self.set_value_from_position(event.event.position, true, cx);
    }

    fn adjust_value(&mut self, delta: f32, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        self.set_value_internal(self.model.value + delta, true, cx);
    }

    fn move_to_value(&mut self, value: f32, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        self.set_value_internal(value, true, cx);
    }

    fn handle_decrease_value(&mut self, _: &DecreaseValue, _window: &mut Window, cx: &mut Context<Self>) {
        self.adjust_value(-self.model.step, cx);
    }

    fn handle_increase_value(&mut self, _: &IncreaseValue, _window: &mut Window, cx: &mut Context<Self>) {
        self.adjust_value(self.model.step, cx);
    }

    fn handle_decrease_value_large(&mut self, _: &DecreaseValueLarge, _window: &mut Window, cx: &mut Context<Self>) {
        self.adjust_value(-(self.model.step * 10.0), cx);
    }

    fn handle_increase_value_large(&mut self, _: &IncreaseValueLarge, _window: &mut Window, cx: &mut Context<Self>) {
        self.adjust_value(self.model.step * 10.0, cx);
    }

    fn handle_move_to_start(&mut self, _: &MoveToStart, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_to_value(self.model.range.start, cx);
    }

    fn handle_move_to_end(&mut self, _: &MoveToEnd, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_to_value(self.model.range.end, cx);
    }
}

impl Focusable for SliderControl {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.interaction.focus_handle().clone()
    }
}

impl Render for SliderControl {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.render_model(window);
        let handlers = self.template_handlers(cx);

        div()
            .child(
                self.model
                    .template
                    .render(&model, handlers, window, cx)
                    .track_focus(self.interaction.focus_handle())
                    .key_context(ControlKeyProfile::RangeValue.context())
                    .on_action(cx.listener(Self::handle_decrease_value))
                    .on_action(cx.listener(Self::handle_increase_value))
                    .on_action(cx.listener(Self::handle_decrease_value_large))
                    .on_action(cx.listener(Self::handle_increase_value_large))
                    .on_action(cx.listener(Self::handle_move_to_start))
                    .on_action(cx.listener(Self::handle_move_to_end)),
            )
            .into_any_element()
    }
}
