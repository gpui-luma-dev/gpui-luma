//! Shared context menu metric resolution.

use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use crate::{LookContext, ShadcnModeTokens};

#[derive(Clone, Debug)]
pub struct ContextMenuMetricTable {
    pub target_padding_x: crate::ResolvedMetric,
    pub target_padding_y: crate::ResolvedMetric,
    pub target_radius: crate::ResolvedMetric,
    pub target_min_width: crate::ResolvedMetric,
    pub menu: super::FloatingMenuMetricTable,
}

pub fn resolve_context_menu_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> ContextMenuMetricTable {
    use super::helpers::{derived_metric, radius_metric, spacing_control_metric};
    use crate::catalog::SpacingField;

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let look = crate::paint::context_menu_look(mode, theme_mode, InteractionState::default());
    let catalog = ctx.catalog();

    ContextMenuMetricTable {
        target_padding_x: spacing_control_metric(catalog, size, SpacingField::PaddingX, look.target_padding_x),
        target_padding_y: spacing_control_metric(catalog, size, SpacingField::PaddingY, look.target_padding_y),
        target_radius: radius_metric(catalog, size, look.target_radius),
        target_min_width: derived_metric("context menu target min width", look.target_min_width),
        menu: super::resolve_floating_menu_metrics(mode, theme_mode, size),
    }
}
