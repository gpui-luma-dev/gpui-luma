use std::sync::Arc;

use gpui::{
    App, Bounds, Context, DragMoveEvent, Empty, EventEmitter, Focusable, IntoElement, MouseDownEvent, MouseUpEvent,
    Pixels, Point, Render, SharedString, Window, div, prelude::*,
};

use super::input::{
    SliderInputStrategy, angle_for_percentage, angle_from_position, angular_drag_value, percentage_from_position,
    unwrap_angle_near,
};
use super::model::{
    build_render_segments, constrain_primary_value, primary_value, SliderBuilder, SliderOrientation, SliderRenderModel,
    SliderThumbSize, ThumbId, TrackPresentation,
};
use super::thumbs::{insert_thumb, nearest_thumb, normalized_hit_radius, remove_thumb, set_thumb_position, thumb_value};
use super::{SliderTemplateHandlers, step_allowed_value};
use crate::controls::interaction::ControlInteraction;
use super::model::SliderModel;
use crate::controls::value::{ControlRange, value_from_input};
use crate::keyhandling::{
    ControlKeyProfile, DecreaseValue, DecreaseValueLarge, IncreaseValue, IncreaseValueLarge, MoveToEnd, MoveToStart,
    RemoveValue,
};
use crate::theme::{ControlSize, observe_theme_revision};

#[derive(Clone, Debug)]
pub enum SliderEvent {
    Change { thumb_id: ThumbId, value: f32 },
    Release { thumb_id: ThumbId, value: f32 },
    ThumbAdded { thumb_id: ThumbId, value: f32 },
    ThumbRemoved { thumb_id: ThumbId },
    ThumbSelected { thumb_id: ThumbId },
}

#[derive(Clone, Debug)]
pub struct SliderDrag {
    id: SharedString,
    thumb_id: ThumbId,
}

impl SliderDrag {
    pub fn new(id: SharedString, thumb_id: ThumbId) -> Self {
        Self { id, thumb_id }
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
    active_thumb_id: Option<ThumbId>,
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
        let active_thumb_id = builder.model.thumbs.first().map(|thumb| thumb.id);

        observe_theme_revision(cx, |_, cx| cx.notify()).detach();
        Self {
            model: builder.model,
            interaction: ControlInteraction::new(enabled, cx),
            track_bounds: None,
            active_thumb_id,
            angular_drag_angle_offset: None,
            angular_drag_pointer_angle: None,
        }
    }

    pub fn value(&self) -> f32 {
        primary_value(&self.model)
    }

    pub fn thumbs(&self) -> &[super::model::SliderThumbValue] {
        &self.model.thumbs
    }

    pub fn active_thumb_id(&self) -> Option<ThumbId> {
        self.active_thumb_id
    }

    pub fn thumb_value(&self, thumb_id: ThumbId) -> Option<f32> {
        self.model
            .thumbs
            .iter()
            .find(|thumb| thumb.id == thumb_id)
            .map(|thumb| thumb_value(thumb, self.model.range, &self.model))
    }

    pub fn primary_thumb_id(&self) -> ThumbId {
        self.model.thumbs.first().map(|thumb| thumb.id).expect("slider always has a primary thumb")
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

    pub fn strategy(&self) -> SliderInputStrategy {
        self.model.strategy
    }

    pub fn orientation(&self) -> SliderOrientation {
        self.model.strategy.orientation()
    }

    pub fn set_value(&mut self, value: impl Into<f64>, cx: &mut Context<Self>) {
        let value = value_from_input(value);
        let thumb_id = self.active_thumb_id.or_else(|| self.model.thumbs.first().map(|thumb| thumb.id));
        if let Some(thumb_id) = thumb_id {
            self.set_thumb_value_internal(thumb_id, value, false, cx);
        }
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.enabled = enabled;
        self.interaction.set_enabled(enabled);
        if !enabled {
            self.active_thumb_id = None;
        }
        cx.notify();
    }

    pub fn set_template(&mut self, template: Arc<dyn super::SliderTemplate>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    fn render_model<'a>(&'a self, window: &Window) -> SliderRenderModel<'a> {
        let presentation = if self.model.thumb_policy.is_multi_thumb() {
            TrackPresentation::Domain
        } else {
            self.model.presentation
        };

        SliderRenderModel {
            id: &self.model.id,
            strategy: self.model.strategy,
            orientation: self.model.strategy.orientation(),
            presentation,
            size: self.model.size,
            thumb_size: self.model.thumb_size,
            range: self.model.range,
            step: self.model.step,
            thumbs: &self.model.thumbs,
            track_segments: build_render_segments(&self.model),
            reversed: self.model.reversed,
            wrapping: self.model.wrapping,
            enabled: self.model.enabled,
            corner_radius: self.model.corner_radius,
            thumb_policy: self.model.thumb_policy,
            active_thumb_id: self.active_thumb_id,
            state: self.interaction.render_state(self.model.enabled, window),
        }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> SliderTemplateHandlers {
        let entity = cx.entity().clone();
        SliderTemplateHandlers {
            track_bounds: Box::new(cx.listener(Self::handle_track_bounds)),
            hover: Box::new(cx.listener(Self::handle_hover)),
            mouse_down: Box::new(cx.listener(Self::handle_mouse_down)),
            mouse_up: Box::new(cx.listener(Self::handle_mouse_up)),
            mouse_up_out: Box::new(cx.listener(Self::handle_mouse_up)),
            drag_move: Arc::new(cx.listener(Self::handle_drag_move)),
            thumb_mouse_down: Arc::new(
                move |thumb_id: &ThumbId, event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                    entity.update(cx, |this, cx| {
                        this.handle_thumb_mouse_down(thumb_id, event, window, cx);
                    });
                },
            ),
        }
    }

    fn select_thumb(&mut self, thumb_id: ThumbId, emit: bool, cx: &mut Context<Self>) -> bool {
        if self.active_thumb_id == Some(thumb_id) {
            return false;
        }

        self.active_thumb_id = Some(thumb_id);
        if emit {
            cx.emit(SliderEvent::ThumbSelected { thumb_id });
        }
        true
    }

    fn set_thumb_value_internal(&mut self, thumb_id: ThumbId, value: f32, emit: bool, cx: &mut Context<Self>) -> bool {
        let value = constrain_primary_value(value, &self.model);
        let position = self.model.range.percentage(value);
        self.set_thumb_position_internal(thumb_id, position, emit, cx)
    }

    fn set_thumb_position_internal(
        &mut self,
        thumb_id: ThumbId,
        percentage: f32,
        emit: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        if !set_thumb_position(&mut self.model, thumb_id, percentage) {
            return false;
        }

        if emit {
            let value = self.thumb_value(thumb_id).unwrap_or(self.model.range.start);
            cx.emit(SliderEvent::Change { thumb_id, value });
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

        let Some(percentage) = percentage_from_position(self.model.strategy, self.model.reversed, bounds, position)
        else {
            return false;
        };

        self.handle_linear_pointer_down(bounds, percentage, emit, cx)
    }

    fn handle_linear_pointer_down(
        &mut self,
        bounds: Bounds<Pixels>,
        percentage: f32,
        emit: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        let hit_radius = self.normalized_hit_radius(bounds);
        let mut changed = false;

        let nearest_hit = nearest_thumb(&self.model.thumbs, percentage, hit_radius).map(|(thumb, _)| thumb.id);
        if let Some(thumb_id) = nearest_hit {
            changed |= self.select_thumb(thumb_id, true, cx);
            changed |= self.set_thumb_position_internal(thumb_id, percentage, emit, cx);
        } else if self.model.thumb_policy.allow_insert && insert_thumb(&mut self.model, percentage).is_some() {
            let thumb_id = self.model.thumbs.last().map(|thumb| thumb.id).expect("inserted thumb");
            changed = true;
            self.select_thumb(thumb_id, true, cx);
            let value = self.thumb_value(thumb_id).unwrap_or(self.model.range.start);
            if emit {
                cx.emit(SliderEvent::ThumbAdded { thumb_id, value });
                cx.emit(SliderEvent::Change { thumb_id, value });
            }
            cx.notify();
        } else if let Some((thumb, _)) = nearest_thumb(&self.model.thumbs, percentage, f32::MAX) {
            let thumb_id = thumb.id;
            changed |= self.select_thumb(thumb_id, true, cx);
            changed |= self.set_thumb_position_internal(thumb_id, percentage, emit, cx);
        }

        changed
    }

    fn normalized_hit_radius(&self, bounds: Bounds<Pixels>) -> f32 {
        normalized_hit_radius(bounds, self.thumb_size_px())
    }

    fn thumb_size_px(&self) -> f32 {
        self.model
            .thumb_size
            .map(|size| match size {
                crate::controls::slider::SliderThumbSize::Sm => 14.0,
                crate::controls::slider::SliderThumbSize::Md => 18.0,
                crate::controls::slider::SliderThumbSize::Lg => 22.0,
            })
            .unwrap_or(18.0)
    }

    fn handle_track_bounds(&mut self, bounds: &Bounds<Pixels>, _window: &mut Window, _cx: &mut Context<Self>) {
        self.track_bounds = Some(*bounds);
    }

    fn handle_hover(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.handle_hover(*hovered) {
            cx.notify();
        }
    }

    fn handle_thumb_mouse_down(
        &mut self,
        thumb_id: &ThumbId,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let interaction_changed = self.interaction.handle_mouse_down(self.model.enabled, window, cx);
        let selection_changed = self.select_thumb(*thumb_id, true, cx);

        if interaction_changed || selection_changed {
            cx.notify();
        }
    }

    fn handle_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let interaction_changed = self.interaction.handle_mouse_down(self.model.enabled, window, cx);
        let value_changed = match self.model.strategy {
            SliderInputStrategy::Angular { min_angle, max_angle } => {
                let thumb_id = self.active_thumb_id.or_else(|| self.model.thumbs.first().map(|thumb| thumb.id));
                if self.model.enabled
                    && let Some(thumb_id) = thumb_id
                {
                    self.select_thumb(thumb_id, false, cx);
                    let value = self.thumb_value(thumb_id).unwrap_or(self.model.range.start);
                    let handle_angle = angle_for_percentage(min_angle, max_angle, self.model.range.percentage(value));
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
            if let Some(thumb_id) = self.active_thumb_id {
                let value = self.thumb_value(thumb_id).unwrap_or(self.model.range.start);
                cx.emit(SliderEvent::Release { thumb_id, value });
            }
            cx.notify();
        }
    }

    fn handle_drag_move(&mut self, event: &DragMoveEvent<SliderDrag>, _window: &mut Window, cx: &mut Context<Self>) {
        let drag = event.drag(cx);
        if drag.id != self.model.id {
            return;
        }

        let thumb_id = drag.thumb_id;
        self.select_thumb(thumb_id, false, cx);

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
                    if let Some(value) = angular_drag_value(
                        pointer_angle,
                        offset,
                        min_angle,
                        max_angle,
                        self.model.range,
                        self.model.wrapping && self.model.allowed_intervals.is_empty(),
                        self.model.step,
                    ) {
                        self.set_thumb_value_internal(thumb_id, value, true, cx);
                    }
                }
            }
            SliderInputStrategy::Horizontal | SliderInputStrategy::Vertical => {
                if self.track_bounds.is_none() {
                    self.track_bounds = Some(event.bounds);
                }

                let bounds = self.track_bounds.unwrap_or(event.bounds);

                if let Some(percentage) =
                    percentage_from_position(self.model.strategy, self.model.reversed, bounds, event.event.position)
                {
                    self.set_thumb_position_internal(thumb_id, percentage, true, cx);
                }
            }
        }
    }

    fn remove_thumb(&mut self, thumb_id: ThumbId, cx: &mut Context<Self>) -> bool {
        if !remove_thumb(&mut self.model, thumb_id) {
            return false;
        }

        cx.emit(SliderEvent::ThumbRemoved { thumb_id });
        self.active_thumb_id = self.model.thumbs.first().map(|thumb| thumb.id);
        if let Some(next_id) = self.active_thumb_id {
            cx.emit(SliderEvent::ThumbSelected { thumb_id: next_id });
        }
        cx.notify();
        true
    }

    fn reset_drag_tracking(&mut self) {
        self.angular_drag_angle_offset = None;
        self.angular_drag_pointer_angle = None;
    }

    fn active_or_primary_thumb_id(&self) -> Option<ThumbId> {
        self.active_thumb_id.or_else(|| self.model.thumbs.first().map(|thumb| thumb.id))
    }

    fn adjust_value(&mut self, delta: f32, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        let Some(thumb_id) = self.active_or_primary_thumb_id() else {
            return;
        };

        let current = self.thumb_value(thumb_id).unwrap_or(self.model.range.start);
        let next = if self.model.wrapping && self.model.allowed_intervals.is_empty() {
            super::input::wrap_and_snap(
                current + super::layout::oriented_step_delta(delta, self.model.reversed),
                self.model.range,
                self.model.step,
            )
        } else {
            step_allowed_value(
                current,
                super::layout::oriented_step_delta(delta, self.model.reversed),
                &self.model.allowed_intervals,
                self.model.range,
                self.model.step,
            )
        };

        if self.set_thumb_value_internal(thumb_id, next, true, cx) {
            cx.emit(SliderEvent::Release { thumb_id, value: next });
        }
    }

    fn move_to_value(&mut self, value: f32, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        let Some(thumb_id) = self.active_or_primary_thumb_id() else {
            return;
        };

        if self.set_thumb_value_internal(thumb_id, value, true, cx) {
            let value = self.thumb_value(thumb_id).unwrap_or(value);
            cx.emit(SliderEvent::Release { thumb_id, value });
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

    fn handle_remove_value(&mut self, _: &RemoveValue, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.model.thumb_policy.allow_remove {
            return;
        }

        if let Some(thumb_id) = self.active_thumb_id {
            self.remove_thumb(thumb_id, cx);
        }
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
        let active_thumb_id = self
            .active_thumb_id
            .or_else(|| self.model.thumbs.first().map(|thumb| thumb.id))
            .expect("slider always has a thumb");

        div()
            .child(
                self.model
                    .template
                    .render(&model, handlers, active_thumb_id, window, cx)
                    .track_focus(self.interaction.focus_handle())
                    .key_context(ControlKeyProfile::RangeValue.context())
                    .on_action(cx.listener(Self::handle_decrease_value))
                    .on_action(cx.listener(Self::handle_increase_value))
                    .on_action(cx.listener(Self::handle_decrease_value_large))
                    .on_action(cx.listener(Self::handle_increase_value_large))
                    .on_action(cx.listener(Self::handle_move_to_start))
                    .on_action(cx.listener(Self::handle_move_to_end))
                    .on_action(cx.listener(Self::handle_remove_value)),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Bounds, Pixels, point, px, size};

    use crate::controls::slider::input::{SliderInputStrategy, percentage_from_position};
    use crate::controls::slider::layout;

    fn test_bounds() -> Bounds<Pixels> {
        Bounds { origin: point(px(10.0), px(20.0)), size: size(px(200.0), px(120.0)) }
    }

    #[test]
    fn horizontal_percentage_tracks_x_position() {
        let percentage =
            percentage_from_position(SliderInputStrategy::Horizontal, false, test_bounds(), point(px(60.0), px(30.0)));

        assert_eq!(percentage, Some(0.25));
    }

    #[test]
    fn horizontal_reversed_pointer_mirrors_x_position() {
        let percentage =
            percentage_from_position(SliderInputStrategy::Horizontal, true, test_bounds(), point(px(60.0), px(30.0)));

        assert_eq!(percentage, Some(0.75));
    }

    #[test]
    fn vertical_percentage_inverts_y_position() {
        let percentage =
            percentage_from_position(SliderInputStrategy::Vertical, false, test_bounds(), point(px(20.0), px(50.0)));

        assert_eq!(percentage, Some(0.75));
    }

    #[test]
    fn vertical_reversed_pointer_mirrors_y_position() {
        let percentage =
            percentage_from_position(SliderInputStrategy::Vertical, true, test_bounds(), point(px(20.0), px(50.0)));

        assert_eq!(percentage, Some(0.25));
    }

    #[test]
    fn reversed_pointer_round_trips_display_position() {
        let raw = layout::position_from_pointer(0.2, true);
        assert!((layout::display_position(raw, true) - 0.2).abs() <= f32::EPSILON);
    }
}
