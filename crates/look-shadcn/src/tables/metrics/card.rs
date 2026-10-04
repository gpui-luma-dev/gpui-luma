//! Shared card metric resolution.

use gpui_luma::theme::ControlSize;
use crate::{LookContext, ResolvedMetric};
use crate::catalog::SpacingField;
use crate::{ShadcnLook, ShadcnRadius};
use super::helpers::{derived_metric, radius_metric, spacing_control_metric};

#[derive(Clone, Debug)]
pub struct CardMetricTable {
    pub padding: ResolvedMetric,
    pub section_gap: ResolvedMetric,
    pub header_gap: ResolvedMetric,
    pub body_gap: ResolvedMetric,
    pub radius: ResolvedMetric,
}

pub fn resolve_card_metrics(look: &ShadcnLook, size: ControlSize) -> CardMetricTable {
    let theme_mode = look.mode();
    let mode = look.mode_tokens();
    let ctx = LookContext::new(mode.as_ref(), theme_mode, Default::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let base_gap = metrics.gap(size);

    let mut table = CardMetricTable {
        padding: spacing_control_metric(catalog, size, SpacingField::PaddingX, metrics.padding_x(size)),
        section_gap: spacing_control_metric(catalog, size, SpacingField::Gap, base_gap),
        header_gap: derived_metric("card header gap = max(gap / 2, 2px)", (base_gap * 0.5).max(2.0)),
        body_gap: spacing_control_metric(catalog, size, SpacingField::Gap, base_gap),
        radius: radius_metric(catalog, size, look.radius(ShadcnRadius::Lg)),
    };
    let geometry = mode.stylesheet().common.card.resolve_geometry(
        super::helpers::control_size_key(size),
        gpui_luma::theme::stylesheet::CardGeometry {
            padding: table.padding.value_px,
            section_gap: table.section_gap.value_px,
            header_gap: table.header_gap.value_px,
            body_gap: table.body_gap.value_px,
        },
    );
    table.padding = super::helpers::prefer_shared_metric(geometry.padding, table.padding);
    table.section_gap = super::helpers::prefer_shared_metric(geometry.section_gap, table.section_gap);
    table.header_gap = super::helpers::prefer_shared_metric(geometry.header_gap, table.header_gap);
    table.body_gap = super::helpers::prefer_shared_metric(geometry.body_gap, table.body_gap);
    table
}
