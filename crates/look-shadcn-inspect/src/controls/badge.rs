//! Inspect metadata for `badge`.

use luma::theme::{ControlSize, ThemeMode};
use luma_look_shadcn::{BadgeVariant, ResolvedColor, ResolvedMetric, ShadcnLook, ShadcnModeTokens};

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
    let table = luma_look_shadcn::tables::metrics::resolve_badge_metrics(mode, look, variant, size, _theme_mode);
    table.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use luma_look_shadcn::{ColorSource, ShadcnLook, ShadcnToken};

    #[test]
    fn inspect_default_badge_uses_primary_fill() {
        let look = ShadcnLook::from_css_str(luma_look_shadcn::FALLBACK_CSS).expect("bundled fallback CSS should parse");
        let palette = inspect_badge_color_palette(&look, BadgeVariant::Default);
        assert_eq!(palette.background.value, look.color(ShadcnToken::Primary));
        assert!(matches!(
            palette.background.source,
            ColorSource::CssVar { ref token } if token == "primary"
        ));
    }

    #[test]
    fn inspect_badge_metrics_use_pill_radius() {
        let look = ShadcnLook::from_css_str(luma_look_shadcn::FALLBACK_CSS).expect("bundled fallback CSS should parse");
        let mode = look.mode_tokens();
        let metrics =
            inspect_badge_metrics(mode.as_ref(), &look, BadgeVariant::Default, ControlSize::Md, ThemeMode::Light);
        assert!(metrics.radius.value_px >= mode.metrics.radius.pill - f32::EPSILON);
    }
}

impl From<luma_look_shadcn::tables::metrics::BadgeMetricTable> for BadgeInspectMetrics {
    fn from(table: luma_look_shadcn::tables::metrics::BadgeMetricTable) -> Self {
        Self {
            min_height: table.min_height,
            padding_x: table.padding_x,
            padding_y: table.padding_y,
            gap: table.gap,
            icon_size: table.icon_size,
            radius: table.radius,
        }
    }
}
