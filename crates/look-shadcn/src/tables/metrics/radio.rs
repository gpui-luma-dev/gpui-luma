//! Shared radio metric resolution.

use gpui_luma::theme::{InteractionState, ThemeMode};
use crate::{LookContext, ResolvedMetric, ShadcnModeTokens};

#[derive(Clone, Debug)]
pub struct RadioButtonMetricTable {
    pub height: ResolvedMetric,
    pub gap: ResolvedMetric,
    pub indicator_size: ResolvedMetric,
    pub dot_size: ResolvedMetric,
    pub control_radius: ResolvedMetric,
    pub border_width: ResolvedMetric,
    pub focus_ring_width: ResolvedMetric,
    pub focus_ring_offset: ResolvedMetric,
}

pub fn resolve_radio_button_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: gpui_luma::theme::ControlSize,
) -> RadioButtonMetricTable {
    resolve_radio_button_metrics_with_stylesheet(mode, mode.stylesheet(), theme_mode, size)
}

pub(crate) fn resolve_radio_button_metrics_with_stylesheet(
    mode: &ShadcnModeTokens,
    stylesheet: &crate::stylesheet::StylesheetConfig,
    theme_mode: ThemeMode,
    size: gpui_luma::theme::ControlSize,
) -> RadioButtonMetricTable {
    use crate::catalog::SpacingField;
    use super::helpers::{
        border_width_metric, control_size_key, derived_metric, focus_ring_offset_metric, focus_ring_width_metric,
        radius_metric, scaffold_control_metric, spacing_control_metric,
    };

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let (scale, geometry) = crate::controls::radio::radio_scale_with_stylesheet(&ctx, stylesheet, size, 1.0);
    let authored = |metric: gpui_luma::theme::provenance::ResolvedMetric, fallback: ResolvedMetric| {
        if matches!(metric.source, gpui_luma::theme::provenance::MetricSource::Constant { .. }) {
            fallback
        } else {
            super::helpers::inspect_shared_metric(metric)
        }
    };
    let size_key = control_size_key(size);

    RadioButtonMetricTable {
        height: scaffold_control_metric(size_key, "control_height", scale.height),
        gap: authored(geometry.gap, spacing_control_metric(catalog, size, SpacingField::Gap, scale.gap)),
        indicator_size: authored(
            geometry.indicator_size,
            derived_metric(format!("{size_key} RadioScale.indicator_size"), scale.indicator_size),
        ),
        dot_size: authored(
            geometry.dot_size,
            derived_metric(format!("{size_key} RadioScale.dot_size"), scale.dot_size),
        ),
        control_radius: radius_metric(catalog, size, scale.control_radius),
        border_width: border_width_metric(metrics),
        focus_ring_width: focus_ring_width_metric(metrics),
        focus_ring_offset: focus_ring_offset_metric(metrics),
    }
}
