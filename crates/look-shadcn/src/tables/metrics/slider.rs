//! Shared slider metric resolution.

use luma::theme::{ControlSize, ThemeMode};
use crate::ShadcnModeTokens;

#[derive(Clone, Debug)]
pub struct SliderMetricTable {
    pub width: crate::ResolvedMetric,
    pub height: crate::ResolvedMetric,
    pub track_height: crate::ResolvedMetric,
    pub thumb_size: crate::ResolvedMetric,
    pub radius: crate::ResolvedMetric,
}

/// Default medium slider geometry retained for existing inspection callers.
pub fn resolve_slider_metrics(mode: &ShadcnModeTokens, theme_mode: ThemeMode) -> SliderMetricTable {
    resolve_slider_metrics_for_size(mode, theme_mode, ControlSize::Md)
}

pub fn resolve_slider_metrics_for_size(
    mode: &ShadcnModeTokens,
    _theme_mode: ThemeMode,
    size: ControlSize,
) -> SliderMetricTable {
    use crate::stylesheet::{embedded_stylesheet, resolve_slider_metrics};
    use super::helpers::derived_metric;
    let rule = embedded_stylesheet().slider.metrics_for_size(size);
    let (width, height, track_height, thumb_size, radius) = rule
        .map(|rule| {
            let metrics = resolve_slider_metrics(rule, &mode.metrics, size);
            (metrics.width, metrics.height, metrics.track_height, metrics.thumb_size, metrics.radius)
        })
        .unwrap_or((260.0, 32.0, 8.0, 18.0, mode.metrics.radius.pill));
    let size_key = super::helpers::control_size_key(size);
    let metric = |field: &str, value| {
        let source = if rule.is_some() {
            format!("style.toml [slider.metrics.{size_key}].{field}")
        } else {
            format!("slider default {field}")
        };
        derived_metric(source, value)
    };
    SliderMetricTable {
        width: metric("width", width),
        height: metric("height", height),
        track_height: metric("track_height", track_height),
        thumb_size: metric("thumb_size", thumb_size),
        radius: metric("radius", radius),
    }
}
