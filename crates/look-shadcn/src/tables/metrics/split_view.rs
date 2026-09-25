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
    use super::helpers::derived_metric;
    use luma::controls::split_view::*;

    SplitViewMetricTable {
        separator_hitbox_width: derived_metric("split view separator hitbox width", SEPARATOR_HITBOX_WIDTH),
        separator_cue_width: derived_metric("split view separator cue width", SEPARATOR_CUE_WIDTH),
        separator_cue_hovered_width: derived_metric(
            "split view separator cue hovered width",
            SEPARATOR_CUE_HOVERED_WIDTH,
        ),
        separator_cue_radius: derived_metric("split view separator cue radius", SEPARATOR_CUE_RADIUS),
        separator_cue_inset_y: derived_metric("split view separator cue inset y", SEPARATOR_CUE_INSET_Y),
    }
}
