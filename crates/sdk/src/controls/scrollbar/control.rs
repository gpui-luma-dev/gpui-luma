use gpui::{
    App, Bounds, Context, DragMoveEvent, Empty, EventEmitter, Focusable, IntoElement, MouseDownEvent, MouseUpEvent,
    Pixels, Point, Render, ScrollDelta, ScrollWheelEvent, SharedString, Window, div, prelude::*, px,
};

use super::{ScrollbarBuilder, ScrollbarOrientation, ScrollbarRenderModel, ScrollbarTemplateHandlers};
use crate::controls::interaction::ControlInteraction;
use crate::controls::scrollbar::model::{ScrollbarModel, normalized_thumb_fraction};
use crate::controls::value::{ControlRange, value_from_input};
use crate::keyhandling::{
    ControlKeyProfile, DecreaseValue, DecreaseValueLarge, IncreaseValue, IncreaseValueLarge, MoveToEnd, MoveToStart,
};
use crate::theme::observe_theme_revision;

#[derive(Clone, Debug)]
pub enum ScrollbarEvent {
    Change { value: f32 },
}

#[derive(Clone, Debug)]
pub struct ScrollbarDrag {
    id: SharedString,
}

impl ScrollbarDrag {
    pub(crate) fn new(id: SharedString) -> Self {
        Self { id }
    }
}

impl Render for ScrollbarDrag {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

pub struct Scrollbar {
    model: ScrollbarModel,
    interaction: ControlInteraction,
    track_bounds: Option<Bounds<Pixels>>,
    thumb_length: Option<f32>,
    drag_anchor: Option<f32>,
    scroll_remainder: f32,
}

const DEFAULT_MIN_THUMB_LENGTH: f32 = 28.0;

impl EventEmitter<ScrollbarEvent> for Scrollbar {}

impl Scrollbar {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> ScrollbarBuilder {
        ScrollbarBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: ScrollbarBuilder, cx: &mut Context<Self>) -> Self {
        let enabled = builder.model.enabled;

        observe_theme_revision(cx, |_, cx| cx.notify()).detach();
        Self {
            model: builder.model,
            interaction: ControlInteraction::new(enabled, cx),
            track_bounds: None,
            thumb_length: None,
            drag_anchor: None,
            scroll_remainder: 0.0,
        }
    }

    pub fn value(&self) -> f32 {
        self.model.value
    }

    pub fn range(&self) -> ControlRange {
        self.model.range
    }

    pub fn orientation(&self) -> ScrollbarOrientation {
        self.model.orientation
    }

    pub fn step(&self) -> f32 {
        self.model.step
    }

    pub fn page_step(&self) -> f32 {
        self.model.page_step
    }

    pub fn thumb_fraction(&self) -> f32 {
        self.model.thumb_fraction
    }

    pub fn set_value(&mut self, value: impl Into<f64>, cx: &mut Context<Self>) {
        let value = value_from_input(value);
        self.set_value_internal(value, false, cx);
    }

    pub fn set_range(&mut self, range: impl Into<ControlRange>, cx: &mut Context<Self>) {
        let range = range.into();
        let value = range.snap(self.model.value, self.model.step);
        if self.model.range != range || (self.model.value - value).abs() > f32::EPSILON {
            self.model.range = range;
            self.model.value = value;
            cx.notify();
        }
    }

    pub fn set_orientation(&mut self, orientation: ScrollbarOrientation, cx: &mut Context<Self>) {
        self.model.orientation = orientation;
        self.drag_anchor = None;
        self.scroll_remainder = 0.0;
        cx.notify();
    }

    pub fn set_step(&mut self, step: impl Into<f64>, cx: &mut Context<Self>) {
        let step = crate::controls::value::normalized_step(value_from_input(step));
        let value = self.model.range.snap(self.model.value, step);
        if (self.model.step - step).abs() > f32::EPSILON || (self.model.value - value).abs() > f32::EPSILON {
            self.model.step = step;
            self.model.value = value;
            self.scroll_remainder = 0.0;
            cx.notify();
        }
    }

    pub fn set_page_step(&mut self, page_step: impl Into<f64>, cx: &mut Context<Self>) {
        let page_step = crate::controls::value::normalized_step(value_from_input(page_step));
        if (self.model.page_step - page_step).abs() > f32::EPSILON {
            self.model.page_step = page_step;
            cx.notify();
        }
    }

    pub fn set_thumb_fraction(&mut self, thumb_fraction: impl Into<f64>, cx: &mut Context<Self>) {
        let thumb_fraction = normalized_thumb_fraction(value_from_input(thumb_fraction));
        if (self.model.thumb_fraction - thumb_fraction).abs() > f32::EPSILON {
            self.model.thumb_fraction = thumb_fraction;
            cx.notify();
        }
    }

    pub fn set_length(&mut self, length: impl Into<f64>, cx: &mut Context<Self>) {
        let length = value_from_input(length);
        let length = (length.is_finite() && length > 0.0).then_some(length);
        if self.model.length != length {
            self.model.length = length;
            cx.notify();
        }
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn super::ScrollbarTemplate>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.enabled = enabled;
        self.interaction.set_enabled(enabled);
        if !enabled {
            self.drag_anchor = None;
            self.scroll_remainder = 0.0;
        }
        cx.notify();
    }

    fn render_model<'a>(&'a self, window: &Window) -> ScrollbarRenderModel<'a> {
        ScrollbarRenderModel {
            id: &self.model.id,
            orientation: self.model.orientation,
            range: self.model.range,
            step: self.model.step,
            page_step: self.model.page_step,
            value: self.model.value,
            percentage: self.model.range.percentage(self.model.value),
            thumb_fraction: self.model.thumb_fraction,
            length: self.model.length,
            enabled: self.model.enabled,
            state: self.interaction.render_state(self.model.enabled, window),
        }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> ScrollbarTemplateHandlers {
        ScrollbarTemplateHandlers {
            track_bounds: Box::new(cx.listener(Self::handle_track_bounds)),
            thumb_bounds: Box::new(cx.listener(Self::handle_thumb_bounds)),
            hover: Box::new(cx.listener(Self::handle_hover)),
            mouse_down: Box::new(cx.listener(Self::handle_mouse_down)),
            mouse_up: Box::new(cx.listener(Self::handle_mouse_up)),
            mouse_up_out: Box::new(cx.listener(Self::handle_mouse_up)),
            drag_move: Box::new(cx.listener(Self::handle_drag_move)),
            scroll_wheel: Box::new(cx.listener(Self::handle_scroll_wheel)),
        }
    }

    fn set_value_internal(&mut self, value: f32, emit: bool, cx: &mut Context<Self>) -> bool {
        let value = self.model.range.snap(value, self.model.step);

        if (self.model.value - value).abs() <= f32::EPSILON {
            return false;
        }

        self.model.value = value;

        if emit {
            cx.emit(ScrollbarEvent::Change { value });
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

        let track_length = self.axis_length(bounds);
        let track_length = f32::from(track_length);
        let thumb_length = self.resolved_thumb_length(track_length);

        let Some(percentage) = percentage_from_thumb_position(
            track_length,
            thumb_length,
            f32::from(self.axis_offset(position, bounds)),
            self.drag_anchor.unwrap_or(thumb_length * 0.5),
        ) else {
            return false;
        };

        let value = self.model.range.value_at(percentage);
        self.set_value_internal(value, emit, cx)
    }

    fn page_towards_position(&mut self, position: Point<Pixels>, cx: &mut Context<Self>) -> bool {
        let Some(bounds) = self.track_bounds else {
            return false;
        };

        let track_length = f32::from(self.axis_length(bounds));
        let thumb_length = self.resolved_thumb_length(track_length);
        let thumb_start = thumb_start_for(track_length, thumb_length, self.model.range.percentage(self.model.value));
        let pointer = f32::from(self.axis_offset(position, bounds));

        if pointer < thumb_start {
            self.adjust_value(-self.model.page_step, cx)
        } else if pointer > thumb_start + thumb_length {
            self.adjust_value(self.model.page_step, cx)
        } else {
            false
        }
    }

    fn set_drag_anchor_from_position(&mut self, position: Point<Pixels>) {
        let Some(bounds) = self.track_bounds else {
            return;
        };

        let track_length = f32::from(self.axis_length(bounds));
        let thumb_length = self.resolved_thumb_length(track_length);
        let thumb_start = thumb_start_for(track_length, thumb_length, self.model.range.percentage(self.model.value));
        let pointer = f32::from(self.axis_offset(position, bounds));

        self.drag_anchor = Some((pointer - thumb_start).clamp(0.0, thumb_length));
    }

    fn axis_length(&self, bounds: Bounds<Pixels>) -> Pixels {
        match self.model.orientation {
            ScrollbarOrientation::Horizontal => bounds.size.width,
            ScrollbarOrientation::Vertical => bounds.size.height,
        }
    }

    fn axis_offset(&self, position: Point<Pixels>, bounds: Bounds<Pixels>) -> Pixels {
        match self.model.orientation {
            ScrollbarOrientation::Horizontal => position.x - bounds.left(),
            ScrollbarOrientation::Vertical => position.y - bounds.top(),
        }
    }

    fn resolved_thumb_length(&self, track_length: f32) -> f32 {
        self.thumb_length
            .filter(|length| *length > 0.0)
            .unwrap_or_else(|| thumb_length_for(track_length, self.model.thumb_fraction, DEFAULT_MIN_THUMB_LENGTH))
            .min(track_length)
    }

    fn handle_track_bounds(&mut self, bounds: &Bounds<Pixels>, _window: &mut Window, _cx: &mut Context<Self>) {
        self.track_bounds = Some(*bounds);
    }

    fn handle_thumb_bounds(&mut self, bounds: &Bounds<Pixels>, _window: &mut Window, _cx: &mut Context<Self>) {
        self.thumb_length = Some(f32::from(self.axis_length(*bounds)));
    }

    fn handle_hover(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.handle_hover(*hovered) {
            cx.notify();
        }
    }

    fn handle_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let interaction_changed = self.interaction.handle_mouse_down(self.model.enabled, window, cx);
        if !self.model.enabled {
            if interaction_changed {
                cx.notify();
            }
            return;
        }

        self.set_drag_anchor_from_position(event.position);
        let value_changed = self.page_towards_position(event.position, cx);
        if value_changed {
            self.set_drag_anchor_from_position(event.position);
        }

        if interaction_changed || value_changed {
            cx.notify();
        }
    }

    fn handle_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.drag_anchor = None;
        if self.interaction.handle_mouse_up() {
            cx.notify();
        }
    }

    fn handle_drag_move(&mut self, event: &DragMoveEvent<ScrollbarDrag>, _window: &mut Window, cx: &mut Context<Self>) {
        if event.drag(cx).id != self.model.id {
            return;
        }

        self.scroll_remainder = 0.0;
        self.track_bounds = Some(event.bounds);
        self.set_value_from_position(event.event.position, true, cx);
    }

    fn handle_scroll_wheel(&mut self, event: &ScrollWheelEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.interaction.focus_handle().is_focused(window) {
            return;
        }

        let delta = scroll_value_delta(event.delta, self.model.orientation);
        if delta.abs() <= f32::EPSILON {
            return;
        }

        let value = self.model.value;
        self.scroll_remainder += delta;
        let target = value + self.scroll_remainder;
        let clamped_target = self.model.range.clamp(target);
        self.scroll_remainder = clamped_target - value;

        if self.set_value_internal(clamped_target, true, cx) {
            self.scroll_remainder -= self.model.value - value;
        }

        cx.stop_propagation();
    }

    fn adjust_value(&mut self, delta: f32, cx: &mut Context<Self>) -> bool {
        if !self.model.enabled {
            return false;
        }

        self.scroll_remainder = 0.0;
        self.set_value_internal(self.model.value + delta, true, cx)
    }

    fn move_to_value(&mut self, value: f32, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        self.scroll_remainder = 0.0;
        self.set_value_internal(value, true, cx);
    }

    fn handle_decrease_value(&mut self, _: &DecreaseValue, _window: &mut Window, cx: &mut Context<Self>) {
        self.adjust_value(-self.model.step, cx);
    }

    fn handle_increase_value(&mut self, _: &IncreaseValue, _window: &mut Window, cx: &mut Context<Self>) {
        self.adjust_value(self.model.step, cx);
    }

    fn handle_decrease_value_large(&mut self, _: &DecreaseValueLarge, _window: &mut Window, cx: &mut Context<Self>) {
        self.adjust_value(-self.model.page_step, cx);
    }

    fn handle_increase_value_large(&mut self, _: &IncreaseValueLarge, _window: &mut Window, cx: &mut Context<Self>) {
        self.adjust_value(self.model.page_step, cx);
    }

    fn handle_move_to_start(&mut self, _: &MoveToStart, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_to_value(self.model.range.start, cx);
    }

    fn handle_move_to_end(&mut self, _: &MoveToEnd, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_to_value(self.model.range.end, cx);
    }
}

impl Focusable for Scrollbar {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.interaction.focus_handle().clone()
    }
}

impl Render for Scrollbar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.render_model(window);
        let handlers = self.template_handlers(cx);

        div()
            .child(
                self.model
                    .template
                    .render(&model, handlers, window, cx)
                    .track_focus(self.interaction.focus_handle())
                    .key_context(ControlKeyProfile::ScrollOffset.context())
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

fn thumb_length_for(track_length: f32, thumb_fraction: f32, min_thumb_length: f32) -> f32 {
    if track_length <= 0.0 {
        return 0.0;
    }

    let min_thumb_length = min_thumb_length.min(track_length);
    (track_length * thumb_fraction.clamp(0.05, 1.0)).clamp(min_thumb_length, track_length)
}

fn thumb_start_for(track_length: f32, thumb_length: f32, percentage: f32) -> f32 {
    ((track_length - thumb_length).max(0.0) * percentage.clamp(0.0, 1.0)).clamp(0.0, track_length)
}

fn percentage_from_thumb_position(
    track_length: f32,
    thumb_length: f32,
    pointer_offset: f32,
    drag_anchor: f32,
) -> Option<f32> {
    let travel = track_length - thumb_length;
    if travel <= 0.0 {
        return None;
    }

    Some(((pointer_offset - drag_anchor) / travel).clamp(0.0, 1.0))
}

fn scroll_value_delta(delta: ScrollDelta, orientation: ScrollbarOrientation) -> f32 {
    let pixel_delta = delta.pixel_delta(px(20.0));
    let axis_delta = match orientation {
        ScrollbarOrientation::Horizontal => pixel_delta.x,
        ScrollbarOrientation::Vertical => pixel_delta.y,
    };

    // Important: GPUI scroll deltas are already adjusted for the platform and user settings
    // such as macOS natural scrolling, so do not invert this in the control.
    f32::from(axis_delta)
}

#[cfg(test)]
mod tests {
    use gpui::{ScrollDelta, point, px};

    use super::{percentage_from_thumb_position, scroll_value_delta, thumb_length_for, thumb_start_for};
    use crate::controls::scrollbar::ScrollbarOrientation;

    #[test]
    fn thumb_length_respects_fraction_minimum_and_track() {
        assert_eq!(thumb_length_for(200.0, 0.25, 18.0), 50.0);
        assert_eq!(thumb_length_for(200.0, 0.01, 18.0), 18.0);
        assert_eq!(thumb_length_for(12.0, 0.25, 18.0), 12.0);
    }

    #[test]
    fn thumb_start_uses_remaining_travel() {
        assert_eq!(thumb_start_for(200.0, 50.0, 0.0), 0.0);
        assert_eq!(thumb_start_for(200.0, 50.0, 0.5), 75.0);
        assert_eq!(thumb_start_for(200.0, 50.0, 1.0), 150.0);
    }

    #[test]
    fn pointer_position_maps_through_drag_anchor() {
        assert_eq!(percentage_from_thumb_position(200.0, 50.0, 100.0, 25.0), Some(0.5));
        assert_eq!(percentage_from_thumb_position(200.0, 200.0, 100.0, 25.0), None);
    }

    #[test]
    fn scroll_delta_maps_to_scrollbar_value_axis() {
        assert_eq!(
            scroll_value_delta(ScrollDelta::Pixels(point(px(0.0), px(-12.0))), ScrollbarOrientation::Vertical),
            -12.0
        );
        assert_eq!(scroll_value_delta(ScrollDelta::Lines(point(-3.0, 0.0)), ScrollbarOrientation::Horizontal), -60.0);
    }
}
