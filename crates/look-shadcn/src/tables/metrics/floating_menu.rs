//! Shared floating menu metric resolution.

use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use crate::{LookContext, ShadcnModeTokens};

#[derive(Clone, Debug)]
pub struct FloatingMenuMetricTable {
    pub radius: crate::ResolvedMetric,
    pub padding: crate::ResolvedMetric,
    pub min_width: crate::ResolvedMetric,
    pub item_height: crate::ResolvedMetric,
    pub item_padding_x: crate::ResolvedMetric,
    pub item_gap: crate::ResolvedMetric,
    pub item_icon_size: crate::ResolvedMetric,
    pub item_radius: crate::ResolvedMetric,
    pub submenu_offset_x: crate::ResolvedMetric,
}

pub fn resolve_floating_menu_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> FloatingMenuMetricTable {
    use super::helpers::{control_size_key, derived_metric, scaffold_control_metric, spacing_control_metric};
    use crate::catalog::SpacingField;

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let look = crate::paint::floating_menu_look(mode, theme_mode, size);
    let catalog = ctx.catalog();

    FloatingMenuMetricTable {
        radius: derived_metric("lg = --radius", look.radius),
        padding: derived_metric("padding_y × 0.5", look.padding),
        min_width: derived_metric("floating menu min width", look.min_width),
        item_height: derived_metric("control_height × 0.9", look.item_height),
        item_padding_x: derived_metric("padding_x × 0.75", look.item_padding_x),
        item_gap: spacing_control_metric(catalog, size, SpacingField::Gap, look.item_gap),
        item_icon_size: crate::ResolvedMetric {
            value_px: look.item_icon_size,
            source: crate::MetricSource::Constant {
                label: format!("style.toml [button.metrics.{}].icon_size", control_size_key(size)),
            },
        },
        item_radius: scaffold_control_metric("sm", "radius", look.item_radius),
        submenu_offset_x: derived_metric("gap × 0.5", look.submenu_offset_x),
    }
}
