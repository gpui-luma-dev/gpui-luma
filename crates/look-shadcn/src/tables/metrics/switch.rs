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
    use crate::catalog::SpacingField;
    use crate::paint::switch_scale;
    use super::helpers::{
        border_width_metric, control_size_key, derived_metric, focus_ring_offset_metric, focus_ring_width_metric,
        pill_radius_metric, spacing_control_metric,
    };

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let scale = switch_scale(mode, theme_mode, style, size, 1.0);
    let size_key = control_size_key(size);
    let style_key = match style {
        ShadcnButtonStyle::Primary => "primary",
        ShadcnButtonStyle::Secondary => "secondary",
        ShadcnButtonStyle::Outline => "outline",
        ShadcnButtonStyle::Ghost => "ghost",
        ShadcnButtonStyle::ContentOnly => "primary",
    };

    SwitchMetricTable {
        track_width: derived_metric(
            format!("{size_key} {style_key} track width (style.toml switch metrics)"),
            scale.track_width,
        ),
        track_height: derived_metric(
            format!("{size_key} {style_key} track height (style.toml switch metrics)"),
            scale.track_height,
        ),
        track_padding: derived_metric(
            format!("{size_key} {style_key} track padding = height × 2/22"),
            scale.track_padding,
        ),
        thumb_size: derived_metric(
            format!("{size_key} {style_key} thumb (style.toml switch metrics)"),
            scale.thumb_size,
        ),
        gap: spacing_control_metric(catalog, size, SpacingField::Gap, scale.gap),
        track_radius: pill_radius_metric(catalog, scale.track_radius),
        border_width: border_width_metric(metrics),
        focus_ring_width: focus_ring_width_metric(metrics),
        focus_ring_offset: focus_ring_offset_metric(metrics),
    }
}
