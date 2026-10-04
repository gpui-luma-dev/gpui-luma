//! Shared control group metric resolution.

use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use crate::{LookContext, ResolvedMetric, ShadcnModeTokens};

#[derive(Clone, Debug)]
pub struct ControlGroupMetricTable {
    pub radius: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub gap: ResolvedMetric,
}

pub fn resolve_control_group_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> ControlGroupMetricTable {
    use super::helpers::{derived_metric, radius_metric};

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let catalog = ctx.catalog();
    let look = crate::paint::control_group_list_look(mode, true);

    let mut table = ControlGroupMetricTable {
        radius: radius_metric(catalog, size, look.radius),
        padding_x: derived_metric("control group padding x", look.padding_x),
        padding_y: derived_metric("control group padding y", look.padding_y),
        gap: derived_metric("control group gap", look.gap),
    };
    let geometry = mode.stylesheet().common.control_group.resolve_geometry(
        "md",
        gpui_luma::theme::stylesheet::ControlGroupGeometry {
            padding_x: table.padding_x.value_px,
            padding_y: table.padding_y.value_px,
            gap: table.gap.value_px,
        },
    );
    table.padding_x = super::helpers::prefer_shared_metric(geometry.padding_x, table.padding_x);
    table.padding_y = super::helpers::prefer_shared_metric(geometry.padding_y, table.padding_y);
    table.gap = super::helpers::prefer_shared_metric(geometry.gap, table.gap);

    table
}
