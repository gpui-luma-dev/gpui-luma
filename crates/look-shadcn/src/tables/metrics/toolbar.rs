//! Shared toolbar metric resolution.

use luma::theme::{ControlSize, ThemeMode};
use crate::{LookContext, ResolvedMetric, ShadcnModeTokens};

#[derive(Clone, Debug)]
pub struct ToolbarMetricTable {
    pub radius: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub gap: ResolvedMetric,
    pub separator_height: ResolvedMetric,
}

pub fn resolve_toolbar_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> ToolbarMetricTable {
    use super::helpers::{control_size_key, derived_metric, scaffold_control_metric};

    let ctx = LookContext::new(mode, theme_mode, Default::default());
    let metrics = ctx.metrics();
    let control = metrics.for_size(size);
    let size_key = control_size_key(size);

    ToolbarMetricTable {
        radius: derived_metric("toolbar radius = radius.md", metrics.radius.md),
        padding_x: derived_metric("toolbar padding x = 6px", 6.0),
        padding_y: derived_metric("toolbar padding y = 4px", 4.0),
        gap: derived_metric(format!("{size_key} toolbar gap = control gap"), control.gap),
        separator_height: scaffold_control_metric(size_key, "height", control.height),
    }
}
