//! Inspect metadata for `progress`.

use gpui_luma::theme::{InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{LookContext, LookResolver, ResolvedColor, ShadcnModeTokens};

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
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "progress_inspect");
    let colors = gpui_luma_look_shadcn::tables::resolve_progress_colors(&resolver, enabled)
        .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::ProgressColorTable::fallback());
    ProgressInspectPalette { track_color: colors.track_color, progress_color: colors.progress_color }
}

pub fn inspect_progress_metrics(mode: &ShadcnModeTokens, _theme_mode: ThemeMode) -> ProgressInspectMetrics {
    use crate::metrics::derived_metric;

    let look = gpui_luma_look_shadcn::paint::progress_look(mode, true, gpui_luma::theme::ControlSize::Md);

    ProgressInspectMetrics {
        size: derived_metric("progress ring diameter", look.size),
        stroke_width: derived_metric("progress stroke width", look.stroke_width),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::sample_catalog;
    use gpui_luma_look_shadcn::ColorSource;

    #[test]
    fn progress_metadata_has_enabled_and_disabled_rows() {
        assert_eq!(
            gpui_luma_look_shadcn::stylesheet::resolve_progress_colors_metadata(
                gpui_luma_look_shadcn::embedded_stylesheet()
            )
            .len(),
            2
        );
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
