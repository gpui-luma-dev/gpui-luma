//! Inspect metadata for `badge`.

use luma::theme::{ControlSize, ThemeMode};
use luma_look_shadcn::{BadgeVariant, ResolvedColor, ResolvedMetric, ShadcnLook, ShadcnModeTokens, ShadcnSize};

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
    let colors = luma_look_shadcn::tables::resolve_badge_colors(look, variant)
        .unwrap_or_else(|_| luma_look_shadcn::BadgeColorTable::fallback(look, variant));
    BadgeInspectPalette { background: colors.background, foreground: colors.foreground, border: colors.border }
}

pub fn inspect_badge_metrics(
    mode: &ShadcnModeTokens,
    look: &ShadcnLook,
    variant: BadgeVariant,
    size: ControlSize,
    _theme_mode: ThemeMode,
) -> BadgeInspectMetrics {
    let look = luma_look_shadcn::badge_look(
        look,
        variant,
        match size {
            ControlSize::Sm => ShadcnSize::Sm,
            ControlSize::Md => ShadcnSize::Md,
            ControlSize::Lg => ShadcnSize::Lg,
        },
    );

    BadgeInspectMetrics {
        min_height: derived_metric("badge min height", look.min_height),
        padding_x: spacing_control_metric(
            &mode.catalog,
            size,
            luma_look_shadcn::catalog::SpacingField::PaddingX,
            look.padding_x,
        ),
        padding_y: derived_metric("badge vertical inset from typography", look.padding_y),
        gap: spacing_control_metric(&mode.catalog, size, luma_look_shadcn::catalog::SpacingField::Gap, look.gap),
        icon_size: derived_metric("badge icon size follows typography size", look.icon_size),
        radius: pill_radius_metric(metrics_catalog(mode), look.radius),
    }
}

fn metrics_catalog(mode: &ShadcnModeTokens) -> &luma_look_shadcn::catalog::CssTokenMap {
    &mode.catalog
}

#[cfg(test)]
mod tests {
    use super::*;
    use luma_look_shadcn::{ColorSource, ShadcnLook, ShadcnToken};

    #[test]
    fn inspect_default_badge_uses_primary_fill() {
        let look = ShadcnLook::from_css_str(include_str!("../../tests/fixtures/native.css"))
            .expect("native CSS fixture should parse");
        let palette = inspect_badge_color_palette(&look, BadgeVariant::Default);
        assert_eq!(palette.background.value, look.color(ShadcnToken::Primary));
        assert!(matches!(
            palette.background.source,
            ColorSource::CssVar { ref token } if token == "primary"
        ));
    }

    #[test]
    fn inspect_badge_metrics_use_pill_radius() {
        let look = ShadcnLook::from_css_str(include_str!("../../tests/fixtures/native.css"))
            .expect("native CSS fixture should parse");
        let mode = look.mode_tokens();
        let metrics =
            inspect_badge_metrics(mode.as_ref(), &look, BadgeVariant::Default, ControlSize::Md, ThemeMode::Light);
        assert!(metrics.radius.value_px >= mode.metrics.radius.pill - f32::EPSILON);
    }
}
