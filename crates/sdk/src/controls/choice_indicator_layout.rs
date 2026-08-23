//! Shared shadow layout helpers for indicator-style choice controls.

use gpui::BoxShadow;

use crate::theme::shadow::shadow_projection_insets;

/// Returns the largest projected shadow edge for diagnostics and visualization.
pub fn shadow_projection_extent(shadows: Option<&[BoxShadow]>, scale_factor: f32, elevation: bool) -> f32 {
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

pub fn should_paint_shadow(elevation: bool, disabled: bool, has_shadows: bool) -> bool {
    !disabled && elevation && has_shadows
}

#[cfg(test)]
mod tests {
    use super::shadow_projection_extent;
    use crate::theme::tokens::LumaShadowLayer;
    use gpui::Hsla;

    #[test]
    fn shadow_projection_extent_reports_projection_without_layout_reservation() {
        let shadows = vec![LumaShadowLayer::new(Hsla::default(), 0.0, 4.0, 10.0, -2.0).to_box_shadow()];

        assert_eq!(shadow_projection_extent(Some(&shadows), 1.0, true), 12.0);
        assert_eq!(shadow_projection_extent(Some(&shadows), 1.0, false), 0.0);
        assert_eq!(shadow_projection_extent(None, 1.0, true), 0.0);
    }
}
