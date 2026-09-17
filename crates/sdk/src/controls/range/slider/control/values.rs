use gpui::Context;

use super::super::model::{constrain_primary_value, position_for_value, value_for_position, ThumbId};
use super::super::thumbs::{remove_thumb, set_thumb_position};
use super::super::{input, layout, step_allowed_value};
use super::motion::sync_thumb_preview;
use super::{SliderControl, SliderEvent};

impl SliderControl {
    pub(super) fn select_thumb(&mut self, thumb_id: ThumbId, emit: bool, cx: &mut Context<Self>) -> bool {
        if self.active_thumb_id == Some(thumb_id) {
            return false;
        }

        self.active_thumb_id = Some(thumb_id);
        if emit {
            cx.emit(SliderEvent::ThumbSelected { thumb_id });
        }
        true
    }

    pub(super) fn set_thumb_value_internal(
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

    pub(super) fn set_thumb_position_internal(
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
    pub(super) fn remove_thumb(&mut self, thumb_id: ThumbId, cx: &mut Context<Self>) -> bool {
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
    pub(super) fn active_or_primary_thumb_id(&self) -> Option<ThumbId> {
        self.active_thumb_id.or_else(|| self.model.thumbs.first().map(|thumb| thumb.id))
    }

    pub(super) fn constrained_position_for_raw_position(&self, raw_position: f32) -> f32 {
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

    pub(super) fn adjust_value(&mut self, delta: f32, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        let Some(thumb_id) = self.active_or_primary_thumb_id() else {
            return;
        };

        let current = self.thumb_value(thumb_id).unwrap_or(self.model.range.start);
        let next = if self.model.wrapping && self.model.allowed_intervals.is_empty() {
            input::wrap_and_snap(
                current + layout::oriented_step_delta(delta, self.model.reversed),
                self.model.range,
                self.model.step,
            )
        } else {
            step_allowed_value(
                current,
                layout::oriented_step_delta(delta, self.model.reversed),
                &self.model.allowed_intervals,
                self.model.range,
                self.model.step,
            )
        };

        if self.set_thumb_value_internal(thumb_id, next, true, true, cx) {
            cx.emit(SliderEvent::Release { thumb_id, value: next });
        }
    }

    pub(super) fn move_to_value(&mut self, value: f32, cx: &mut Context<Self>) {
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
}
