//! Shared sidebar metric resolution.

use luma::theme::{ControlSize, InteractionState, ThemeMode};
use crate::{LookContext, ResolvedMetric, ShadcnModeTokens};
use crate::catalog::SpacingField;

#[derive(Clone, Debug)]
pub struct SidebarMetricTable {
    pub section_height: ResolvedMetric,
    pub item_height: ResolvedMetric,
    pub item_padding_x: ResolvedMetric,
    pub item_gap: ResolvedMetric,
    pub item_radius: ResolvedMetric,
    pub item_icon_size: ResolvedMetric,
}

pub fn resolve_sidebar_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> SidebarMetricTable {
    use super::helpers::{derived_metric, radius_metric, spacing_control_metric};

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let size_metrics = metrics.for_size(size);

    SidebarMetricTable {
        section_height: derived_metric("navigation sidebar section height", 20.0),
        item_height: derived_metric("navigation sidebar item height", 30.0),
        item_padding_x: derived_metric("navigation sidebar item padding x", 8.0),
        item_gap: spacing_control_metric(catalog, size, SpacingField::Gap, size_metrics.gap),
        item_radius: radius_metric(catalog, size, metrics.radius(size)),
        item_icon_size: derived_metric("navigation sidebar icon size", 16.0),
    }
}
