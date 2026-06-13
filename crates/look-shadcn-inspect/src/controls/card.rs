//! Inspect metadata for `card`.

use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::catalog::SpacingField;
use gpui_luma_look_shadcn::{AppearanceContext, ResolvedMetric, ShadcnLook, ShadcnRadius};

use crate::metrics::{derived_metric, radius_metric, spacing_control_metric};

#[derive(Clone, Debug)]
pub struct CardInspectMetrics {
    pub padding: ResolvedMetric,
    pub section_gap: ResolvedMetric,
    pub header_gap: ResolvedMetric,
    pub body_gap: ResolvedMetric,
    pub radius: ResolvedMetric,
}

pub fn inspect_card_metrics(look: &ShadcnLook, size: ControlSize) -> CardInspectMetrics {
    let theme_mode = look.mode();
    let mode = look.mode_tokens();
    let ctx = AppearanceContext::new(mode.as_ref(), theme_mode, Default::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let base_gap = metrics.gap(size);

    CardInspectMetrics {
        padding: spacing_control_metric(catalog, size, SpacingField::PaddingX, metrics.padding_x(size)),
        section_gap: spacing_control_metric(catalog, size, SpacingField::Gap, base_gap),
        header_gap: derived_metric("card header gap = max(gap / 2, 2px)", (base_gap * 0.5).max(2.0)),
        body_gap: spacing_control_metric(catalog, size, SpacingField::Gap, base_gap),
        radius: radius_metric(catalog, size, look.radius(ShadcnRadius::Lg)),
    }
}
