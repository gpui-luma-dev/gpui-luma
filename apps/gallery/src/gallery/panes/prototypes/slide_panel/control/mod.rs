mod overlay;
mod state;

pub(in crate::gallery) use overlay::{SlidePanelOverlayHandlers, render_slide_panel_overlay};
pub(in crate::gallery) use state::{SlidePanelEdge, SlidePanelState, SlidePanelTopAnchor};
