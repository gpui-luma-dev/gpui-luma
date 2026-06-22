use std::ops::RangeInclusive;

use gpui::{
    AbsoluteLength, App, Bounds, Context, DragMoveEvent, Empty, EventEmitter, Focusable, IntoElement, MouseDownEvent,
    MouseUpEvent, Pixels, Point, Render, SharedString, Window, div, prelude::*, px,
};

use super::constraints::{normalize_intervals, step_allowed_value};
use super::model::{RangeSliderBuilder, RangeSliderModel, RangeSliderRenderModel, build_track_segments, constrain_value};
use super::template::RangeSliderTemplateHandlers;
use super::{RangeSliderTemplate, SliderOrientation, SliderThumbSize};
use crate::controls::interaction::ControlInteraction;
use crate::controls::value::{ControlRange, value_from_input};
use crate::keyhandling::{
    ControlKeyProfile, DecreaseValue, DecreaseValueLarge, IncreaseValue, IncreaseValueLarge, MoveToEnd, MoveToStart,
};
use crate::theme::{ControlSize, observe_theme_revision};

#[derive(Clone, Debug)]
pub enum RangeSliderEvent {
    Change { value: f32 },
    Release { value: f32 },
}

#[derive(Clone, Debug)]
pub struct RangeSliderDrag {
    id: SharedString,
}

impl RangeSliderDrag {
    pub fn new(id: SharedString) -> Self {
        Self { id }
    }
}

impl Render for RangeSliderDrag {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

pub struct RangeSliderControl {
    model: RangeSliderModel,
    interaction: ControlInteraction,
    track_bounds: Option<Bounds<Pixels>>,
}

impl EventEmitter<RangeSliderEvent> for RangeSliderControl {}

impl RangeSliderControl {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> RangeSliderBuilder {
        RangeSliderBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: RangeSliderBuilder, cx: &mut Context<Self>) -> Self {
        let enabled = builder.model.enabled;

        observe_theme_revision(cx, |_, cx| cx.notify()).detach();
        Self { model: builder.model, interaction: ControlInteraction::new(enabled, cx), track_bounds: None }
    }

    pub fn value(&self) -> f32 {
        self.model.value
    }

    pub fn size(&self) -> ControlSize {
        self.model.size
    }

    pub fn thumb_size(&self) -> Option<SliderThumbSize> {
        self.model.thumb_size
    }

    pub fn range(&self) -> ControlRange {
        self.model.range
    }

    pub fn orientation(&self) -> SliderOrientation {
        self.model.orientation
    }

    pub fn allowed_intervals(&self) -> &[RangeInclusive<f32>] {
        &self.model.allowed_intervals
    }

    pub fn set_value(&mut self, value: impl Into<f64>, cx: &mut Context<Self>) {
        let value = value_from_input(value);
        self.set_value_internal(value, false, cx);
    }

    pub fn set_size(&mut self, size: ControlSize, cx: &mut Context<Self>) {
        if self.model.size != size {
            self.model.size = size;
            cx.notify();
        }
    }

    pub fn set_thumb_size(&mut self, size: SliderThumbSize, cx: &mut Context<Self>) {
        if self.model.thumb_size != Some(size) {
            self.model.thumb_size = Some(size);
            cx.notify();
        }
    }

    pub fn clear_thumb_size(&mut self, cx: &mut Context<Self>) {
        if self.model.thumb_size.is_some() {
            self.model.thumb_size = None;
            cx.notify();
        }
    }

    pub fn set_corner_radius(&mut self, radius: AbsoluteLength, cx: &mut Context<Self>) {
        self.model.corner_radius = Some(radius);
        cx.notify();
    }

    pub fn clear_corner_radius(&mut self, cx: &mut Context<Self>) {
        if self.model.corner_radius.is_some() {
            self.model.corner_radius = None;
            cx.notify();
        }
    }

    pub fn set_range(&mut self, range: impl Into<ControlRange>, cx: &mut Context<Self>) {
        self.model.range = range.into();
        self.model.allowed_intervals = normalize_intervals(&self.model.allowed_intervals, self.model.range);
        self.model.value =
            constrain_value(self.model.value, self.model.range, self.model.step, &self.model.allowed_intervals);
        cx.notify();
    }

    pub fn set_step(&mut self, step: impl Into<f64>, cx: &mut Context<Self>) {
        self.model.step = crate::controls::value::normalized_step(value_from_input(step));
        self.model.value =
            constrain_value(self.model.value, self.model.range, self.model.step, &self.model.allowed_intervals);
        cx.notify();
    }

    pub fn set_orientation(&mut self, orientation: SliderOrientation, cx: &mut Context<Self>) {
        if self.model.orientation != orientation {
            self.model.orientation = orientation;
            cx.notify();
        }
    }

    pub fn set_allowed_intervals(&mut self, intervals: Vec<RangeInclusive<f32>>, cx: &mut Context<Self>) {
        self.model.allowed_intervals = normalize_intervals(&intervals, self.model.range);
        self.model.value =
            constrain_value(self.model.value, self.model.range, self.model.step, &self.model.allowed_intervals);
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.enabled = enabled;
        self.interaction.set_enabled(enabled);
        cx.notify();
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn RangeSliderTemplate>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    fn render_model<'a>(&'a self, window: &Window) -> RangeSliderRenderModel<'a> {
        RangeSliderRenderModel {
            id: &self.model.id,
            orientation: self.model.orientation,
            size: self.model.size,
            thumb_size: self.model.thumb_size,
            range: self.model.range,
            step: self.model.step,
            value: self.model.value,
            percentage: self.model.range.percentage(self.model.value),
            allowed_intervals: &self.model.allowed_intervals,
            track_segments: build_track_segments(&self.model.allowed_intervals, self.model.range),
            enabled: self.model.enabled,
            corner_radius: self.model.corner_radius,
            state: self.interaction.render_state(self.model.enabled, window),
        }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> RangeSliderTemplateHandlers {
        RangeSliderTemplateHandlers {
            track_bounds: Box::new(cx.listener(Self::handle_track_bounds)),
            hover: Box::new(cx.listener(Self::handle_hover)),
            mouse_down: Box::new(cx.listener(Self::handle_mouse_down)),
            mouse_up: Box::new(cx.listener(Self::handle_mouse_up)),
            mouse_up_out: Box::new(cx.listener(Self::handle_mouse_up)),
            drag_move: Box::new(cx.listener(Self::handle_drag_move)),
        }
    }

    fn set_value_internal(&mut self, value: f32, emit: bool, cx: &mut Context<Self>) -> bool {
        let value = constrain_value(value, self.model.range, self.model.step, &self.model.allowed_intervals);

        if (self.model.value - value).abs() <= f32::EPSILON {
            return false;
        }

        self.model.value = value;

        if emit {
            cx.emit(RangeSliderEvent::Change { value });
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

        let Some(percentage) = percentage_from_position(self.model.orientation, bounds, position) else {
            return false;
        };
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
            cx.emit(RangeSliderEvent::Release { value: self.model.value });
            cx.notify();
        }
    }

    fn handle_drag_move(
        &mut self,
        event: &DragMoveEvent<RangeSliderDrag>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.drag(cx).id != self.model.id {
            return;
        }

        if self.track_bounds.is_none() {
            self.track_bounds = Some(event.bounds);
        }
        self.set_value_from_position(event.event.position, true, cx);
    }

    fn adjust_value(&mut self, delta: f32, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        let value = step_allowed_value(
            self.model.value,
            delta,
            &self.model.allowed_intervals,
            self.model.range,
            self.model.step,
        );

        if self.set_value_internal(value, true, cx) {
            cx.emit(RangeSliderEvent::Release { value: self.model.value });
        }
    }

    fn move_to_value(&mut self, value: f32, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        if self.set_value_internal(value, true, cx) {
            cx.emit(RangeSliderEvent::Release { value: self.model.value });
        }
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

impl Focusable for RangeSliderControl {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.interaction.focus_handle().clone()
    }
}

impl Render for RangeSliderControl {
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

fn percentage_from_position(
    orientation: SliderOrientation,
    bounds: Bounds<Pixels>,
    position: Point<Pixels>,
) -> Option<f32> {
    match orientation {
        SliderOrientation::Horizontal => {
            if bounds.size.width <= px(0.0) {
                return None;
            }

            Some(((position.x - bounds.left()) / bounds.size.width).clamp(0.0, 1.0))
        }
        SliderOrientation::Vertical => {
            if bounds.size.height <= px(0.0) {
                return None;
            }

            Some((1.0 - ((position.y - bounds.top()) / bounds.size.height)).clamp(0.0, 1.0))
        }
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Bounds, Pixels, point, px, size};

    use super::{SliderOrientation, percentage_from_position};

    fn test_bounds() -> Bounds<Pixels> {
        Bounds { origin: point(px(10.0), px(20.0)), size: size(px(200.0), px(120.0)) }
    }

    #[test]
    fn horizontal_percentage_tracks_x_position() {
        let percentage =
            percentage_from_position(SliderOrientation::Horizontal, test_bounds(), point(px(60.0), px(30.0)));

        assert_eq!(percentage, Some(0.25));
    }

    #[test]
    fn vertical_percentage_inverts_y_position() {
        let percentage =
            percentage_from_position(SliderOrientation::Vertical, test_bounds(), point(px(20.0), px(50.0)));

        assert_eq!(percentage, Some(0.75));
    }
}
