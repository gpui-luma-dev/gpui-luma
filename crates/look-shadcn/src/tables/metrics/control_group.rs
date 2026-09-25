//! Shared control group metric resolution.

use luma::theme::{ControlSize, InteractionState, ThemeMode};
use crate::{LookContext, ResolvedMetric, ShadcnModeTokens};

#[derive(Clone, Debug)]
pub struct ControlGroupMetricTable {
    pub radius: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub gap: ResolvedMetric,
}

pub fn resolve_control_group_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> ControlGroupMetricTable {
    use super::helpers::{derived_metric, radius_metric};

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let catalog = ctx.catalog();
    let look = crate::paint::control_group_list_look(mode, true);

    ControlGroupMetricTable {
        radius: radius_metric(catalog, size, look.radius),
        padding_x: derived_metric("control group padding x", look.padding_x),
        padding_y: derived_metric("control group padding y", look.padding_y),
        gap: derived_metric("control group gap", look.gap),
    }
}
