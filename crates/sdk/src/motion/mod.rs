//! Shared frame-driven motion: transitions, overlay presence, and popup lifecycle.

mod animation;
pub mod overlay_presence;
pub mod popup_lifecycle;

pub use animation::{
    ContinuousPhase, DEFAULT_CONTINUOUS_PERIOD, DEFAULT_TRANSITION_DURATION, DisclosureMotion, VisualTransition,
};
pub use overlay_presence::{OVERLAY_ENTER_SCALE_MIN, OverlayPresence, overlay_enter_offset, overlay_enter_scale};
pub use popup_lifecycle::PopupLifecycle;
