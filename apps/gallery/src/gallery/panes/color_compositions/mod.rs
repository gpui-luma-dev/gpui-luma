use gpui::{AnyElement, IntoElement};
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::panes::color::common::color_gallery_pane;

pub(super) mod compositions;
mod color_harmonies;
mod color_picker_photoshop;
mod hsv_plane_photoshop;
mod hsv_wheel;
mod hue_ring_sv_triangle;
mod multi_mixer;
mod split_ring_pixagram;

pub(in crate::gallery) use color_harmonies::ColorHarmoniesPane;
pub(in crate::gallery) use color_picker_photoshop::ColorPickerPane;
pub(in crate::gallery) use hsv_plane_photoshop::HsvPlanePane;
pub(in crate::gallery) use hsv_wheel::HsvWheelPane;
pub(in crate::gallery) use hue_ring_sv_triangle::SvTrianglePane;
pub(in crate::gallery) use multi_mixer::MultiMixerPane;
pub(in crate::gallery) use split_ring_pixagram::SplitRingPane;

pub(super) fn render_single_composition_page(
    title: &'static str,
    description: &'static str,
    content: impl IntoElement,
    look: &ShadcnLook,
) -> AnyElement {
    color_gallery_pane(title, description, content, look)
}
