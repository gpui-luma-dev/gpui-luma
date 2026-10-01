//! Shared stepper metric resolution.

use gpui_luma::theme::{ControlSize, ThemeMode};
use crate::ShadcnModeTokens;

#[derive(Clone, Debug)]
pub struct StepperMetricTable {
    pub step_badge_size: crate::ResolvedMetric,
    pub track_thickness: crate::ResolvedMetric,
}

pub fn resolve_stepper_metrics(mode: &ShadcnModeTokens, _theme_mode: ThemeMode) -> StepperMetricTable {
    use super::helpers::derived_metric;

    let look = crate::paint::stepper_look(mode, true, ControlSize::Md);

    StepperMetricTable {
        step_badge_size: derived_metric("stepper badge size", look.step_badge_size),
        track_thickness: derived_metric("stepper track thickness", look.track_thickness),
    }
}
