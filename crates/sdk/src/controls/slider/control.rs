use gpui::{
    App, Bounds, Context, DragMoveEvent, Empty, EventEmitter, Focusable, IntoElement, MouseDownEvent, MouseUpEvent,
    Pixels, Point, Render, SharedString, Window, div, prelude::*, px,
};

use super::{SliderBuilder, SliderInputStrategy, SliderOrientation, SliderRenderModel, SliderTemplateHandlers};
use crate::controls::interaction::ControlInteraction;
use crate::controls::slider::model::SliderModel;
use crate::controls::value::{ControlRange, value_from_input};
use crate::keyhandling::{
    ControlKeyProfile, DecreaseValue, DecreaseValueLarge, IncreaseValue, IncreaseValueLarge, MoveToEnd, MoveToStart,
};
use crate::theme::observe_theme_revision;

#[derive(Clone, Debug)]
pub enum SliderEvent {
    Change { value: f32 },
}

#[derive(Clone, Debug)]
pub struct SliderDrag {
    id: SharedString,
}

impl SliderDrag {
    pub fn new(id: SharedString) -> Self {
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
    angular_drag_angle_offset: Option<f32>,
    angular_drag_pointer_angle: Option<f32>,
}

impl EventEmitter<SliderEvent> for SliderControl {}

impl SliderControl {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> SliderBuilder {
        SliderBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: SliderBuilder, cx: &mut Context<Self>) -> Self {
        let enabled = builder.model.enabled;

        observe_theme_revision(cx, |_, cx| cx.notify()).detach();
        Self {
            model: builder.model,
            interaction: ControlInteraction::new(enabled, cx),
            track_bounds: None,
            angular_drag_angle_offset: None,
            angular_drag_pointer_angle: None,
        }
    }

    pub fn value(&self) -> f32 {
        self.model.value
    }

    pub fn range(&self) -> ControlRange {
        self.model.range
    }

    pub fn strategy(&self) -> SliderInputStrategy {
        self.model.strategy
    }

    pub fn orientation(&self) -> SliderOrientation {
        self.model.strategy.orientation()
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

    pub fn set_strategy(&mut self, strategy: SliderInputStrategy, cx: &mut Context<Self>) {
        if self.model.strategy != strategy {
            self.model.strategy = strategy;
            self.reset_drag_tracking();
            cx.notify();
        }
    }

    pub fn set_orientation(&mut self, orientation: SliderOrientation, cx: &mut Context<Self>) {
        self.set_strategy(orientation.into(), cx);
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.enabled = enabled;
        self.interaction.set_enabled(enabled);
        if !enabled {
            self.reset_drag_tracking();
        }
        cx.notify();
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn super::SliderTemplate>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    fn render_model<'a>(&'a self, window: &Window) -> SliderRenderModel<'a> {
        SliderRenderModel {
            id: &self.model.id,
            strategy: self.model.strategy,
            orientation: self.model.strategy.orientation(),
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

        let Some(percentage) = percentage_from_position(self.model.strategy, bounds, position) else {
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
        let value_changed = match self.model.strategy {
            SliderInputStrategy::Angular { min_angle, max_angle } => {
                if self.model.enabled {
                    let handle_angle =
                        angle_for_percentage(min_angle, max_angle, self.model.range.percentage(self.model.value));
                    let pointer_angle = self
                        .track_bounds
                        .and_then(|bounds| angle_from_position(bounds, event.position))
                        .map(|pointer_angle| unwrap_angle_near(handle_angle, pointer_angle));
                    self.angular_drag_angle_offset = pointer_angle.map(|pointer_angle| pointer_angle - handle_angle);
                    self.angular_drag_pointer_angle = pointer_angle;
                }
                false
            }
            SliderInputStrategy::Horizontal | SliderInputStrategy::Vertical => {
                self.set_value_from_position(event.position, true, cx)
            }
        };

        if interaction_changed || value_changed {
            cx.notify();
        }
    }

    fn handle_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.reset_drag_tracking();
        if self.interaction.handle_mouse_up() {
            cx.notify();
        }
    }

    fn handle_drag_move(&mut self, event: &DragMoveEvent<SliderDrag>, _window: &mut Window, cx: &mut Context<Self>) {
        if event.drag(cx).id != self.model.id {
            return;
        }

        match self.model.strategy {
            SliderInputStrategy::Angular { min_angle, max_angle } => {
                let bounds = self.track_bounds.unwrap_or(event.bounds);
                if let (Some(pointer_angle), Some(offset), Some(previous_pointer_angle)) = (
                    angle_from_position(bounds, event.event.position),
                    self.angular_drag_angle_offset,
                    self.angular_drag_pointer_angle,
                ) {
                    let pointer_angle = unwrap_angle_near(previous_pointer_angle, pointer_angle);
                    self.angular_drag_pointer_angle = Some(pointer_angle);
                    let target_angle = (pointer_angle - offset).clamp(min_angle, max_angle);
                    let percentage = ((target_angle - min_angle) / (max_angle - min_angle)).clamp(0.0, 1.0);
                    let value = self.model.range.value_at(percentage);
                    self.set_value_internal(value, true, cx);
                }
            }
            SliderInputStrategy::Horizontal | SliderInputStrategy::Vertical => {
                if self.track_bounds.is_none() {
                    self.track_bounds = Some(event.bounds);
                }
                self.set_value_from_position(event.event.position, true, cx);
            }
        }
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

    fn reset_drag_tracking(&mut self) {
        self.angular_drag_angle_offset = None;
        self.angular_drag_pointer_angle = None;
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

fn percentage_from_position(
    strategy: SliderInputStrategy,
    bounds: Bounds<Pixels>,
    position: Point<Pixels>,
) -> Option<f32> {
    match strategy {
        SliderInputStrategy::Horizontal => {
            if bounds.size.width <= px(0.0) {
                return None;
            }

            Some(((position.x - bounds.left()) / bounds.size.width).clamp(0.0, 1.0))
        }
        SliderInputStrategy::Vertical => {
            if bounds.size.height <= px(0.0) {
                return None;
            }

            Some((1.0 - ((position.y - bounds.top()) / bounds.size.height)).clamp(0.0, 1.0))
        }
        SliderInputStrategy::Angular { min_angle, max_angle } => {
            let angle = angle_from_position(bounds, position)?;
            let span = max_angle - min_angle;
            if span.abs() <= f32::EPSILON {
                return None;
            }

            let normalized_angle = normalize_angle_for_range(angle, min_angle, max_angle);

            Some(((normalized_angle.clamp(min_angle, max_angle) - min_angle) / span).clamp(0.0, 1.0))
        }
    }
}

fn angle_from_position(bounds: Bounds<Pixels>, position: Point<Pixels>) -> Option<f32> {
    let center = bounds.center();
    let dy = (position.y - center.y).as_f32();
    let dx = (position.x - center.x).as_f32();

    if dx.abs() <= f32::EPSILON && dy.abs() <= f32::EPSILON {
        return None;
    }

    Some(dy.atan2(dx))
}

fn angle_for_percentage(min_angle: f32, max_angle: f32, percentage: f32) -> f32 {
    min_angle + (max_angle - min_angle) * percentage.clamp(0.0, 1.0)
}

fn normalize_angle_for_range(angle: f32, min_angle: f32, max_angle: f32) -> f32 {
    let mut normalized_angle = angle;

    while normalized_angle > max_angle {
        normalized_angle -= std::f32::consts::TAU;
    }
    while normalized_angle < min_angle {
        normalized_angle += std::f32::consts::TAU;
    }

    normalized_angle
}

fn unwrap_angle_near(reference: f32, angle: f32) -> f32 {
    let mut angle = angle;

    while angle - reference > std::f32::consts::PI {
        angle -= std::f32::consts::TAU;
    }
    while angle - reference < -std::f32::consts::PI {
        angle += std::f32::consts::TAU;
    }

    angle
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use gpui::{Bounds, Pixels, point, px, size};

    use super::{SliderInputStrategy, percentage_from_position, unwrap_angle_near};

    fn test_bounds() -> Bounds<Pixels> {
        Bounds { origin: point(px(10.0), px(20.0)), size: size(px(200.0), px(120.0)) }
    }

    #[test]
    fn horizontal_percentage_tracks_x_position() {
        let percentage =
            percentage_from_position(SliderInputStrategy::Horizontal, test_bounds(), point(px(60.0), px(30.0)));

        assert_eq!(percentage, Some(0.25));
    }

    #[test]
    fn vertical_percentage_inverts_y_position() {
        let percentage =
            percentage_from_position(SliderInputStrategy::Vertical, test_bounds(), point(px(20.0), px(50.0)));

        assert_eq!(percentage, Some(0.75));
    }

    #[test]
    fn angular_percentage_maps_clamped_arc() {
        let strategy = SliderInputStrategy::Angular { min_angle: -1.25 * PI, max_angle: 0.25 * PI };
        let bounds = Bounds { origin: point(px(0.0), px(0.0)), size: size(px(200.0), px(200.0)) };
        let percentage = percentage_from_position(strategy, bounds, point(px(100.0), px(0.0)));

        assert_eq!(percentage, Some(0.5));
    }

    #[test]
    fn unwrap_angle_keeps_pointer_continuous_across_atan_seam() {
        let reference = -3.9;
        let raw_angle = 2.9;
        let unwrapped = unwrap_angle_near(reference, raw_angle);

        assert!(unwrapped < -3.0, "expected seam-crossing angle near the reference, got {unwrapped}");
    }
}
