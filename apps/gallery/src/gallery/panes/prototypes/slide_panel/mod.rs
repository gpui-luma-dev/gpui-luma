mod control;
mod pane;

pub(in crate::gallery) use control::{
    SlidePanelEdge, SlidePanelOverlayHandlers, SlidePanelResizeDrag, SlidePanelResizeHandlers, SlidePanelSizeConfig,
    SlidePanelState, SlidePanelTopAnchor, render_slide_panel_inset, slide_panel_panels_look,
};
pub(in crate::gallery) use pane::SlidePanelPane;
