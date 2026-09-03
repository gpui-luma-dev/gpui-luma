//! Edge-anchored drawer overlay (app-owned state + templates).
//!
//! Not a spawnable LMTP entity — there is no `SlidePanel` control. Layout:
//! - [`model`] — [`SlidePanelState`] / edge / size config
//! - [`template`] — overlay render plus resize-handle chrome (`template/handle`, `template/resize`)
//!
//! Apps own [`SlidePanelState`] and call [`render_slide_panel_overlay`].

mod model;
mod template;

pub use model::{SlidePanelEdge, SlidePanelSizeConfig, SlidePanelState, SlidePanelTopAnchor};
pub use template::{
    SlidePanelOverlayHandlers, SlidePanelResizeDrag, SlidePanelResizeHandlers, SlidePanelResizeHitHandlers,
    render_slide_panel_inner_resize_handle, render_slide_panel_inset, render_slide_panel_overlay,
    render_slide_panel_resize_handle,
};
