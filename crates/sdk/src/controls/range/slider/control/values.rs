use gpui::Context;

use super::super::model::{constrain_primary_value, position_for_value, value_for_position, ThumbId};
use super::super::thumbs::{remove_thumb, set_thumb_position};
use super::super::{input, layout, step_allowed_value};
use super::motion::sync_thumb_preview;
use super::{SliderControl, SliderEvent};

impl SliderControl {
    pub(super) fn handle_position_step_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        let modifiers = event.keystroke.modifiers;
        if !self.model.enabled
            || !self.interaction.focus_handle().is_focused(window)
            || !modifiers.shift
            || modifiers.control
            || modifiers.platform
            || modifiers.alt
            || modifiers.function
        {
            return;
        }
        let direction = match event.keystroke.key.as_str() {
            "right" | "up" => 1.0,
            "left" | "down" => -1.0,
            _ => return,
        };
        self.adjust_value(direction * self.model.step * 10.0, cx);
        window.prevent_default();
        cx.stop_propagation();
    }

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

        if let Some(step) = self.model.keyboard_position_step
            && self.model.allowed_intervals.is_empty()
        {
            let Some(thumb) = self.model.thumbs.iter().find(|thumb| thumb.id == thumb_id) else {
                return;
            };
            let next =
                thumb.position + layout::oriented_step_delta(delta / self.model.step * step, self.model.reversed);
            let next = if self.model.wrapping {
                next.rem_euclid(1.0)
            } else {
                next.clamp(0.0, 1.0)
            };
            // Immediate angular steps avoid interpolating the long way across the wrap seam.
            if self.set_thumb_position_internal(thumb_id, next, true, false, cx) {
                let value = self.thumb_value(thumb_id).unwrap_or(self.model.range.start);
                cx.emit(SliderEvent::Release { thumb_id, value });
            }
            return;
        }

        let current = self.thumb_value(thumb_id).unwrap_or(self.model.range.start);
        let next = if self.model.wrapping
            && self.model.allowed_intervals.is_empty()
            && self.model.value_map.as_ref().is_none_or(|map| map.wraps_value())
        {
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

#[cfg(all(test, feature = "test-support"))]
mod keyboard_tests {
    use super::*;
    use gpui::{Focusable, TestAppContext};
    use crate::controls::slider::SliderBuilder;

    #[test]
    fn ordinary_and_value_constrained_sliders_retain_scalar_steps() {
        for constrained in [false, true] {
            let mut app = TestAppContext::single();
            let (slider, cx) = app.add_window_view(|window, cx| {
                window.activate_window();
                crate::key_handling::bind_default_control_keys(cx);
                let mut builder = SliderBuilder::new("scalar").range(0.0..100.0).step(5.0).value(20.0);
                if constrained {
                    builder = builder.keyboard_position_step(0.01).allowed_intervals(vec![20.0..=40.0]);
                }
                SliderControl::from_builder(builder, cx)
            });
            cx.run_until_parked();
            cx.update(|window, app| slider.read(app).focus_handle(app).focus(window, app));
            cx.simulate_keystrokes("right");
            cx.run_until_parked();
            cx.update(|_, app| assert_eq!(slider.read(app).value(), 25.0));
        }
    }
}
