//! Styled viewport plus scrollbar host.

mod control;
mod model;
mod template;
pub(crate) use template::AUTO_HIDE_TIMEOUT;

pub use control::ScrollContainer;
pub use model::{ScrollbarAutoHideActivate, ScrollbarPlacement, ScrollbarVisibility};

mod motion;
pub(crate) use motion::ScrollMotion;
