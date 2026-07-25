//! Shared oversize/shadow layout policy for indicator-style choice controls
//! and button-family hosts (elevation under, adorners over).

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

/// Layout reservation for elevation. Prefer enabled-state shadows so clearing paint
/// when disabled does not shrink the control footprint.
pub fn reserve_shadow_extent(
    current: Option<&Vec<BoxShadow>>,
    enabled_probe: Option<&Vec<BoxShadow>>,
    scale_factor: f32,
    elevation: bool,
) -> f32 {
    let shadows = enabled_probe
        .filter(|shadows| !shadows.is_empty())
        .or_else(|| current.filter(|shadows| !shadows.is_empty()));
    shadow_extent_from(shadows, scale_factor, elevation)
}

pub fn reserve_shadow_extent_from_slice(
    current: &[BoxShadow],
    enabled_probe: Option<&[BoxShadow]>,
    scale_factor: f32,
    elevation: bool,
) -> f32 {
    let shadows = enabled_probe.filter(|shadows| !shadows.is_empty()).unwrap_or(current);
    shadow_extent_from_slice(shadows, scale_factor, elevation)
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
    elevation: bool,
    state_focused: bool,
    adorner: Option<AdornerSpec>,
    focused_adorner: Option<AdornerSpec>,
    shadow_extent: f32,
) -> f32 {
    indicator_oversize_extent(
        ChoiceLayoutPolicy { elevation, compact },
        state_focused,
        adorner,
        focused_adorner,
        shadow_extent,
    )
}

#[cfg(test)]
mod tests {
    use gpui::{black, hsla, point, px, Hsla};

    use super::*;
    use crate::theme::adorner::{AdornerPlacement, FocusRingAdornerSpec};

    fn sample_shadow() -> Vec<BoxShadow> {
        vec![BoxShadow {
            offset: point(px(0.0), px(2.0)),
            blur_radius: px(4.0),
            spread_radius: px(0.0),
            color: Hsla { a: 0.5, ..black() },
            inset: false,
        }]
    }

    fn oversize_focus_adorner() -> AdornerSpec {
        AdornerSpec::FocusRing(FocusRingAdornerSpec {
            color: hsla(0.6, 1.0, 0.5, 1.0),
            placement: AdornerPlacement::Oversize,
            distance: 3.0,
            width: 1.0,
        })
    }

    #[test]
    fn reserve_shadow_prefers_enabled_probe_when_current_cleared() {
        let enabled = sample_shadow();
        let extent = reserve_shadow_extent(None, Some(&enabled), 1.0, true);
        assert!(extent > 0.0);
        assert_eq!(reserve_shadow_extent(None, None, 1.0, true), 0.0);
    }

    #[test]
    fn button_family_extent_keeps_shadow_when_unfocused() {
        let shadows = sample_shadow();
        let shadow_extent = shadow_extent_from(Some(&shadows), 1.0, true);
        let extent =
            button_family_oversize_extent(false, true, false, None, Some(oversize_focus_adorner()), shadow_extent);
        assert_eq!(extent, shadow_extent.max(3.0));
    }

    #[test]
    fn button_family_compact_unfocused_keeps_shadow_reserve_only() {
        let shadows = sample_shadow();
        let shadow_extent = shadow_extent_from(Some(&shadows), 1.0, true);
        let extent =
            button_family_oversize_extent(true, true, false, None, Some(oversize_focus_adorner()), shadow_extent);
        assert_eq!(extent, shadow_extent);
    }

    #[test]
    fn indicator_extent_stable_when_shadow_cleared_if_probe_supplied() {
        let enabled = sample_shadow();
        let enabled_extent = reserve_shadow_extent(None, Some(&enabled), 1.0, true);
        let policy = ChoiceLayoutPolicy { elevation: true, compact: false };
        let with_paint = indicator_oversize_extent(policy, false, None, None, enabled_extent);
        let without_paint = indicator_oversize_extent(
            policy,
            false,
            None,
            None,
            reserve_shadow_extent(None, Some(&enabled), 1.0, true),
        );
        assert_eq!(with_paint, without_paint);
        assert!(with_paint > 0.0);
    }
}
