mod arc_pane;
pub(super) mod common;
mod field_pane;
mod ring_pane;
mod slider_pane;
mod slider_revealed_pane;

pub(in crate::gallery) use arc_pane::ColorArcPane;
pub(in crate::gallery) use field_pane::ColorFieldPane;
pub(in crate::gallery) use ring_pane::ColorRingPane;
pub(in crate::gallery) use slider_pane::ColorSliderPane;
pub(in crate::gallery) use slider_revealed_pane::ColorSliderRevealedPane;
