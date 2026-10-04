//! Shared progress metric resolution.

use gpui_luma::theme::ThemeMode;
use crate::ShadcnModeTokens;

#[derive(Clone, Debug)]
pub struct ProgressMetricTable {
    pub size: crate::ResolvedMetric,
    pub stroke_width: crate::ResolvedMetric,
    pub track_height: crate::ResolvedMetric,
    pub thumb_size: crate::ResolvedMetric,
}

pub fn resolve_progress_metrics(mode: &ShadcnModeTokens, theme_mode: ThemeMode) -> ProgressMetricTable {
    resolve_progress_metrics_for_size(mode, theme_mode, gpui_luma::theme::ControlSize::Md)
}

/// Resolve the selected look's Progress dimensions and their sources for a size.
pub fn resolve_progress_metrics_for_size(
    mode: &ShadcnModeTokens,
    _theme_mode: ThemeMode,
    size: gpui_luma::theme::ControlSize,
) -> ProgressMetricTable {
    use super::helpers::derived_metric;

    let look = crate::paint::progress_look(mode, true, size);

    let mut table = ProgressMetricTable {
        size: derived_metric("progress ring diameter", look.size),
        stroke_width: derived_metric("progress stroke width", look.stroke_width),
        track_height: derived_metric("progress track height", look.track_height),
        thumb_size: derived_metric("progress thumb size", look.thumb_size),
    };
    let geometry = mode.stylesheet().common.progress.resolve_geometry(
        super::helpers::control_size_key(size),
        gpui_luma::theme::stylesheet::ProgressGeometry {
            size: table.size.value_px,
            stroke_width: table.stroke_width.value_px,
            track_height: table.track_height.value_px,
            thumb_size: table.thumb_size.value_px,
        },
    );
    table.size = super::helpers::prefer_shared_metric(geometry.size, table.size);
    table.stroke_width = super::helpers::prefer_shared_metric(geometry.stroke_width, table.stroke_width);
    table.track_height = super::helpers::prefer_shared_metric(geometry.track_height, table.track_height);
    table.thumb_size = super::helpers::prefer_shared_metric(geometry.thumb_size, table.thumb_size);

    table
}
