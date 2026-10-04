//! Shared switch metric resolution.

use gpui_luma::theme::{InteractionState, ThemeMode};
use crate::{LookContext, ResolvedMetric, ShadcnModeTokens};
use crate::ShadcnButtonStyle;

#[derive(Clone, Debug)]
pub struct SwitchMetricTable {
    pub track_width: ResolvedMetric,
    pub track_height: ResolvedMetric,
    pub track_padding: ResolvedMetric,
    pub thumb_size: ResolvedMetric,
    pub gap: ResolvedMetric,
    pub track_radius: ResolvedMetric,
    pub border_width: ResolvedMetric,
    pub focus_ring_width: ResolvedMetric,
    pub focus_ring_offset: ResolvedMetric,
}

pub fn resolve_switch_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    size: gpui_luma::theme::ControlSize,
) -> SwitchMetricTable {
    resolve_switch_metrics_with_stylesheet(mode, mode.stylesheet(), theme_mode, style, size)
}

pub fn resolve_switch_metrics_with_stylesheet(
    mode: &ShadcnModeTokens,
    stylesheet: &crate::stylesheet::StylesheetConfig,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    size: gpui_luma::theme::ControlSize,
) -> SwitchMetricTable {
    use crate::catalog::SpacingField;
    use super::helpers::{
        border_width_metric, control_size_key, derived_metric, focus_ring_offset_metric, focus_ring_width_metric,
        pill_radius_metric, spacing_control_metric,
    };

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let scale =
        crate::controls::switch::switch_scale_with_stylesheet(mode, stylesheet, theme_mode, style, size, None, 1.0);
    let geometry = crate::controls::switch::switch_geometry_with_stylesheet(mode, stylesheet, size, 1.0);
    let size_key = control_size_key(size);
    let legacy = stylesheet.switch.metrics_for_size(size).is_some();
    let dimension = |field: &str, value| {
        let note = if legacy {
            format!("style.toml · switch.metrics.{size_key}.{field}")
        } else {
            format!("SDK switch {field} fallback")
        };
        derived_metric(note, value)
    };
    let mut table = SwitchMetricTable {
        track_width: dimension("width", scale.track_width),
        track_height: dimension("height", scale.track_height),
        track_padding: derived_metric("SDK switch track padding fallback", scale.track_padding),
        thumb_size: dimension("thumb_size", scale.thumb_size),
        gap: spacing_control_metric(catalog, size, SpacingField::Gap, scale.gap),
        track_radius: pill_radius_metric(catalog, scale.track_radius),
        border_width: border_width_metric(metrics),
        focus_ring_width: focus_ring_width_metric(metrics),
        focus_ring_offset: focus_ring_offset_metric(metrics),
    };
    let authored = |common: gpui_luma::theme::provenance::ResolvedMetric, fallback: ResolvedMetric| {
        if matches!(common.source, gpui_luma::theme::provenance::MetricSource::Authored { .. }) {
            super::helpers::inspect_shared_metric(common)
        } else {
            fallback
        }
    };
    table.track_width = authored(geometry.track_width, table.track_width);
    table.track_height = authored(geometry.track_height, table.track_height);
    table.track_padding = authored(geometry.track_padding, table.track_padding);
    table.thumb_size = authored(geometry.thumb_size, table.thumb_size);
    table.gap = authored(geometry.gap, table.gap);
    table
}
