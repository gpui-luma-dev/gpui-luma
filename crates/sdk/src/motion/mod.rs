//! Shared frame-driven motion: transitions, overlay presence, and popup lifecycle.

mod animation;
mod drop_layout;
pub use drop_layout::DropLayoutMotion;
pub mod overlay_presence;
pub mod popup_lifecycle;

pub use animation::{ContinuousPhase, DisclosureMotion, VisualTransition};
pub use overlay_presence::{
    PresenceMotion, OVERLAY_ENTER_SCALE_MIN, OverlayPresence, overlay_enter_offset, overlay_enter_scale,
};
pub use popup_lifecycle::PopupLifecycle;

pub(crate) use animation::ease_out_cubic;

mod transition;
pub use transition::{
    Easing, MotionDefinition, MotionOverrides, MotionSample, MotionState, RunningMotion, Transition,
    TransitionDescription,
};

mod config;
pub use config::MotionConfig;

#[cfg(feature = "test-support")]
mod test_support;
