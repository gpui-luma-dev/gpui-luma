//! Shared tree view metric resolution.

use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use crate::{LookContext, ResolvedMetric, ShadcnModeTokens};

#[derive(Clone, Debug)]
pub struct TreeViewMetricTable {
    pub row_height: ResolvedMetric,
    pub base_padding_x: ResolvedMetric,
    pub indentation_width: ResolvedMetric,
    pub inner_gap: ResolvedMetric,
    pub radius: ResolvedMetric,
    pub icon_size: ResolvedMetric,
    pub chevron_size: ResolvedMetric,
}

pub fn resolve_tree_view_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> TreeViewMetricTable {
    use gpui_luma::controls::tree_view::TreeViewScale;

    use crate::catalog::SpacingField;
    use super::helpers::{control_size_key, derived_metric, radius_metric, spacing_control_metric};

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let scale = TreeViewScale::compute_with_stylesheet(size, metrics, 1.0, &mode.stylesheet().common);
    let size_key = control_size_key(size);

    let mut table = TreeViewMetricTable {
        row_height: derived_metric(format!("{size_key} tree row height from configured scale"), scale.row_height),
        base_padding_x: spacing_control_metric(catalog, size, SpacingField::PaddingX, scale.base_padding_x),
        indentation_width: derived_metric("tree indentation width", scale.indentation_width),
        inner_gap: spacing_control_metric(catalog, size, SpacingField::Gap, scale.inner_gap),
        radius: radius_metric(catalog, size, scale.radius),
        icon_size: derived_metric("tree icon size", scale.icon_size),
        chevron_size: derived_metric("tree chevron size", scale.chevron_size),
    };
    let geometry = mode.stylesheet().common.tree_view.resolve_geometry(
        size_key,
        gpui_luma::theme::stylesheet::TreeViewGeometry {
            row_height: scale.row_height,
            base_padding_x: scale.base_padding_x,
            indentation_width: scale.indentation_width,
            inner_gap: scale.inner_gap,
            radius: scale.radius,
            icon_size: scale.icon_size,
            chevron_size: scale.chevron_size,
            ..Default::default()
        },
    );
    table.row_height = super::helpers::prefer_shared_metric(geometry.row_height, table.row_height);
    table.row_height.value_px = scale.row_height;
    table.base_padding_x = super::helpers::prefer_shared_metric(geometry.base_padding_x, table.base_padding_x);
    table.base_padding_x.value_px = scale.base_padding_x;
    table.indentation_width = super::helpers::prefer_shared_metric(geometry.indentation_width, table.indentation_width);
    table.indentation_width.value_px = scale.indentation_width;
    table.inner_gap = super::helpers::prefer_shared_metric(geometry.inner_gap, table.inner_gap);
    table.inner_gap.value_px = scale.inner_gap;
    table.radius = super::helpers::prefer_shared_metric(geometry.radius, table.radius);
    table.radius.value_px = scale.radius;
    table.icon_size = super::helpers::prefer_shared_metric(geometry.icon_size, table.icon_size);
    table.icon_size.value_px = scale.icon_size;
    table.chevron_size = super::helpers::prefer_shared_metric(geometry.chevron_size, table.chevron_size);
    table.chevron_size.value_px = scale.chevron_size;
    table
}
