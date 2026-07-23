mod handle;
mod overlay;
mod resize;
mod state;

pub(in crate::gallery) use handle::SlidePanelResizeHandlers;
pub(in crate::gallery) use overlay::{SlidePanelOverlayHandlers, render_slide_panel_overlay, slide_panel_panels_look};
pub(in crate::gallery) use resize::SlidePanelResizeDrag;
pub(in crate::gallery) use state::{SlidePanelEdge, SlidePanelSizeConfig, SlidePanelState, SlidePanelTopAnchor};
