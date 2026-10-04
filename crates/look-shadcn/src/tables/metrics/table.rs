//! Shared table metric resolution.

use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use crate::{LookContext, ResolvedMetric, ShadcnModeTokens};

#[derive(Clone, Debug)]
pub struct TableMetricTable {
    pub radius: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub row_min_height: ResolvedMetric,
    pub row_padding_x: ResolvedMetric,
    pub row_padding_y: ResolvedMetric,
}

pub fn resolve_table_metrics(mode: &ShadcnModeTokens, theme_mode: ThemeMode, size: ControlSize) -> TableMetricTable {
    use gpui_luma::theme::ListRowScale;

    use crate::catalog::SpacingField;
    use super::helpers::{control_size_key, derived_metric, radius_metric, scaffold_control_metric, spacing_control_metric};

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let look = crate::paint::table_look(mode, true, false, size);
    let row_scale = ListRowScale::compute(size, metrics, 1.0);
    let size_key = control_size_key(size);

    let mut table = TableMetricTable {
        radius: radius_metric(catalog, size, look.radius),
        padding_x: derived_metric("list padding x", look.padding_x),
        padding_y: spacing_control_metric(catalog, size, SpacingField::PaddingY, look.padding_y),
        row_min_height: scaffold_control_metric(size_key, "control_height", row_scale.min_height),
        row_padding_x: spacing_control_metric(catalog, size, SpacingField::PaddingX, row_scale.padding_x),
        row_padding_y: spacing_control_metric(catalog, size, SpacingField::PaddingY, row_scale.padding_y),
    };
    let geometry = mode.stylesheet().common.table.resolve_geometry(
        size_key,
        gpui_luma::theme::stylesheet::TableGeometry {
            padding_x: table.padding_x.value_px,
            padding_y: table.padding_y.value_px,
        },
    );
    table.padding_x = super::helpers::prefer_shared_metric(geometry.padding_x, table.padding_x);
    table.padding_y = super::helpers::prefer_shared_metric(geometry.padding_y, table.padding_y);

    table
}
