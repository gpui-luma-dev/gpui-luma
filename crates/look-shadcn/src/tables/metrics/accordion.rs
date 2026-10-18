//! Shared accordion metric resolution.

use gpui_luma::theme::{InteractionState, ThemeMode};
use crate::{LookContext, ResolvedMetric, ShadcnModeTokens};

#[derive(Clone, Debug)]
pub struct AccordionMetricTable {
    pub disabled_opacity: ResolvedMetric,
    pub drag_handle_padding: ResolvedMetric,
    pub trigger_height: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub content_padding_y: ResolvedMetric,
    pub radius: ResolvedMetric,
    pub item_gap: ResolvedMetric,
    pub inner_gap: ResolvedMetric,
    pub icon_size: ResolvedMetric,
    pub chevron_size: ResolvedMetric,
}

pub fn resolve_accordion_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: gpui_luma::theme::ControlSize,
    scale_factor: f32,
) -> AccordionMetricTable {
    use gpui_luma::controls::accordion::AccordionScale;

    use crate::catalog::SpacingField;
    use super::helpers::{control_size_key, derived_metric, radius_metric, scaffold_control_metric, spacing_control_metric};

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let scale = AccordionScale::compute_with_stylesheet(size, metrics, scale_factor, &mode.stylesheet().common);
    let size_key = control_size_key(size);

    let mut table = AccordionMetricTable {
        disabled_opacity: derived_metric("configured disabled trigger opacity", scale.disabled_opacity),
        drag_handle_padding: derived_metric("configured drag handle padding", scale.drag_handle_padding),
        trigger_height: scaffold_control_metric(size_key, "control_height", scale.trigger_height),
        padding_x: spacing_control_metric(catalog, size, SpacingField::PaddingX, scale.padding_x),
        padding_y: spacing_control_metric(catalog, size, SpacingField::PaddingY, scale.padding_y),
        content_padding_y: derived_metric(
            format!("{size_key} content padding y from configured scale"),
            scale.content_padding_y,
        ),
        radius: radius_metric(catalog, size, scale.radius),
        item_gap: derived_metric("accordion item gap", scale.item_gap),
        inner_gap: spacing_control_metric(catalog, size, SpacingField::Gap, scale.inner_gap),
        icon_size: derived_metric("accordion icon size", scale.icon_size),
        chevron_size: derived_metric("accordion chevron size", scale.chevron_size),
    };
    let geometry = mode.stylesheet().common.accordion.resolve_geometry(
        size_key,
        gpui_luma::theme::stylesheet::AccordionGeometry {
            trigger_height: scale.trigger_height,
            padding_x: scale.padding_x,
            padding_y: scale.padding_y,
            content_padding_y: scale.content_padding_y,
            radius: scale.radius,
            item_gap: scale.item_gap,
            inner_gap: scale.inner_gap,
            icon_size: scale.icon_size,
            chevron_size: scale.chevron_size,
            ..Default::default()
        },
    );
    table.trigger_height = super::helpers::prefer_shared_metric(geometry.trigger_height, table.trigger_height);
    table.trigger_height.value_px = scale.trigger_height;
    table.padding_x = super::helpers::prefer_shared_metric(geometry.padding_x, table.padding_x);
    table.padding_x.value_px = scale.padding_x;
    table.padding_y = super::helpers::prefer_shared_metric(geometry.padding_y, table.padding_y);
    table.padding_y.value_px = scale.padding_y;
    table.content_padding_y = super::helpers::prefer_shared_metric(geometry.content_padding_y, table.content_padding_y);
    table.content_padding_y.value_px = scale.content_padding_y;
    table.radius = super::helpers::prefer_shared_metric(geometry.radius, table.radius);
    table.radius.value_px = scale.radius;
    table.item_gap = super::helpers::prefer_shared_metric(geometry.item_gap, table.item_gap);
    table.item_gap.value_px = scale.item_gap;
    table.inner_gap = super::helpers::prefer_shared_metric(geometry.inner_gap, table.inner_gap);
    table.inner_gap.value_px = scale.inner_gap;
    table.icon_size = super::helpers::prefer_shared_metric(geometry.icon_size, table.icon_size);
    table.icon_size.value_px = scale.icon_size;
    table.chevron_size = super::helpers::prefer_shared_metric(geometry.chevron_size, table.chevron_size);
    table.chevron_size.value_px = scale.chevron_size;
    table
}

impl AccordionMetricTable {
    /// Converts the complete resolved table to the SDK layout contract.
    pub fn scale(&self) -> gpui_luma::controls::accordion::AccordionScale {
        gpui_luma::controls::accordion::AccordionScale {
            disabled_opacity: self.disabled_opacity.value_px,
            drag_handle_padding: self.drag_handle_padding.value_px,
            trigger_height: self.trigger_height.value_px,
            padding_x: self.padding_x.value_px,
            padding_y: self.padding_y.value_px,
            content_padding_y: self.content_padding_y.value_px,
            radius: self.radius.value_px,
            item_gap: self.item_gap.value_px,
            inner_gap: self.inner_gap.value_px,
            icon_size: self.icon_size.value_px,
            chevron_size: self.chevron_size.value_px,
        }
    }
}
