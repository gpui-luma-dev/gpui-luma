//! Shared textfield metric resolution.

use gpui_luma::theme::{InteractionState, ThemeMode};
use crate::{LookContext, ResolvedMetric, ShadcnModeTokens};

#[derive(Clone, Debug)]
pub struct TextFieldMetricTable {
    pub min_height: ResolvedMetric,
    pub icon_size: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub radius: ResolvedMetric,
    pub border_width: ResolvedMetric,
    pub focus_ring_width: ResolvedMetric,
    pub focus_ring_offset: ResolvedMetric,
}

pub fn resolve_textfield_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: gpui_luma::theme::ControlSize,
) -> TextFieldMetricTable {
    use gpui_luma::theme::StandardBoxScale;

    use super::helpers::{
        border_width_metric, control_size_key, focus_ring_offset_metric, focus_ring_width_metric, radius_metric,
        scaffold_control_metric, spacing_control_metric,
    };
    use crate::catalog::SpacingField;

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let catalog = ctx.catalog();
    let metrics = ctx.metrics();
    let scale = StandardBoxScale::compute(size, metrics, 1.0);

    let geometry = crate::controls::textfield::textfield_geometry(mode, size, &scale, &mode.typography.text.body);
    let mut table = TextFieldMetricTable {
        min_height: scaffold_control_metric(control_size_key(size), "control_height", scale.height),
        icon_size: scaffold_control_metric(control_size_key(size), "icon_size", scale.icon_size),
        padding_x: spacing_control_metric(catalog, size, SpacingField::PaddingX, scale.padding_x),
        padding_y: spacing_control_metric(catalog, size, SpacingField::PaddingY, scale.padding_y),
        radius: radius_metric(catalog, size, scale.radius),
        border_width: border_width_metric(metrics),
        focus_ring_width: focus_ring_width_metric(metrics),
        focus_ring_offset: focus_ring_offset_metric(metrics),
    };
    table.min_height = super::helpers::prefer_shared_metric(geometry.min_height, table.min_height);
    table.icon_size = super::helpers::prefer_shared_metric(geometry.icon_size, table.icon_size);
    table.padding_x = super::helpers::prefer_shared_metric(geometry.padding_x, table.padding_x);
    table.padding_y = super::helpers::prefer_shared_metric(geometry.padding_y, table.padding_y);
    table
}
