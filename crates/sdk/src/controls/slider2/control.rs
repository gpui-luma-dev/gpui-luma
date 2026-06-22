use gpui::{
    App, Bounds, Context, DragMoveEvent, Empty, EventEmitter, Focusable, IntoElement, MouseDownEvent, MouseUpEvent,
    Pixels, Point, Render, SharedString, Window, div, prelude::*, px,
};

use super::model::{
    build_render_segments, constrain_primary_value, primary_value, Slider2Builder, Slider2Orientation,
    Slider2RenderModel, Slider2ThumbSize, ThumbId,
};
use super::{Slider2TemplateHandlers, step_allowed_value};
use crate::controls::interaction::ControlInteraction;
use crate::controls::slider2::model::Slider2Model;
use crate::controls::value::{ControlRange, value_from_input};
use crate::keyhandling::{
    ControlKeyProfile, DecreaseValue, DecreaseValueLarge, IncreaseValue, IncreaseValueLarge, MoveToEnd, MoveToStart,
};
use crate::theme::{ControlSize, observe_theme_revision};

#[derive(Clone, Debug)]
pub enum Slider2Event {
    Change { value: f32 },
    Release { value: f32 },
}

#[derive(Clone, Debug)]
pub struct Slider2Drag {
    id: SharedString,
    #[allow(dead_code)]
    thumb_id: ThumbId,
}

impl Slider2Drag {
    pub fn new(id: SharedString, thumb_id: ThumbId) -> Self {
        Self { id, thumb_id }
    }
}

impl Render for Slider2Drag {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

pub struct Slider2Control {
    model: Slider2Model,
    interaction: ControlInteraction,
    track_bounds: Option<Bounds<Pixels>>,
    active_thumb_id: Option<ThumbId>,
}

impl EventEmitter<Slider2Event> for Slider2Control {}

impl Slider2Control {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> Slider2Builder {
        Slider2Builder::new(id)
    }

    pub(crate) fn from_builder(builder: Slider2Builder, cx: &mut Context<Self>) -> Self {
        let enabled = builder.model.enabled;

        observe_theme_revision(cx, |_, cx| cx.notify()).detach();
        Self {
            model: builder.model,
            interaction: ControlInteraction::new(enabled, cx),
            track_bounds: None,
            active_thumb_id: None,
        }
    }

    pub fn value(&self) -> f32 {
        primary_value(&self.model)
    }

    pub fn primary_thumb_id(&self) -> ThumbId {
        self.model.thumbs.first().map(|thumb| thumb.id).expect("slider2 always has a primary thumb")
    }

    pub fn size(&self) -> ControlSize {
        self.model.size
    }

    pub fn thumb_size(&self) -> Option<Slider2ThumbSize> {
        self.model.thumb_size
    }

    pub fn range(&self) -> ControlRange {
        self.model.range
    }

    pub fn orientation(&self) -> Slider2Orientation {
        self.model.orientation
    }

    pub fn set_value(&mut self, value: impl Into<f64>, cx: &mut Context<Self>) {
        let value = value_from_input(value);
        self.set_value_internal(value, false, cx);
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.enabled = enabled;
        self.interaction.set_enabled(enabled);
        if !enabled {
            self.active_thumb_id = None;
        }
        cx.notify();
    }

    fn render_model<'a>(&'a self, window: &Window) -> Slider2RenderModel<'a> {
        Slider2RenderModel {
            id: &self.model.id,
            orientation: self.model.orientation,
            presentation: self.model.presentation,
            size: self.model.size,
            thumb_size: self.model.thumb_size,
            range: self.model.range,
            step: self.model.step,
            thumbs: &self.model.thumbs,
            track_segments: build_render_segments(&self.model),
            reversed: self.model.reversed,
            enabled: self.model.enabled,
            corner_radius: self.model.corner_radius,
            domain_track: self.model.domain_track.clone(),
            state: self.interaction.render_state(self.model.enabled, window),
        }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> Slider2TemplateHandlers {
        Slider2TemplateHandlers {
            track_bounds: Box::new(cx.listener(Self::handle_track_bounds)),
            hover: Box::new(cx.listener(Self::handle_hover)),
            mouse_down: Box::new(cx.listener(Self::handle_mouse_down)),
            mouse_up: Box::new(cx.listener(Self::handle_mouse_up)),
            mouse_up_out: Box::new(cx.listener(Self::handle_mouse_up)),
            drag_move: Box::new(cx.listener(Self::handle_drag_move)),
        }
    }

    fn set_value_internal(&mut self, value: f32, emit: bool, cx: &mut Context<Self>) -> bool {
        let value = constrain_primary_value(value, &self.model);
        let position = self.model.range.percentage(value);

        let Some(thumb) = self.model.thumbs.first_mut() else {
            return false;
        };

        if (thumb.position - position).abs() <= f32::EPSILON {
            return false;
        }

        thumb.position = position;

        if emit {
            cx.emit(Slider2Event::Change { value });
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

        let Some(percentage) = percentage_from_position(self.model.orientation, self.model.reversed, bounds, position)
        else {
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
        self.active_thumb_id = self.model.thumbs.first().map(|thumb| thumb.id);
        let value_changed = self.set_value_from_position(event.position, true, cx);

        if interaction_changed || value_changed {
            cx.notify();
        }
    }

    fn handle_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.active_thumb_id = None;
        if self.interaction.handle_mouse_up() {
            cx.emit(Slider2Event::Release { value: self.value() });
            cx.notify();
        }
    }

    fn handle_drag_move(&mut self, event: &DragMoveEvent<Slider2Drag>, _window: &mut Window, cx: &mut Context<Self>) {
        let drag = event.drag(cx);
        if drag.id != self.model.id {
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

        let next = step_allowed_value(
            self.value(),
            super::layout::oriented_step_delta(delta, self.model.reversed),
            &self.model.allowed_intervals,
            self.model.range,
            self.model.step,
        );

        if self.set_value_internal(next, true, cx) {
            cx.emit(Slider2Event::Release { value: self.value() });
        }
    }

    fn move_to_value(&mut self, value: f32, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        if self.set_value_internal(value, true, cx) {
            cx.emit(Slider2Event::Release { value: self.value() });
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

impl Focusable for Slider2Control {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.interaction.focus_handle().clone()
    }
}

impl Render for Slider2Control {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.render_model(window);
        let handlers = self.template_handlers(cx);
        let primary_thumb_id = self.primary_thumb_id();

        div()
            .child(
                self.model
                    .template
                    .render(&model, handlers, primary_thumb_id, window, cx)
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
    orientation: Slider2Orientation,
    reversed: bool,
    bounds: Bounds<Pixels>,
    position: Point<Pixels>,
) -> Option<f32> {
    let raw = match orientation {
        Slider2Orientation::Horizontal => {
            if bounds.size.width <= px(0.0) {
                return None;
            }

            ((position.x - bounds.left()) / bounds.size.width).clamp(0.0, 1.0)
        }
        Slider2Orientation::Vertical => {
            if bounds.size.height <= px(0.0) {
                return None;
            }

            (1.0 - ((position.y - bounds.top()) / bounds.size.height)).clamp(0.0, 1.0)
        }
    };

    Some(super::layout::position_from_pointer(raw, reversed))
}

#[cfg(test)]
mod tests {
    use gpui::{Bounds, Pixels, point, px, size};

    use super::{Slider2Orientation, percentage_from_position};
    use crate::controls::slider2::layout;

    fn test_bounds() -> Bounds<Pixels> {
        Bounds { origin: point(px(10.0), px(20.0)), size: size(px(200.0), px(120.0)) }
    }

    #[test]
    fn horizontal_percentage_tracks_x_position() {
        let percentage =
            percentage_from_position(Slider2Orientation::Horizontal, false, test_bounds(), point(px(60.0), px(30.0)));

        assert_eq!(percentage, Some(0.25));
    }

    #[test]
    fn horizontal_reversed_pointer_mirrors_x_position() {
        let percentage =
            percentage_from_position(Slider2Orientation::Horizontal, true, test_bounds(), point(px(60.0), px(30.0)));

        assert_eq!(percentage, Some(0.75));
    }

    #[test]
    fn vertical_percentage_inverts_y_position() {
        let percentage =
            percentage_from_position(Slider2Orientation::Vertical, false, test_bounds(), point(px(20.0), px(50.0)));

        assert_eq!(percentage, Some(0.75));
    }

    #[test]
    fn vertical_reversed_pointer_mirrors_y_position() {
        let percentage =
            percentage_from_position(Slider2Orientation::Vertical, true, test_bounds(), point(px(20.0), px(50.0)));

        assert_eq!(percentage, Some(0.25));
    }

    #[test]
    fn reversed_pointer_round_trips_display_position() {
        let raw = layout::position_from_pointer(0.2, true);
        assert!((layout::display_position(raw, true) - 0.2).abs() <= f32::EPSILON);
    }
}
