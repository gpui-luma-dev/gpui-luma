//! Shared stepper metric resolution.

use gpui_luma::theme::ThemeMode;
use crate::ShadcnModeTokens;

#[derive(Clone, Debug)]
pub struct StepperMetricTable {
    pub step_badge_size: crate::ResolvedMetric,
    pub track_thickness: crate::ResolvedMetric,
}

pub fn resolve_stepper_metrics(mode: &ShadcnModeTokens, theme_mode: ThemeMode) -> StepperMetricTable {
    resolve_stepper_metrics_for_size(mode, theme_mode, gpui_luma::theme::ControlSize::Md)
}

/// Resolve the selected look's Stepper dimensions and their sources for a size.
pub fn resolve_stepper_metrics_for_size(
    mode: &ShadcnModeTokens,
    _theme_mode: ThemeMode,
    size: gpui_luma::theme::ControlSize,
) -> StepperMetricTable {
    use super::helpers::derived_metric;

    let look = crate::paint::stepper_look(mode, true, size);

    let mut table = StepperMetricTable {
        step_badge_size: derived_metric("stepper badge size", look.step_badge_size),
        track_thickness: derived_metric("stepper track thickness", look.track_thickness),
    };
    let geometry = mode.stylesheet().common.stepper.resolve_geometry(
        super::helpers::control_size_key(size),
        gpui_luma::theme::stylesheet::StepperGeometry {
            step_badge_size: table.step_badge_size.value_px,
            track_thickness: table.track_thickness.value_px,
        },
    );
    table.step_badge_size = super::helpers::prefer_shared_metric(geometry.step_badge_size, table.step_badge_size);
    table.track_thickness = super::helpers::prefer_shared_metric(geometry.track_thickness, table.track_thickness);

    table
}
