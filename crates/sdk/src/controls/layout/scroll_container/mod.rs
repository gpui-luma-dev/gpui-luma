//! Styled viewport plus scrollbar host.

mod control;
mod model;
mod template;

pub use control::ScrollContainer;
pub use model::{ScrollbarAutoHideActivate, ScrollbarPlacement, ScrollbarVisibility};

mod motion;
pub(crate) use motion::ScrollMotion;
