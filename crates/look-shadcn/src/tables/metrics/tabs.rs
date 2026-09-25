//! Shared tabs metric resolution.

use luma::theme::{ControlSize, InteractionState, ThemeMode};
use crate::{LookContext, ShadcnModeTokens};

#[derive(Clone, Debug)]
pub struct TabsMetricTable {
    pub list_radius: crate::ResolvedMetric,
    pub list_gap: crate::ResolvedMetric,
    pub list_padding: crate::ResolvedMetric,
    pub item_padding_x: crate::ResolvedMetric,
    pub item_height: crate::ResolvedMetric,
    pub item_radius: crate::ResolvedMetric,
    pub indicator_height: crate::ResolvedMetric,
}

pub fn resolve_tabs_metrics(mode: &ShadcnModeTokens, theme_mode: ThemeMode, size: ControlSize) -> TabsMetricTable {
    use super::helpers::{derived_metric, radius_metric, spacing_control_metric};
    use crate::catalog::SpacingField;

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let catalog = ctx.catalog();
    let list = crate::paint::tabs_list_look(mode, true, size);
    let item = crate::paint::tabs_item_look(mode, true, InteractionState::default(), size);

    TabsMetricTable {
        list_radius: radius_metric(catalog, size, list.radius),
        list_gap: derived_metric("spacing.s5", list.gap),
        list_padding: derived_metric("list padding", list.padding),
        item_padding_x: spacing_control_metric(catalog, size, SpacingField::PaddingX, item.padding_x),
        item_height: derived_metric("label line_height + spacing.s2", item.height),
        item_radius: radius_metric(catalog, size, item.radius),
        indicator_height: derived_metric("active tab indicator height", item.indicator_height),
    }
}
