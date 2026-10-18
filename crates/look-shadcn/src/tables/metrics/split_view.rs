//! Shared split view metric resolution.

use crate::ResolvedMetric;

#[derive(Clone, Debug)]
pub struct SplitViewMetricTable {
    pub separator_hitbox_width: ResolvedMetric,
    pub separator_cue_width: ResolvedMetric,
    pub separator_cue_hovered_width: ResolvedMetric,
    pub separator_cue_radius: ResolvedMetric,
    pub separator_cue_inset_y: ResolvedMetric,
}

pub fn resolve_split_view_metrics() -> SplitViewMetricTable {
    resolve_split_view_metrics_with_stylesheet(gpui_luma::theme::stylesheet::CommonStylesheet::sdk_default())
}

/// Resolve separator inspection from the same selected stylesheet used for paint.
pub fn resolve_split_view_metrics_with_stylesheet(
    stylesheet: &gpui_luma::theme::stylesheet::CommonStylesheet,
) -> SplitViewMetricTable {
    use super::helpers::derived_metric;
    let geometry = stylesheet.split_view.resolve_geometry("", Default::default());

    SplitViewMetricTable {
        separator_hitbox_width: derived_metric("split view separator hitbox width", geometry.hitbox_width.value_px),
        separator_cue_width: derived_metric("split view separator cue width", geometry.cue_width.value_px),
        separator_cue_hovered_width: derived_metric(
            "split view separator cue hovered width",
            geometry.cue_hovered_width.value_px,
        ),
        separator_cue_radius: derived_metric("split view separator cue radius", geometry.cue_radius.value_px),
        separator_cue_inset_y: derived_metric("split view separator cue inset y", geometry.cue_inset_y.value_px),
    }
}
