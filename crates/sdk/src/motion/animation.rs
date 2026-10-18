use std::time::{Duration, Instant};
use std::sync::Arc;
use super::{MotionState, RunningMotion, Transition};
use gpui::{Context, Pixels, Window, px};

/// Shared open/closed motion for disclosure affordances and expandable content.
#[derive(Clone, Debug)]
pub struct DisclosureMotion {
    transition: VisualTransition,
    motion_enabled: bool,
    definition: Transition,
}

impl DisclosureMotion {
    pub fn new(initial_progress: f32, motion_enabled: bool) -> Self {
        Self::with_transition(initial_progress, motion_enabled, Transition::default())
    }

    pub fn with_transition(initial_progress: f32, motion_enabled: bool, definition: Transition) -> Self {
        Self {
            transition: VisualTransition::with_transition(
                initial_progress,
                if motion_enabled {
                    definition.clone()
                } else {
                    Transition::instant()
                },
            ),
            motion_enabled,
            definition,
        }
    }

    pub fn set_target(&mut self, target: f32) {
        if self.motion_enabled {
            self.transition.set_target(target);
        } else {
            self.transition.snap_to(target);
        }
    }

    pub fn set_motion_enabled(&mut self, motion_enabled: bool) {
        if self.motion_enabled == motion_enabled {
            return;
        }
        self.motion_enabled = motion_enabled;
        self.transition.set_transition(if motion_enabled {
            self.definition.clone()
        } else {
            Transition::instant()
        });
    }

    pub fn sync(&mut self) -> bool {
        self.transition.sync()
    }

    pub fn progress(&self) -> f32 {
        self.transition.progress()
    }

    pub fn is_animating(&self) -> bool {
        self.transition.is_animating()
    }

    pub fn schedule_frame<T>(&self, window: &mut Window, cx: &mut Context<T>)
    where
        T: 'static,
    {
        self.transition.schedule_frame(window, cx);
    }
}

/// Frame-driven scalar channel. Timed, spring, and custom implementations all
/// run through the same lifecycle. Samples may overshoot the target; clamp only
/// bounded visual properties in templates. Retargeting passes current velocity
/// to the selected implementation.
#[derive(Clone, Debug)]
pub struct VisualTransition {
    progress: f32,
    velocity: f32,
    target: f32,
    definition: Transition,
    running: Option<Arc<dyn RunningMotion>>,
    started_at: Option<Instant>,
    #[cfg(all(test, feature = "test-support"))]
    test_elapsed: Option<Duration>,
}

impl Default for VisualTransition {
    fn default() -> Self {
        Self::with_transition(1.0, Transition::default())
    }
}

impl VisualTransition {
    /// Creates a scalar channel, including positions and widths in logical pixels.
    pub fn with_transition(initial_target: f32, definition: Transition) -> Self {
        let target = if initial_target.is_finite() {
            initial_target
        } else {
            0.0
        };
        Self {
            progress: target,
            velocity: 0.0,
            target,
            definition,
            running: None,
            started_at: None,
            #[cfg(all(test, feature = "test-support"))]
            test_elapsed: None,
        }
    }

    pub fn set_target(&mut self, target: f32) {
        if !target.is_finite() || self.target == target {
            return;
        }
        // Retarget from the last presented sample, avoiding a jump between frames.
        self.target = target;
        self.running = Some(Arc::from(self.definition.start(self.state(), target)));
        self.started_at = Some(Instant::now());
        #[cfg(all(test, feature = "test-support"))]
        if self.test_elapsed.is_some() {
            self.test_elapsed = Some(Duration::ZERO);
        }
        // Instant motion settles synchronously, without scheduling a frame.
        self.sample(Duration::ZERO);
    }

    fn sample(&mut self, elapsed: Duration) -> bool {
        let Some(running) = &self.running else {
            return false;
        };
        let sample = running.sample(elapsed);
        if sample.complete || !sample.state.value.is_finite() || !sample.state.velocity.is_finite() {
            self.snap_to(self.target);
            return false;
        }
        self.progress = sample.state.value;
        self.velocity = sample.state.velocity;
        true
    }

    /// Samples absolute elapsed time from the channel's clock.
    pub fn sync(&mut self) -> bool {
        let Some(started_at) = self.started_at else {
            return false;
        };
        let elapsed = started_at.elapsed();
        #[cfg(all(test, feature = "test-support"))]
        let elapsed = self.test_elapsed.unwrap_or(elapsed);
        self.sample(elapsed)
    }

    /// Changes the configuration while retaining the channel's current target.
    pub fn set_transition(&mut self, definition: Transition) {
        self.definition = definition;
        if self.is_animating() {
            self.running = Some(Arc::from(self.definition.start(self.state(), self.target)));
            self.started_at = Some(Instant::now());
            #[cfg(all(test, feature = "test-support"))]
            if self.test_elapsed.is_some() {
                self.test_elapsed = Some(Duration::ZERO);
            }
            self.sample(Duration::ZERO);
        }
    }

    pub fn state(&self) -> MotionState {
        MotionState { value: self.progress, velocity: self.velocity }
    }

    pub(crate) fn finish(&mut self) {
        self.snap_to(self.target);
    }

    pub fn schedule_frame<T: 'static>(&self, window: &mut Window, cx: &mut Context<T>) {
        if self.is_animating() {
            cx.on_next_frame(window, |_, _, cx| cx.notify());
        }
    }

    #[cfg(all(test, feature = "test-support"))]
    pub(crate) fn elapse_for_test(&mut self, elapsed: Duration) {
        self.test_elapsed = Some(elapsed);
    }

    pub fn progress(&self) -> f32 {
        self.progress
    }
    pub fn is_animating(&self) -> bool {
        self.running.is_some()
    }

    pub fn interpolate(&self, start: f32, end: f32) -> f32 {
        start + ((end - start) * self.progress)
    }

    pub fn interpolate_pixels(&self, start: Pixels, end: Pixels) -> Pixels {
        px(self.interpolate(start.as_f32(), end.as_f32()))
    }

    pub fn snap_to(&mut self, target: f32) {
        if !target.is_finite() {
            return;
        }
        self.progress = target;
        self.velocity = 0.0;
        self.target = target;
        self.started_at = None;
        self.running = None;
    }
}

/// Looping timeline phase for continuous motion (shimmer, indeterminate sweeps).
///
/// Unlike [`VisualTransition`], this does not settle at a target — `phase()` wraps in
/// `0.0`..`1.0` for as long as the phase remains active.
#[derive(Clone, Copy, Debug)]
pub struct ContinuousPhase {
    started_at: Instant,
    period: Duration,
    active: bool,
}

impl Default for ContinuousPhase {
    fn default() -> Self {
        Self::new(crate::theme::MotionDefaults::vanilla().progress.period)
    }
}

impl ContinuousPhase {
    /// Creates an inactive phase with the given loop period.
    pub fn new(period: Duration) -> Self {
        Self { started_at: Instant::now(), period, active: false }
    }

    /// Starts (or restarts) the looping phase.
    pub fn start(&mut self) {
        self.started_at = Instant::now();
        self.active = !self.period.is_zero();
    }

    /// Stops the looping phase.
    pub fn stop(&mut self) {
        self.active = false;
    }

    /// Returns `true` while the phase is driving continuous motion.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Returns the current loop phase in `0.0`..`1.0`.
    pub fn phase(&self) -> f32 {
        if !self.active {
            return 0.0;
        }

        let period_secs = self.period.as_secs_f32();
        if period_secs <= f32::EPSILON {
            return 0.0;
        }

        let elapsed = self.started_at.elapsed().as_secs_f32();
        (elapsed / period_secs).fract().clamp(0.0, 1.0)
    }

    /// Schedules a context re-render on the next window frame while active.
    pub fn schedule_frame<T>(&self, window: &mut Window, cx: &mut Context<T>)
    where
        T: 'static,
    {
        if !self.active {
            return;
        }

        cx.on_next_frame(window, |_, _, cx| {
            cx.notify();
        });
    }
}

pub(crate) fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "test-support")]
    #[test]
    fn explicit_test_clock_ignores_wall_time_and_samples_interpolation() {
        let mut transition = VisualTransition::with_transition(
            0.,
            crate::motion::Transition::timed(Duration::from_secs(1), crate::motion::Easing::OutCubic),
        );
        transition.elapse_for_test(Duration::ZERO);
        transition.set_target(1.);
        // Simulate a heavily delayed runner without sleeping or extending the
        // animation duration. Real elapsed time already exceeds its duration.
        transition.started_at = Instant::now().checked_sub(Duration::from_secs(60));
        assert!(transition.sync());
        assert_eq!(transition.progress(), 0.);
        transition.elapse_for_test(Duration::from_millis(500));
        assert!(transition.sync());
        assert_eq!(transition.progress(), 0.875);
        transition.elapse_for_test(Duration::from_millis(1100));
        assert!(!transition.sync());
        assert_eq!(transition.progress(), 1.);
    }

    #[test]
    fn transition_initial_state() {
        let transition = VisualTransition::with_transition(
            1.0,
            crate::motion::Transition::timed(Duration::from_millis(200), crate::motion::Easing::OutCubic),
        );
        assert_eq!(transition.progress(), 1.0);
        assert!(!transition.is_animating());
    }

    #[test]
    fn transition_start_and_interpolation() {
        let mut transition = VisualTransition::with_transition(
            0.0,
            crate::motion::Transition::timed(Duration::from_millis(200), crate::motion::Easing::OutCubic),
        );
        transition.set_target(1.0);
        assert!(transition.is_animating());
        assert_eq!(transition.interpolate(48.0, 240.0), 48.0);
    }

    #[test]
    fn transition_idempotent_target() {
        let mut transition = VisualTransition::with_transition(
            1.0,
            crate::motion::Transition::timed(Duration::from_millis(200), crate::motion::Easing::OutCubic),
        );
        transition.set_target(1.0);
        assert!(!transition.is_animating());
    }

    #[test]
    fn transition_snap_to_settles() {
        let mut transition = VisualTransition::with_transition(
            0.0,
            crate::motion::Transition::timed(Duration::from_millis(200), crate::motion::Easing::OutCubic),
        );
        transition.set_target(1.0);
        transition.snap_to(0.4);
        assert_eq!(transition.progress(), 0.4);
        assert!(!transition.is_animating());
    }

    #[test]
    fn continuous_phase_inactive_by_default() {
        let phase = ContinuousPhase::default();
        assert!(!phase.is_active());
        assert_eq!(phase.phase(), 0.0);
    }

    #[test]
    fn continuous_phase_wraps_in_unit_interval() {
        let mut phase = ContinuousPhase::new(Duration::from_millis(100));
        phase.start();
        let value = phase.phase();
        assert!(phase.is_active());
        assert!((0.0..=1.0).contains(&value));
    }

    #[test]
    fn continuous_phase_zero_period() {
        let mut phase = ContinuousPhase::new(Duration::ZERO);
        phase.start();
        assert_eq!(phase.phase(), 0.0);
        assert!(!phase.is_active());
    }

    #[test]
    fn continuous_phase_stop() {
        let mut phase = ContinuousPhase::default();
        phase.start();
        phase.stop();
        assert!(!phase.is_active());
        assert_eq!(phase.phase(), 0.0);
    }

    #[test]
    fn disclosure_motion_snaps_when_animation_is_disabled() {
        let mut motion = DisclosureMotion::new(0.0, false);
        motion.set_target(1.0);
        assert_eq!(motion.progress(), 1.0);
        assert!(!motion.is_animating());
    }

    #[test]
    fn disclosure_motion_animates_when_enabled() {
        let mut motion = DisclosureMotion::new(0.0, true);
        motion.set_target(1.0);
        assert!(motion.is_animating());
    }
}
