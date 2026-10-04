//! Shared badge metric resolution.

use gpui_luma::theme::{ControlSize, ThemeMode};
use crate::{ResolvedMetric, ShadcnModeTokens};
use crate::{BadgeVariant, ShadcnLook, ShadcnSize};
use super::helpers::{derived_metric, pill_radius_metric, spacing_control_metric};

#[derive(Clone, Debug)]
pub struct BadgeMetricTable {
    pub min_height: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub gap: ResolvedMetric,
    pub icon_size: ResolvedMetric,
    pub radius: ResolvedMetric,
}

pub fn resolve_badge_metrics(
    mode: &ShadcnModeTokens,
    look: &ShadcnLook,
    variant: BadgeVariant,
    size: ControlSize,
    _theme_mode: ThemeMode,
) -> BadgeMetricTable {
    let look = crate::badge_look(
        look,
        variant,
        match size {
            ControlSize::Sm => ShadcnSize::Sm,
            ControlSize::Md => ShadcnSize::Md,
            ControlSize::Lg => ShadcnSize::Lg,
        },
    );

    let mut table = BadgeMetricTable {
        min_height: derived_metric("badge min height", look.min_height),
        padding_x: spacing_control_metric(&mode.catalog, size, crate::catalog::SpacingField::PaddingX, look.padding_x),
        padding_y: derived_metric("badge vertical inset from typography", look.padding_y),
        gap: spacing_control_metric(&mode.catalog, size, crate::catalog::SpacingField::Gap, look.gap),
        icon_size: derived_metric("badge icon size follows typography size", look.icon_size),
        radius: pill_radius_metric(&mode.catalog, look.radius),
    };
    let geometry = mode.stylesheet().common.badge.resolve_geometry(
        super::helpers::control_size_key(size),
        gpui_luma::theme::stylesheet::BadgeGeometry {
            min_height: table.min_height.value_px,
            padding_x: table.padding_x.value_px,
            padding_y: table.padding_y.value_px,
            gap: table.gap.value_px,
            icon_size: table.icon_size.value_px,
            ..Default::default()
        },
    );
    table.min_height = super::helpers::prefer_shared_metric(geometry.min_height, table.min_height);
    table.padding_x = super::helpers::prefer_shared_metric(geometry.padding_x, table.padding_x);
    table.padding_y = super::helpers::prefer_shared_metric(geometry.padding_y, table.padding_y);
    table.gap = super::helpers::prefer_shared_metric(geometry.gap, table.gap);
    table.icon_size = super::helpers::prefer_shared_metric(geometry.icon_size, table.icon_size);

    table
}
