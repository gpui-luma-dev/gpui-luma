//! Inspect metadata for `progress`.

use gpui_luma::theme::{InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{
    AppearanceContext, ColorSource, LookResolver, ResolvedColor, ShadcnModeTokens,
};


pub struct ProgressInspectPalette {
    pub track_color: ResolvedColor,
    pub progress_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ProgressInspectMetrics {
    pub size: gpui_luma_look_shadcn::ResolvedMetric,
    pub stroke_width: gpui_luma_look_shadcn::ResolvedMetric,
}

pub fn inspect_progress_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    enabled: bool,
) -> ProgressInspectPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    if ctx.catalog().tokens.is_empty() {
        let appearance = gpui_luma_look_shadcn::paint::progress_from_palette(&ctx, enabled);
        return ProgressInspectPalette {
            track_color: resolved_from_hsla(
                appearance.track_color,
                if enabled {
                    ColorSource::CssVar { token: "muted".into() }
                } else {
                    ColorSource::CssVar { token: "muted-foreground".into() }
                },
            ),
            progress_color: resolved_from_hsla(
                appearance.progress_color,
                if enabled {
                    ColorSource::CssVar { token: "primary".into() }
                } else {
                    ColorSource::CssVar { token: "muted-foreground".into() }
                },
            ),
        };
    }

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "progress_inspect");
    let colors = gpui_luma_look_shadcn::tables::resolve_progress_colors(&resolver, enabled).unwrap_or_else(|_| gpui_luma_look_shadcn::tables::ProgressColorTable::fallback());
    ProgressInspectPalette { track_color: colors.track_color, progress_color: colors.progress_color }
}

pub fn inspect_progress_metrics(mode: &ShadcnModeTokens, theme_mode: ThemeMode) -> ProgressInspectMetrics {
    use crate::metrics::derived_metric;

    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    let appearance = if ctx.catalog().tokens.is_empty() {
        gpui_luma_look_shadcn::paint::progress_from_palette(&ctx, true)
    } else {
        gpui_luma_look_shadcn::paint::progress_from_catalog(&ctx, true).unwrap_or_else(|_| gpui_luma_look_shadcn::paint::progress_from_palette(&ctx, true))
    };

    ProgressInspectMetrics {
        size: derived_metric("progress ring diameter", appearance.size),
        stroke_width: derived_metric("progress stroke width", appearance.stroke_width),
    }
}

fn resolved_from_hsla(value: gpui::Hsla, source: ColorSource) -> ResolvedColor {
    ResolvedColor { value, source }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{retro_arcade_catalog, sample_catalog};


    #[test]
    fn progress_metadata_has_enabled_and_disabled_rows() {
        assert_eq!(gpui_luma_look_shadcn::tables::resolve_progress_colors_metadata().len(), 2);
    }

    #[test]
    fn inspect_enabled_progress_uses_primary_fill() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let palette = inspect_progress_color_palette(&mode, ThemeMode::Light, true);
        assert!(matches!(
            palette.progress_color.source,
            ColorSource::CssVar { ref token } if token == "primary"
        ));
    }
}
