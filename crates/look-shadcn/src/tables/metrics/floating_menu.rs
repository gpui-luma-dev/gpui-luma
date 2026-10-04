//! Shared floating menu metric resolution.

use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use crate::{LookContext, ShadcnModeTokens};

#[derive(Clone, Debug)]
pub struct FloatingMenuMetricTable {
    pub radius: crate::ResolvedMetric,
    pub padding: crate::ResolvedMetric,
    pub min_width: crate::ResolvedMetric,
    pub item_height: crate::ResolvedMetric,
    pub item_padding_x: crate::ResolvedMetric,
    pub item_gap: crate::ResolvedMetric,
    pub item_icon_size: crate::ResolvedMetric,
    pub item_radius: crate::ResolvedMetric,
    pub submenu_offset_x: crate::ResolvedMetric,
}

pub fn resolve_floating_menu_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> FloatingMenuMetricTable {
    use super::helpers::{derived_metric, scaffold_control_metric, spacing_control_metric};
    use crate::catalog::SpacingField;

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let look = crate::paint::floating_menu_look(mode, theme_mode, size);
    let catalog = ctx.catalog();

    let mut table = FloatingMenuMetricTable {
        radius: derived_metric("lg = --radius", look.radius),
        padding: derived_metric("padding_y × 0.5", look.padding),
        min_width: derived_metric("floating menu min width", look.min_width),
        item_height: derived_metric("control_height × 0.9", look.item_height),
        item_padding_x: derived_metric("padding_x × 0.75", look.item_padding_x),
        item_gap: spacing_control_metric(catalog, size, SpacingField::Gap, look.item_gap),
        item_icon_size: super::helpers::inspect_shared_metric(
            crate::controls::button::resolve_family_geometry(&ctx, mode.stylesheet(), size, false, 1.0)
                .0
                .icon_size,
        ),
        item_radius: scaffold_control_metric("sm", "radius", look.item_radius),
        submenu_offset_x: derived_metric("gap × 0.5", look.submenu_offset_x),
    };
    let geometry = mode.stylesheet().common.floating_menu.resolve_geometry(
        super::helpers::control_size_key(size),
        gpui_luma::theme::stylesheet::FloatingMenuGeometry {
            padding: table.padding.value_px,
            min_width: table.min_width.value_px,
            item_height: table.item_height.value_px,
            item_padding_x: table.item_padding_x.value_px,
            item_gap: table.item_gap.value_px,
            item_icon_size: table.item_icon_size.value_px,
            submenu_offset_x: table.submenu_offset_x.value_px,
            ..Default::default()
        },
    );
    table.padding = super::helpers::prefer_shared_metric(geometry.padding, table.padding);
    table.min_width = super::helpers::prefer_shared_metric(geometry.min_width, table.min_width);
    table.item_height = super::helpers::prefer_shared_metric(geometry.item_height, table.item_height);
    table.item_padding_x = super::helpers::prefer_shared_metric(geometry.item_padding_x, table.item_padding_x);
    table.item_gap = super::helpers::prefer_shared_metric(geometry.item_gap, table.item_gap);
    table.item_icon_size = super::helpers::prefer_shared_metric(geometry.item_icon_size, table.item_icon_size);
    table.submenu_offset_x = super::helpers::prefer_shared_metric(geometry.submenu_offset_x, table.submenu_offset_x);

    table
}
