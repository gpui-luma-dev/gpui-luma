//! Shared shadow layout helpers for indicator-style choice controls.

use gpui::BoxShadow;

use crate::theme::shadow::shadow_projection_insets;

pub fn shadow_extent_from(shadows: Option<&Vec<BoxShadow>>, scale_factor: f32, elevation: bool) -> f32 {
    if !elevation {
        return 0.0;
    }
    shadows
        .filter(|shadows| !shadows.is_empty())
        .map(|shadows| {
            let insets = shadow_projection_insets(shadows, scale_factor);
            insets.top.max(insets.right).max(insets.bottom).max(insets.left)
        })
        .unwrap_or(0.0)
}

pub fn shadow_extent_from_slice(shadows: &[BoxShadow], scale_factor: f32, elevation: bool) -> f32 {
    if !elevation || shadows.is_empty() {
        return 0.0;
    }
    let insets = shadow_projection_insets(shadows, scale_factor);
    insets.top.max(insets.right).max(insets.bottom).max(insets.left)
}

pub fn should_paint_shadow(elevation: bool, disabled: bool, has_shadows: bool) -> bool {
    !disabled && elevation && has_shadows
}
