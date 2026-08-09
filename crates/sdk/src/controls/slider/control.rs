use std::sync::Arc;
use std::time::Duration;

use gpui::{
    App, Bounds, Context, DragMoveEvent, Empty, EventEmitter, FocusOutEvent, Focusable, IntoElement, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Pixels, Point, Render, SharedString, Subscription, Window, div, prelude::*,
};

use super::input::{
    SliderInputStrategy, angle_for_percentage, angle_from_position, percentage_from_angle, percentage_from_position,
    unwrap_angle_near,
};
use super::model::{
    build_render_segments_at, constrain_primary_value, position_for_value, primary_value, value_for_position,
    SliderBuilder, SliderOrientation, SliderRenderModel, SliderThumbSize, SliderThumbValue, ThumbId, TrackPresentation,
};
use super::domain::DomainTrackRenderer;
use super::thumbs::{insert_thumb, nearest_thumb, normalized_hit_radius, remove_thumb, set_thumb_position, thumb_value};
use super::{SliderTemplateHandlers, step_allowed_value};
use crate::animation::{DEFAULT_TRANSITION_DURATION, VisualTransition};
use crate::controls::interaction::ControlInteraction;
use super::constraints::normalize_intervals;
use super::model::SliderModel;
use crate::controls::value::{ControlRange, value_from_input};
use crate::keyhandling::{
    ControlKeyProfile, DecreaseValue, DecreaseValueLarge, IncreaseValue, IncreaseValueLarge, MoveToEnd, MoveToStart,
    RemoveValue,
};
use crate::theme::{ControlSize, observe_theme_revision};

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum SliderEvent {
    Change { thumb_id: ThumbId, value: f32 },
    Release { thumb_id: ThumbId, value: f32 },
    DragStart { thumb_id: ThumbId },
    DragEnd { thumb_id: ThumbId, value: f32 },
    ThumbAdded { thumb_id: ThumbId, value: f32 },
    ThumbRemoved { thumb_id: ThumbId },
    ThumbSelected { thumb_id: ThumbId },
    FocusChanged { focused: bool },
    HoverChanged { hovered: bool },
    EnabledChanged { enabled: bool },
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
    focus_in_subscription: Option<Subscription>,
    focus_out_subscription: Option<Subscription>,
    emitted_focused: bool,
    animated: bool,
    thumb_transitions: Vec<VisualTransition>,
    display_thumbs: Vec<SliderThumbValue>,
}

impl EventEmitter<SliderEvent> for SliderControl {}

impl SliderControl {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> SliderBuilder {
        SliderBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: SliderBuilder, cx: &mut Context<Self>) -> Self {
        let enabled = builder.model.enabled;
        let animated = builder.animated;
        let active_thumb_id = builder.model.thumbs.first().map(|thumb| thumb.id);
        let mut model = builder.model;
        sync_thumb_previews(&mut model);
        let duration = transition_duration(animated);
        let thumb_transitions =
            model.thumbs.iter().map(|thumb| VisualTransition::new(thumb.position, duration)).collect();
        let display_thumbs = model.thumbs.clone();

        observe_theme_revision(cx, |_, cx| cx.notify()).detach();
        Self {
            model,
            interaction: ControlInteraction::new(enabled, cx),
            track_bounds: None,
            active_thumb_id,
            angular_drag_angle_offset: None,
            angular_drag_pointer_angle: None,
            focus_in_subscription: None,
            focus_out_subscription: None,
            emitted_focused: false,
            animated,
            thumb_transitions,
            display_thumbs,
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

    pub fn animated(&self) -> bool {
        self.animated
    }

    pub fn set_animated(&mut self, animated: bool, cx: &mut Context<Self>) {
        if self.animated == animated {
            return;
        }
        self.animated = animated;
        let duration = transition_duration(animated);
        self.thumb_transitions = self
            .model
            .thumbs
            .iter()
            .zip(self.thumb_transitions.iter())
            .map(|(thumb, transition)| {
                let mut next = VisualTransition::new(transition.progress(), duration);
                next.set_target(thumb.position);
                next
            })
            .collect();
        if self.thumb_transitions.len() != self.model.thumbs.len() {
            self.rebuild_thumb_transitions();
        }
        cx.notify();
    }

    pub fn set_value(&mut self, value: impl Into<f64>, cx: &mut Context<Self>) {
        let value = value_from_input(value);
        let thumb_id = self.active_thumb_id.or_else(|| self.model.thumbs.first().map(|thumb| thumb.id));
        if let Some(thumb_id) = thumb_id {
            self.set_thumb_value_internal(thumb_id, value, false, true, cx);
        }
    }

    pub fn set_range(&mut self, range: impl Into<ControlRange>, cx: &mut Context<Self>) {
        self.model.range = range.into();
        self.model.allowed_intervals = normalize_intervals(&self.model.allowed_intervals, self.model.range);
        sync_thumb_previews(&mut self.model);
        cx.notify();
    }

    pub fn set_allowed_intervals(&mut self, intervals: Vec<std::ops::RangeInclusive<f32>>, cx: &mut Context<Self>) {
        self.model.allowed_intervals = normalize_intervals(&intervals, self.model.range);
        sync_thumb_previews(&mut self.model);
        cx.notify();
    }

    pub fn clear_allowed_intervals(&mut self, cx: &mut Context<Self>) {
        if self.model.allowed_intervals.is_empty() {
            return;
        }
        self.model.allowed_intervals.clear();
        sync_thumb_previews(&mut self.model);
        cx.notify();
    }

    pub fn set_track_intervals(&mut self, intervals: Vec<std::ops::RangeInclusive<f32>>, cx: &mut Context<Self>) {
        self.model.track_intervals = Some(normalize_intervals(&intervals, self.model.range));
        cx.notify();
    }

    pub fn clear_track_intervals(&mut self, cx: &mut Context<Self>) {
        if self.model.track_intervals.is_none() {
            return;
        }
        self.model.track_intervals = None;
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }

        let was_hovered = self.interaction.hovered();
        let was_pressed = self.interaction.is_pressed();
        let active_thumb_id = self.active_thumb_id;
        self.model.enabled = enabled;
        self.interaction.set_enabled(enabled);
        if !enabled {
            self.active_thumb_id = None;
            if was_pressed && let Some(thumb_id) = active_thumb_id {
                let value = self.thumb_value(thumb_id).unwrap_or(self.model.range.start);
                cx.emit(SliderEvent::DragEnd { thumb_id, value });
            }
            if was_hovered {
                cx.emit(SliderEvent::HoverChanged { hovered: false });
            }
            self.emit_focus_changed(false, cx);
        }
        cx.emit(SliderEvent::EnabledChanged { enabled });
        cx.notify();
    }

    pub fn set_template(&mut self, template: Arc<dyn super::SliderTemplate>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    pub fn set_domain_track(&mut self, renderer: Arc<dyn DomainTrackRenderer>, cx: &mut Context<Self>) {
        self.model.presentation = TrackPresentation::Domain;
        self.model.domain_track = Some(renderer);
        sync_thumb_previews(&mut self.model);
        cx.notify();
    }

    pub fn set_thumb_position(&mut self, thumb_id: ThumbId, percentage: f32, cx: &mut Context<Self>) -> bool {
        self.set_thumb_position_internal(thumb_id, percentage.clamp(0.0, 1.0), true, true, cx)
    }

    pub fn select_thumb_id(&mut self, thumb_id: ThumbId, cx: &mut Context<Self>) -> bool {
        let changed = self.select_thumb(thumb_id, true, cx);
        if changed {
            cx.notify();
        }
        changed
    }

    pub fn insert_thumb_at(&mut self, percentage: f32, cx: &mut Context<Self>) -> Option<ThumbId> {
        let percentage = percentage.clamp(0.0, 1.0);
        let thumb_id = insert_thumb(&mut self.model, percentage)?;
        sync_thumb_preview(&mut self.model, thumb_id);
        self.sync_transitions_with_thumbs();
        if let Some(transition) = self.transition_for_thumb_mut(thumb_id) {
            transition.snap_to(percentage);
        }
        self.select_thumb(thumb_id, true, cx);
        let value = self.thumb_value(thumb_id).unwrap_or(self.model.range.start);
        cx.emit(SliderEvent::ThumbAdded { thumb_id, value });
        cx.emit(SliderEvent::Change { thumb_id, value });
        cx.notify();
        Some(thumb_id)
    }

    pub fn remove_thumb_id(&mut self, thumb_id: ThumbId, cx: &mut Context<Self>) -> bool {
        self.remove_thumb(thumb_id, cx)
    }

    pub fn sync_domain_thumb_previews(&mut self) {
        sync_thumb_previews(&mut self.model);
    }

    fn render_model<'a>(&'a self, window: &Window) -> SliderRenderModel<'a> {
        let presentation = if self.model.thumb_policy.is_multi_thumb() {
            TrackPresentation::Domain
        } else {
            self.model.presentation
        };
        let display_thumb_position = self.display_thumbs.first().map(|thumb| thumb.position).unwrap_or(0.0);

        SliderRenderModel {
            id: &self.model.id,
            strategy: self.model.strategy,
            orientation: self.model.strategy.orientation(),
            presentation,
            size: self.model.size,
            thumb_size: self.model.thumb_size,
            range: self.model.range,
            step: self.model.step,
            thumbs: &self.display_thumbs,
            track_segments: build_render_segments_at(&self.model, display_thumb_position),
            reversed: self.model.reversed,
            wrapping: self.model.wrapping,
            enabled: self.model.enabled,
            corner_radius: self.model.corner_radius,
            thumb_radius: self.model.thumb_radius,
            thumb_policy: self.model.thumb_policy,
            active_thumb_id: self.active_thumb_id,
            state: self.interaction.render_state(self.model.enabled, window),
            domain_track: self.model.domain_track.clone(),
        }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> SliderTemplateHandlers {
        let entity = cx.entity().clone();
        SliderTemplateHandlers {
            track_bounds: Box::new(cx.listener(Self::handle_track_bounds)),
            hover: Box::new(cx.listener(Self::handle_hover)),
            mouse_down: Box::new(cx.listener(Self::handle_mouse_down)),
            mouse_move: Box::new(cx.listener(Self::handle_mouse_move)),
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

    fn set_thumb_value_internal(
        &mut self,
        thumb_id: ThumbId,
        value: f32,
        emit: bool,
        animate: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        let value = constrain_primary_value(value, &self.model);
        let position = position_for_value(&self.model, value);
        self.set_thumb_position_internal(thumb_id, position, emit, animate, cx)
    }

    fn set_thumb_position_internal(
        &mut self,
        thumb_id: ThumbId,
        percentage: f32,
        emit: bool,
        animate: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        if !set_thumb_position(&mut self.model, thumb_id, percentage) {
            return false;
        }

        sync_thumb_preview(&mut self.model, thumb_id);
        self.apply_thumb_motion(thumb_id, percentage, animate);

        if emit {
            let value = self.thumb_value(thumb_id).unwrap_or(self.model.range.start);
            cx.emit(SliderEvent::Change { thumb_id, value });
        }

        cx.notify();
        true
    }

    fn apply_thumb_motion(&mut self, thumb_id: ThumbId, percentage: f32, animate: bool) {
        self.sync_transitions_with_thumbs();
        let should_animate = animate && self.animated;
        let Some(transition) = self.transition_for_thumb_mut(thumb_id) else {
            return;
        };
        if should_animate {
            transition.set_target(percentage);
        } else {
            transition.snap_to(percentage);
        }
    }

    fn transition_for_thumb_mut(&mut self, thumb_id: ThumbId) -> Option<&mut VisualTransition> {
        let index = self.model.thumbs.iter().position(|thumb| thumb.id == thumb_id)?;
        self.thumb_transitions.get_mut(index)
    }

    fn sync_transitions_with_thumbs(&mut self) {
        let duration = transition_duration(self.animated);
        while self.thumb_transitions.len() < self.model.thumbs.len() {
            let index = self.thumb_transitions.len();
            let position = self.model.thumbs[index].position;
            self.thumb_transitions.push(VisualTransition::new(position, duration));
        }
        if self.thumb_transitions.len() > self.model.thumbs.len() {
            self.thumb_transitions.truncate(self.model.thumbs.len());
        }
    }

    fn rebuild_thumb_transitions(&mut self) {
        let duration = transition_duration(self.animated);
        self.thumb_transitions =
            self.model.thumbs.iter().map(|thumb| VisualTransition::new(thumb.position, duration)).collect();
    }

    fn refresh_display_thumbs(&mut self) {
        self.display_thumbs = self
            .model
            .thumbs
            .iter()
            .enumerate()
            .map(|(index, thumb)| {
                let mut display = thumb.clone();
                display.position =
                    self.thumb_transitions.get(index).map(|transition| transition.progress()).unwrap_or(thumb.position);
                display
            })
            .collect();
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
            let constrained = self.constrained_position_for_raw_position(percentage);
            changed |= self.select_thumb(thumb_id, true, cx);
            changed |= self.set_thumb_position_internal(thumb_id, constrained, emit, false, cx);
        } else if self.model.thumb_policy.allow_insert && insert_thumb(&mut self.model, percentage).is_some() {
            let thumb_id = self.model.thumbs.last().map(|thumb| thumb.id).expect("inserted thumb");
            changed = true;
            self.sync_transitions_with_thumbs();
            if let Some(transition) = self.transition_for_thumb_mut(thumb_id) {
                transition.snap_to(percentage);
            }
            self.select_thumb(thumb_id, true, cx);
            let value = self.thumb_value(thumb_id).unwrap_or(self.model.range.start);
            if emit {
                cx.emit(SliderEvent::ThumbAdded { thumb_id, value });
                cx.emit(SliderEvent::Change { thumb_id, value });
            }
            cx.notify();
        } else if let Some((thumb, _)) = nearest_thumb(&self.model.thumbs, percentage, f32::MAX) {
            let thumb_id = thumb.id;
            let constrained = self.constrained_position_for_raw_position(percentage);
            changed |= self.select_thumb(thumb_id, true, cx);
            changed |= self.set_thumb_position_internal(thumb_id, constrained, emit, false, cx);
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
            cx.emit(SliderEvent::HoverChanged { hovered: self.interaction.hovered() });
            cx.notify();
        }
    }

    fn handle_focus_in(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.model.enabled && self.emit_focus_changed(true, cx) {
            cx.notify();
        }
    }

    fn handle_focus_out(&mut self, _: FocusOutEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.emit_focus_changed(false, cx) {
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
        let was_pressed = self.interaction.is_pressed();
        let interaction_changed = self.interaction.handle_mouse_down(self.model.enabled, window, cx);
        let selection_changed = self.select_thumb(*thumb_id, true, cx);
        self.emit_drag_start_if_needed(was_pressed, Some(*thumb_id), cx);

        if interaction_changed || selection_changed {
            cx.stop_propagation();
            cx.notify();
        }
    }

    fn handle_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        match self.model.strategy {
            SliderInputStrategy::Angular { min_angle, max_angle } => {
                let Some(bounds) = self.track_bounds else {
                    return;
                };
                if !self.angular_accepts_pointer(bounds, event.position) {
                    return;
                }
                let was_pressed = self.interaction.is_pressed();
                let interaction_changed = self.interaction.handle_mouse_down(self.model.enabled, window, cx);
                let thumb_id = self.active_thumb_id.or_else(|| self.model.thumbs.first().map(|thumb| thumb.id));
                let mut changed = false;
                if self.model.enabled
                    && let Some(thumb_id) = thumb_id
                {
                    self.select_thumb(thumb_id, false, cx);
                    if let Some(percentage) =
                        percentage_from_position(self.model.strategy, self.model.reversed, bounds, event.position)
                    {
                        let constrained = self.constrained_position_for_raw_position(percentage);
                        changed |= self.set_thumb_position_internal(thumb_id, constrained, true, false, cx);
                    }

                    let handle_position = self
                        .model
                        .thumbs
                        .iter()
                        .find(|thumb| thumb.id == thumb_id)
                        .map(|thumb| thumb.position)
                        .unwrap_or(0.0);
                    let handle_angle = angle_for_percentage(min_angle, max_angle, handle_position);
                    let pointer_angle = angle_from_position(bounds, event.position)
                        .map(|pointer_angle| unwrap_angle_near(handle_angle, pointer_angle));
                    self.angular_drag_angle_offset = Some(0.0);
                    self.angular_drag_pointer_angle = pointer_angle;
                }
                self.emit_drag_start_if_needed(was_pressed, thumb_id, cx);
                if interaction_changed || changed {
                    cx.stop_propagation();
                    cx.notify();
                }
            }
            SliderInputStrategy::Horizontal | SliderInputStrategy::Vertical => {
                let was_pressed = self.interaction.is_pressed();
                let interaction_changed = self.interaction.handle_mouse_down(self.model.enabled, window, cx);
                let changed = self.set_value_from_position(event.position, true, cx);
                self.emit_drag_start_if_needed(was_pressed, self.active_thumb_id, cx);
                if interaction_changed || changed {
                    cx.stop_propagation();
                    cx.notify();
                }
            }
        }
    }

    fn angular_accepts_pointer(&self, bounds: Bounds<Pixels>, pointer: Point<Pixels>) -> bool {
        let Some(target) = self.model.radial_hit_target.as_ref() else {
            return true;
        };
        let thumb_position = self
            .active_thumb_id
            .and_then(|thumb_id| self.model.thumbs.iter().find(|thumb| thumb.id == thumb_id))
            .or_else(|| self.model.thumbs.first())
            .map(|thumb| thumb.position)
            .unwrap_or(0.0);
        target.accepts_pointer(bounds, pointer, thumb_position, self.model.reversed)
    }

    fn handle_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.reset_drag_tracking();
        if self.interaction.handle_mouse_up() {
            if let Some(thumb_id) = self.active_thumb_id {
                let value = self.thumb_value(thumb_id).unwrap_or(self.model.range.start);
                cx.emit(SliderEvent::Release { thumb_id, value });
                cx.emit(SliderEvent::DragEnd { thumb_id, value });
            }
            cx.notify();
        }
    }

    fn handle_mouse_move(&mut self, event: &MouseMoveEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.interaction.is_pressed() || !self.model.enabled {
            return;
        }

        let SliderInputStrategy::Angular { min_angle, max_angle } = self.model.strategy else {
            return;
        };
        let Some(bounds) = self.track_bounds else {
            return;
        };

        let Some(thumb_id) = self.active_or_primary_thumb_id() else {
            return;
        };
        if let Some(pointer_angle) = angle_from_position(bounds, event.position) {
            let previous = self.angular_drag_pointer_angle.unwrap_or(pointer_angle);
            let pointer_angle = unwrap_angle_near(previous, pointer_angle);
            self.angular_drag_pointer_angle = Some(pointer_angle);
            let offset = self.angular_drag_angle_offset.unwrap_or(0.0);
            if let Some(raw_percentage) = percentage_from_angle(pointer_angle - offset, min_angle, max_angle)
                .map(|raw| if self.model.reversed { 1.0 - raw } else { raw })
            {
                let constrained = self.constrained_position_for_raw_position(raw_percentage);
                if self.set_thumb_position_internal(thumb_id, constrained, true, false, cx) {
                    cx.stop_propagation();
                }
            }
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
                    if let Some(raw_percentage) = percentage_from_angle(pointer_angle - offset, min_angle, max_angle)
                        .map(|raw| if self.model.reversed { 1.0 - raw } else { raw })
                    {
                        let constrained = self.constrained_position_for_raw_position(raw_percentage);
                        self.set_thumb_position_internal(thumb_id, constrained, true, false, cx);
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
                    let constrained = self.constrained_position_for_raw_position(percentage);
                    self.set_thumb_position_internal(thumb_id, constrained, true, false, cx);
                }
            }
        }
    }

    fn remove_thumb(&mut self, thumb_id: ThumbId, cx: &mut Context<Self>) -> bool {
        let index = self.model.thumbs.iter().position(|thumb| thumb.id == thumb_id);
        if !remove_thumb(&mut self.model, thumb_id) {
            return false;
        }
        if let Some(index) = index
            && index < self.thumb_transitions.len()
        {
            self.thumb_transitions.remove(index);
        }
        self.sync_transitions_with_thumbs();

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

    fn constrained_position_for_raw_position(&self, raw_position: f32) -> f32 {
        let raw_position = raw_position.clamp(0.0, 1.0);
        let raw_value = value_for_position(&self.model, raw_position);
        let constrained = constrain_primary_value(raw_value, &self.model);

        // Preserve the live drag position when it still maps to the constrained value.
        // Mirrored ring mappings intentionally have multiple valid positions for one value.
        if (raw_value - constrained).abs() <= 0.001 {
            raw_position
        } else {
            position_for_value(&self.model, constrained)
        }
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

        if self.set_thumb_value_internal(thumb_id, next, true, true, cx) {
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

        if self.set_thumb_value_internal(thumb_id, value, true, true, cx) {
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

    fn emit_focus_changed(&mut self, focused: bool, cx: &mut Context<Self>) -> bool {
        if self.emitted_focused == focused {
            return false;
        }

        self.emitted_focused = focused;
        cx.emit(SliderEvent::FocusChanged { focused });
        true
    }

    fn emit_drag_start_if_needed(&mut self, was_pressed: bool, thumb_id: Option<ThumbId>, cx: &mut Context<Self>) {
        if was_pressed || !self.interaction.is_pressed() {
            return;
        }

        if let Some(thumb_id) = thumb_id {
            cx.emit(SliderEvent::DragStart { thumb_id });
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
        if self.focus_in_subscription.is_none() {
            let focus_handle = self.interaction.focus_handle().clone();
            self.focus_in_subscription = Some(cx.on_focus(&focus_handle, window, Self::handle_focus_in));
        }
        if self.focus_out_subscription.is_none() {
            let focus_handle = self.interaction.focus_handle().clone();
            self.focus_out_subscription = Some(cx.on_focus_out(&focus_handle, window, Self::handle_focus_out));
        }

        self.sync_transitions_with_thumbs();
        let mut was_animating = false;
        let mut is_animating = false;
        for transition in &mut self.thumb_transitions {
            was_animating |= transition.is_animating();
            is_animating |= transition.sync();
        }
        self.refresh_display_thumbs();
        for transition in &self.thumb_transitions {
            transition.schedule_frame(window, cx);
        }
        if was_animating || is_animating {
            cx.notify();
        }

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

fn transition_duration(animated: bool) -> Duration {
    if animated {
        DEFAULT_TRANSITION_DURATION
    } else {
        Duration::ZERO
    }
}

fn sync_thumb_previews(model: &mut SliderModel) {
    let thumb_ids: Vec<_> = model.thumbs.iter().map(|thumb| thumb.id).collect();
    for thumb_id in thumb_ids {
        sync_thumb_preview(model, thumb_id);
    }
}

fn sync_thumb_preview(model: &mut SliderModel, thumb_id: ThumbId) {
    let Some(renderer) = model.domain_track.as_ref() else {
        return;
    };

    let position = model.thumbs.iter().find(|thumb| thumb.id == thumb_id).map(|thumb| thumb.position);
    let Some(position) = position else {
        return;
    };

    if let Some(color) = renderer.get_color_at_position(position)
        && let Some(thumb) = model.thumbs.iter_mut().find(|thumb| thumb.id == thumb_id)
    {
        thumb.preview = Some(color);
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
