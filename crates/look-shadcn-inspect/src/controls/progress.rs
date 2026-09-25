//! Inspect metadata for `progress`.

use luma::theme::{InteractionState, ThemeMode};
use luma_look_shadcn::{LookContext, LookResolver, ResolvedColor, ShadcnModeTokens};

pub struct ProgressInspectPalette {
    pub track_color: ResolvedColor,
    pub progress_color: ResolvedColor,
    pub thumb_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ProgressInspectMetrics {
    pub size: luma_look_shadcn::ResolvedMetric,
    pub stroke_width: luma_look_shadcn::ResolvedMetric,
    pub track_height: luma_look_shadcn::ResolvedMetric,
    pub thumb_size: luma_look_shadcn::ResolvedMetric,
}

pub fn inspect_progress_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    enabled: bool,
) -> ProgressInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "progress_inspect");
    let colors = luma_look_shadcn::tables::resolve_progress_colors(&resolver, enabled)
        .unwrap_or_else(|_| luma_look_shadcn::tables::ProgressColorTable::fallback());
    ProgressInspectPalette {
        track_color: colors.track_color,
        progress_color: colors.progress_color.clone(),
        thumb_color: colors.progress_color,
    }
}

pub fn inspect_progress_metrics(mode: &ShadcnModeTokens, _theme_mode: ThemeMode) -> ProgressInspectMetrics {
    let table = luma_look_shadcn::tables::metrics::resolve_progress_metrics(mode, _theme_mode);
    table.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::sample_catalog;
    use luma_look_shadcn::ColorSource;

    #[test]
    fn progress_metadata_has_enabled_and_disabled_rows() {
        assert_eq!(
            luma_look_shadcn::stylesheet::resolve_progress_colors_metadata(luma_look_shadcn::embedded_stylesheet())
                .len(),
            2
        );
    }

    #[test]
    fn inspect_enabled_progress_uses_accent_foreground_fill() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let palette = inspect_progress_color_palette(&mode, ThemeMode::Light, true);
        assert!(matches!(
            palette.progress_color.source,
            ColorSource::CssVar { ref token } if token == "accent-foreground"
        ));
        assert!(matches!(
            palette.track_color.source,
            ColorSource::CssVar { ref token } if token == "accent"
        ));
    }

    #[test]
    fn inspect_progress_metrics_include_linear_fields() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let metrics = inspect_progress_metrics(&mode, ThemeMode::Light);
        assert!(metrics.track_height.value_px > 0.0);
        assert!(metrics.thumb_size.value_px > 0.0);
    }
}

impl From<luma_look_shadcn::tables::metrics::ProgressMetricTable> for ProgressInspectMetrics {
    fn from(table: luma_look_shadcn::tables::metrics::ProgressMetricTable) -> Self {
        Self {
            size: table.size,
            stroke_width: table.stroke_width,
            track_height: table.track_height,
            thumb_size: table.thumb_size,
        }
    }
}
