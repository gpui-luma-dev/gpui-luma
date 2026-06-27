//! Shared oversize/shadow layout policy for indicator-style choice controls.

use gpui::BoxShadow;

use crate::controls::command::button::ButtonRenderModel;
use crate::theme::adorner::{AdornerSpec, adorner_oversize_extent};
use crate::theme::shadow::shadow_projection_insets;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChoiceLayoutPolicy {
    pub elevation: bool,
    pub compact: bool,
}

impl ChoiceLayoutPolicy {
    pub fn from_render_model(model: &ButtonRenderModel<bool>) -> Self {
        Self { elevation: model.elevation, compact: model.compact }
    }
}

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

pub fn indicator_oversize_extent(
    policy: ChoiceLayoutPolicy,
    state_focused: bool,
    palette_adorner: Option<AdornerSpec>,
    focused_probe_adorner: Option<AdornerSpec>,
    shadow_extent: f32,
) -> f32 {
    let focus_extent = if policy.compact {
        if state_focused {
            adorner_oversize_extent(palette_adorner).max(adorner_oversize_extent(focused_probe_adorner))
        } else {
            0.0
        }
    } else {
        adorner_oversize_extent(palette_adorner).max(adorner_oversize_extent(focused_probe_adorner))
    };

    let shadow = if policy.elevation { shadow_extent } else { 0.0 };
    focus_extent.max(shadow)
}

pub fn button_family_oversize_extent(
    compact: bool,
    state_focused: bool,
    adorner: Option<AdornerSpec>,
    focused_adorner: Option<AdornerSpec>,
) -> f32 {
    if compact && !state_focused {
        return 0.0;
    }
    adorner_oversize_extent(adorner).max(adorner_oversize_extent(focused_adorner))
}
