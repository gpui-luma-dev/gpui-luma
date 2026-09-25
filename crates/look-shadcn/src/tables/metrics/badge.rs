//! Shared badge metric resolution.

use luma::theme::{ControlSize, ThemeMode};
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

    BadgeMetricTable {
        min_height: derived_metric("badge min height", look.min_height),
        padding_x: spacing_control_metric(&mode.catalog, size, crate::catalog::SpacingField::PaddingX, look.padding_x),
        padding_y: derived_metric("badge vertical inset from typography", look.padding_y),
        gap: spacing_control_metric(&mode.catalog, size, crate::catalog::SpacingField::Gap, look.gap),
        icon_size: derived_metric("badge icon size follows typography size", look.icon_size),
        radius: pill_radius_metric(&mode.catalog, look.radius),
    }
}
