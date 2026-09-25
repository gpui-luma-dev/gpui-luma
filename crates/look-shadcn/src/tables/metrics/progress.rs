//! Shared progress metric resolution.

use luma::theme::ThemeMode;
use crate::ShadcnModeTokens;

#[derive(Clone, Debug)]
pub struct ProgressMetricTable {
    pub size: crate::ResolvedMetric,
    pub stroke_width: crate::ResolvedMetric,
    pub track_height: crate::ResolvedMetric,
    pub thumb_size: crate::ResolvedMetric,
}

pub fn resolve_progress_metrics(mode: &ShadcnModeTokens, _theme_mode: ThemeMode) -> ProgressMetricTable {
    use super::helpers::derived_metric;

    let look = crate::paint::progress_look(mode, true, luma::theme::ControlSize::Md);

    ProgressMetricTable {
        size: derived_metric("progress ring diameter", look.size),
        stroke_width: derived_metric("progress stroke width", look.stroke_width),
        track_height: derived_metric("progress track height", look.track_height),
        thumb_size: derived_metric("progress thumb size", look.thumb_size),
    }
}
