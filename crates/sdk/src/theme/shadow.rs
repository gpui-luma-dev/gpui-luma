use gpui::{BoxShadow, Div, SharedString, Stateful, div, px, prelude::*};

use crate::theme::scales::{ShadowProjectionInsets, snap_to_pixel};

pub fn shadow_projection_insets(shadows: &[BoxShadow], scale_factor: f32) -> ShadowProjectionInsets {
    shadows.iter().fold(ShadowProjectionInsets::default(), |acc, shadow| {
        let offset_x = snap_to_pixel(shadow.offset.x.as_f32(), scale_factor);
        let offset_y = snap_to_pixel(shadow.offset.y.as_f32(), scale_factor);
        let blur = snap_to_pixel(shadow.blur_radius.as_f32(), scale_factor);
        let spread = snap_to_pixel(shadow.spread_radius.as_f32(), scale_factor);
        acc.union(ShadowProjectionInsets::compute(offset_x, offset_y, blur, spread))
    })
}

pub fn render_shadow_backing(
    id: impl Into<SharedString>,
    shadows: &[BoxShadow],
    radius: f32,
    _scale_factor: f32,
) -> Stateful<Div> {
    div()
        .id(format!("{}-shadow", id.into()))
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .rounded(px(radius))
        .shadow(shadows.to_vec())
}
