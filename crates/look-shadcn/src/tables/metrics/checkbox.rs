//! Shared checkbox metric resolution.

use luma::theme::{InteractionState, ThemeMode};
use crate::{LookContext, ResolvedMetric, ShadcnModeTokens};

#[derive(Clone, Debug)]
pub struct CheckboxMetricTable {
    pub height: ResolvedMetric,
    pub gap: ResolvedMetric,
    pub indicator_size: ResolvedMetric,
    pub indicator_radius: ResolvedMetric,
    pub glyph_size: ResolvedMetric,
    pub control_radius: ResolvedMetric,
    pub border_width: ResolvedMetric,
    pub focus_ring_width: ResolvedMetric,
    pub focus_ring_offset: ResolvedMetric,
}

pub fn resolve_checkbox_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: luma::theme::ControlSize,
) -> CheckboxMetricTable {
    use luma::controls::checkbox::CheckboxScale;

    use crate::catalog::SpacingField;
    use super::helpers::{
        border_width_metric, control_size_key, derived_metric, focus_ring_offset_metric, focus_ring_width_metric,
        radius_metric, scaffold_control_metric, spacing_control_metric,
    };

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let scale = CheckboxScale::compute(size, metrics, 1.0);
    let size_key = control_size_key(size);

    CheckboxMetricTable {
        height: scaffold_control_metric(size_key, "control_height", scale.height),
        gap: spacing_control_metric(catalog, size, SpacingField::Gap, scale.gap),
        indicator_size: derived_metric(format!("{size_key} CheckboxScale.indicator_size"), scale.indicator_size),
        indicator_radius: derived_metric(format!("{size_key} CheckboxScale.indicator_radius"), scale.indicator_radius),
        glyph_size: derived_metric(format!("{size_key} glyph = indicator − inset"), scale.glyph_size),
        control_radius: radius_metric(catalog, size, scale.control_radius),
        border_width: border_width_metric(metrics),
        focus_ring_width: focus_ring_width_metric(metrics),
        focus_ring_offset: focus_ring_offset_metric(metrics),
    }
}
