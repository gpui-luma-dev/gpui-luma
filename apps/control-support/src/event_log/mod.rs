//! Shared event history, extracted from Luma Studio. Look adapters live in their apps.
mod control;
mod model;
mod template;
mod theme;
pub use control::EventLogView;
pub use model::EventLogViewBuilder;
pub use theme::{EventLogTheme, EventLogLook, compose_event_log_look};
