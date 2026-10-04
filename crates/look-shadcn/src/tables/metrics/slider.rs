//! Shared slider metric resolution.

use gpui_luma::theme::{ControlSize, ThemeMode};
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
    resolve_slider_metrics_with_stylesheet(mode, mode.stylesheet(), size, None)
}

/// Shared resolved dimensions; legacy metrics remain below common authored fields.
pub(crate) fn slider_geometry_with_stylesheet(
    mode: &ShadcnModeTokens,
    stylesheet: &crate::stylesheet::StylesheetConfig,
    size: ControlSize,
    thumb_size: Option<gpui_luma::controls::slider::SliderThumbSize>,
) -> (gpui_luma::theme::stylesheet::ResolvedSliderGeometry, f32) {
    use gpui_luma::controls::slider::{DefaultSliderTheme, SliderTheme, SliderThumbSize};
    let theme = DefaultSliderTheme::new(gpui_luma::theme::ThemeTokens { metrics: mode.metrics, ..Default::default() });
    let mut fallback = theme.resolve(size, thumb_size, Default::default());
    if let Some(rule) = stylesheet.slider.metrics_for_size(size) {
        let metrics = crate::stylesheet::resolve_slider_metrics(rule, &mode.metrics, size);
        fallback.width = metrics.width;
        fallback.height = metrics.height;
        fallback.track_height = metrics.track_height;
        fallback.radius = metrics.radius;
        if thumb_size.is_none() {
            fallback.thumb_size = metrics.thumb_size;
        }
    }
    let mut legacy_thumb_source = None;
    if let Some(thumb_size) = thumb_size {
        let (thumb_control_size, common_thumb) = match thumb_size {
            SliderThumbSize::Sm => {
                (ControlSize::Sm, stylesheet.common.slider.sizes.get("sm").and_then(|geometry| geometry.thumb_size))
            }
            SliderThumbSize::Md => {
                (ControlSize::Md, stylesheet.common.slider.sizes.get("md").and_then(|geometry| geometry.thumb_size))
            }
            SliderThumbSize::Lg => {
                (ControlSize::Lg, stylesheet.common.slider.sizes.get("lg").and_then(|geometry| geometry.thumb_size))
            }
        };
        if let Some(rule) = stylesheet.slider.metrics_for_size(thumb_control_size) {
            fallback.thumb_size = rule.thumb_size;
            if common_thumb.is_none() && stylesheet.common.slider.geometry.thumb_size.is_none() {
                let key = super::helpers::control_size_key(thumb_control_size);
                legacy_thumb_source =
                    Some(format!("instance thumb_size {key} → style.toml · slider.metrics.{key}.thumb_size"));
            }
        }
    }
    let thumb_key = thumb_size.map(|size| match size {
        SliderThumbSize::Sm => "sm",
        SliderThumbSize::Md => "md",
        SliderThumbSize::Lg => "lg",
    });
    let mut geometry =
        stylesheet
            .common
            .slider
            .resolve_geometry(super::helpers::control_size_key(size), thumb_key, &fallback);
    if let Some(note) = legacy_thumb_source {
        geometry.thumb_size.source = gpui_luma::theme::provenance::MetricSource::Derived { note };
    }
    (geometry, fallback.radius)
}

pub fn resolve_slider_metrics_with_stylesheet(
    mode: &ShadcnModeTokens,
    stylesheet: &crate::stylesheet::StylesheetConfig,
    size: ControlSize,
    thumb_size: Option<gpui_luma::controls::slider::SliderThumbSize>,
) -> SliderMetricTable {
    use super::helpers::{derived_metric, inspect_shared_metric, pill_radius_metric};
    let (geometry, radius) = slider_geometry_with_stylesheet(mode, stylesheet, size, thumb_size);
    let size_key = super::helpers::control_size_key(size);
    let legacy = stylesheet.slider.metrics_for_size(size).is_some();
    let metric = |field: &str, value: gpui_luma::theme::provenance::ResolvedMetric| {
        if legacy && matches!(value.source, gpui_luma::theme::provenance::MetricSource::Constant { .. }) {
            derived_metric(format!("style.toml · slider.metrics.{size_key}.{field}"), value.value_px)
        } else {
            inspect_shared_metric(value)
        }
    };
    SliderMetricTable {
        width: metric("width", geometry.width),
        height: metric("height", geometry.height),
        track_height: metric("track_height", geometry.track_height),
        thumb_size: metric("thumb_size", geometry.thumb_size),
        radius: if legacy {
            derived_metric(format!("style.toml · slider.metrics.{size_key}.radius"), radius)
        } else {
            pill_radius_metric(&mode.catalog, radius)
        },
    }
}
