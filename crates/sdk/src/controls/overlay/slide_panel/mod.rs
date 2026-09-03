//! Edge-anchored drawer overlay. Not a spawnable LMTP entity — apps own
//! [`SlidePanelState`] and render via [`render_slide_panel_overlay`].

mod handle;
mod model;
mod resize;
mod template;

pub use handle::{
    SlidePanelResizeHandlers, SlidePanelResizeHitHandlers, render_slide_panel_inner_resize_handle,
    render_slide_panel_resize_handle,
};
pub use template::{SlidePanelOverlayHandlers, render_slide_panel_inset, render_slide_panel_overlay};
pub use resize::SlidePanelResizeDrag;
pub use model::{SlidePanelEdge, SlidePanelSizeConfig, SlidePanelState, SlidePanelTopAnchor};
