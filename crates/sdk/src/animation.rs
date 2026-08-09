use std::time::{Duration, Instant};
use gpui::{Context, Pixels, Window, px};

/// Default duration for visual UI transitions (200 milliseconds).
pub const DEFAULT_TRANSITION_DURATION: Duration = Duration::from_millis(200);

/// Lightweight frame-driven state transition primitive for GPUI controls.
///
/// `VisualTransition` manages progress interpolation (`0.0`..`1.0`), cubic ease-out curve
/// calculations, and frame scheduling via `cx.on_next_frame`.
#[derive(Clone, Copy, Debug)]
pub struct VisualTransition {
    progress: f32,
    from: f32,
    target: f32,
    started_at: Option<Instant>,
    duration: Duration,
}

impl Default for VisualTransition {
    fn default() -> Self {
        Self::new(1.0, DEFAULT_TRANSITION_DURATION)
    }
}

impl VisualTransition {
    /// Creates a new transition initialized at `initial_target` (typically `0.0` or `1.0`).
    pub fn new(initial_target: f32, duration: Duration) -> Self {
        let target = initial_target.clamp(0.0, 1.0);
        Self { progress: target, from: target, target, started_at: None, duration }
    }

    /// Sets a new target value (`0.0`..`1.0`) and initiates a transition if different.
    pub fn set_target(&mut self, target: f32) {
        let target = target.clamp(0.0, 1.0);
        if (self.target - target).abs() <= f32::EPSILON {
            return;
        }

        self.from = self.progress;
        self.target = target;
        self.started_at = Some(Instant::now());
    }

    /// Updates internal progress based on elapsed time since animation start.
    /// Returns `true` if actively animating.
    pub fn sync(&mut self) -> bool {
        let Some(started_at) = self.started_at else {
            return false;
        };

        let elapsed_secs = started_at.elapsed().as_secs_f32();
        let duration_secs = self.duration.as_secs_f32();

        if duration_secs <= f32::EPSILON || elapsed_secs >= duration_secs {
            self.progress = self.target;
            self.started_at = None;
            return false;
        }

        let linear = (elapsed_secs / duration_secs).clamp(0.0, 1.0);
        let eased = ease_out_cubic(linear);
        self.progress = self.from + ((self.target - self.from) * eased);
        true
    }

    /// Schedules a context re-render notification on the next window frame if animating.
    pub fn schedule_frame<T>(&self, window: &mut Window, cx: &mut Context<T>)
    where
        T: 'static,
    {
        if self.started_at.is_none() {
            return;
        }

        cx.on_next_frame(window, |_, _, cx| {
            cx.notify();
        });
    }

    /// Returns current progress in range `0.0`..`1.0`.
    pub fn progress(&self) -> f32 {
        self.progress
    }

    /// Returns `true` if a transition is currently in progress.
    pub fn is_animating(&self) -> bool {
        self.started_at.is_some()
    }

    /// Linearly interpolates progress between `start` and `end` scalar values.
    pub fn interpolate(&self, start: f32, end: f32) -> f32 {
        start + ((end - start) * self.progress)
    }

    /// Linearly interpolates progress between `start` and `end` pixel bounds.
    pub fn interpolate_pixels(&self, start: Pixels, end: Pixels) -> Pixels {
        px(self.interpolate(start.as_f32(), end.as_f32()))
    }
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transition_initial_state() {
        let transition = VisualTransition::new(1.0, Duration::from_millis(200));
        assert_eq!(transition.progress(), 1.0);
        assert!(!transition.is_animating());
    }

    #[test]
    fn transition_start_and_interpolation() {
        let mut transition = VisualTransition::new(0.0, Duration::from_millis(200));
        transition.set_target(1.0);
        assert!(transition.is_animating());
        assert_eq!(transition.interpolate(48.0, 240.0), 48.0);
    }

    #[test]
    fn transition_idempotent_target() {
        let mut transition = VisualTransition::new(1.0, Duration::from_millis(200));
        transition.set_target(1.0);
        assert!(!transition.is_animating());
    }
}
