//! Shared toolbar metric resolution.

use gpui_luma::theme::{ControlSize, ThemeMode};
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

    let mut table = ToolbarMetricTable {
        radius: derived_metric("toolbar radius = radius.md", metrics.radius.md),
        padding_x: derived_metric("toolbar padding x = 6px", 6.0),
        padding_y: derived_metric("toolbar padding y = 4px", 4.0),
        gap: derived_metric(format!("{size_key} toolbar gap = control gap"), control.gap),
        separator_height: scaffold_control_metric(size_key, "height", control.height),
    };
    let geometry = mode.stylesheet().common.toolbar.resolve_geometry(
        super::helpers::control_size_key(size),
        gpui_luma::theme::stylesheet::ToolbarGeometry {
            padding_x: table.padding_x.value_px,
            padding_y: table.padding_y.value_px,
            gap: table.gap.value_px,
        },
    );
    table.padding_x = super::helpers::prefer_shared_metric(geometry.padding_x, table.padding_x);
    table.padding_y = super::helpers::prefer_shared_metric(geometry.padding_y, table.padding_y);
    table.gap = super::helpers::prefer_shared_metric(geometry.gap, table.gap);
    table
}
