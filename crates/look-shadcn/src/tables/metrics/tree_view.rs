//! Shared tree view metric resolution.

use luma::theme::{ControlSize, InteractionState, ThemeMode};
use crate::{LookContext, ResolvedMetric, ShadcnModeTokens};

#[derive(Clone, Debug)]
pub struct TreeViewMetricTable {
    pub row_height: ResolvedMetric,
    pub base_padding_x: ResolvedMetric,
    pub indentation_width: ResolvedMetric,
    pub inner_gap: ResolvedMetric,
    pub radius: ResolvedMetric,
    pub icon_size: ResolvedMetric,
    pub chevron_size: ResolvedMetric,
}

pub fn resolve_tree_view_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> TreeViewMetricTable {
    use luma::controls::tree_view::TreeViewScale;

    use crate::catalog::SpacingField;
    use super::helpers::{control_size_key, derived_metric, radius_metric, spacing_control_metric};

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let scale = TreeViewScale::compute(size, metrics, 1.0);
    let size_key = control_size_key(size);

    TreeViewMetricTable {
        row_height: derived_metric(format!("{size_key} tree row height = control_height × 0.85"), scale.row_height),
        base_padding_x: spacing_control_metric(catalog, size, SpacingField::PaddingX, scale.base_padding_x),
        indentation_width: derived_metric("tree indentation width", scale.indentation_width),
        inner_gap: spacing_control_metric(catalog, size, SpacingField::Gap, scale.inner_gap),
        radius: radius_metric(catalog, size, scale.radius),
        icon_size: derived_metric("tree icon size", scale.icon_size),
        chevron_size: derived_metric("tree chevron size", scale.chevron_size),
    }
}
