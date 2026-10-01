//! Shared accordion metric resolution.

use gpui_luma::theme::{InteractionState, ThemeMode};
use crate::{LookContext, ResolvedMetric, ShadcnModeTokens};

#[derive(Clone, Debug)]
pub struct AccordionMetricTable {
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
    let scale = AccordionScale::compute(size, metrics, scale_factor);
    let size_key = control_size_key(size);

    AccordionMetricTable {
        trigger_height: scaffold_control_metric(size_key, "control_height", scale.trigger_height),
        padding_x: spacing_control_metric(catalog, size, SpacingField::PaddingX, scale.padding_x),
        padding_y: spacing_control_metric(catalog, size, SpacingField::PaddingY, scale.padding_y),
        content_padding_y: derived_metric(
            format!("{size_key} content padding y = padding_y × 1.5"),
            scale.content_padding_y,
        ),
        radius: radius_metric(catalog, size, scale.radius),
        item_gap: derived_metric("accordion item gap", scale.item_gap),
        inner_gap: spacing_control_metric(catalog, size, SpacingField::Gap, scale.inner_gap),
        icon_size: derived_metric("accordion icon size", scale.icon_size),
        chevron_size: derived_metric("accordion chevron size", scale.chevron_size),
    }
}

impl AccordionMetricTable {
    /// Converts the complete resolved table to the SDK layout contract.
    pub fn scale(&self) -> gpui_luma::controls::accordion::AccordionScale {
        gpui_luma::controls::accordion::AccordionScale {
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
