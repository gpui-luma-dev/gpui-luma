//! Inspect metadata for `badge`.

use gpui_luma::theme::{ControlSize, ThemeMode};
use gpui_luma_look_shadcn::{BadgeVariant, ResolvedColor, ResolvedMetric, ShadcnLook, ShadcnModeTokens};

use crate::metrics::{derived_metric, pill_radius_metric, spacing_control_metric};

pub struct BadgeInspectPalette {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct BadgeInspectMetrics {
    pub min_height: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub gap: ResolvedMetric,
    pub icon_size: ResolvedMetric,
    pub radius: ResolvedMetric,
}

pub fn inspect_badge_color_palette(look: &ShadcnLook, variant: BadgeVariant) -> BadgeInspectPalette {
    let colors = gpui_luma_look_shadcn::tables::resolve_badge_colors(look, variant)
        .unwrap_or_else(|_| gpui_luma_look_shadcn::BadgeColorTable::fallback(look, variant));
    BadgeInspectPalette { background: colors.background, foreground: colors.foreground, border: colors.border }
}

pub fn inspect_badge_metrics(
    mode: &ShadcnModeTokens,
    look: &ShadcnLook,
    variant: BadgeVariant,
    size: ControlSize,
    _theme_mode: ThemeMode,
) -> BadgeInspectMetrics {
    let appearance = gpui_luma_look_shadcn::badge_appearance(look, variant, size);

    BadgeInspectMetrics {
        min_height: derived_metric("badge min height", appearance.min_height),
        padding_x: spacing_control_metric(
            &mode.catalog,
            size,
            gpui_luma_look_shadcn::catalog::SpacingField::PaddingX,
            appearance.padding_x,
        ),
        padding_y: derived_metric("badge vertical inset from typography", appearance.padding_y),
        gap: spacing_control_metric(
            &mode.catalog,
            size,
            gpui_luma_look_shadcn::catalog::SpacingField::Gap,
            appearance.gap,
        ),
        icon_size: derived_metric("badge icon size follows typography size", appearance.icon_size),
        radius: pill_radius_metric(metrics_catalog(mode), appearance.radius),
    }
}

fn metrics_catalog(mode: &ShadcnModeTokens) -> &gpui_luma_look_shadcn::catalog::CssTokenMap {
    &mode.catalog
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_luma_look_shadcn::{ColorSource, ShadcnToken};

    #[test]
    fn inspect_default_badge_uses_primary_fill() {
        let look = ShadcnLook::native();
        let palette = inspect_badge_color_palette(&look, BadgeVariant::Default);
        assert_eq!(palette.background.value, look.color(ShadcnToken::Primary));
        assert!(matches!(
            palette.background.source,
            ColorSource::CssVar { ref token } if token == "primary"
        ));
    }

    #[test]
    fn inspect_badge_metrics_use_pill_radius() {
        let look = ShadcnLook::native();
        let mode = look.mode_tokens();
        let metrics =
            inspect_badge_metrics(mode.as_ref(), &look, BadgeVariant::Default, ControlSize::Md, ThemeMode::Light);
        assert!(metrics.radius.value_px >= mode.metrics.radius.pill - f32::EPSILON);
    }
}
