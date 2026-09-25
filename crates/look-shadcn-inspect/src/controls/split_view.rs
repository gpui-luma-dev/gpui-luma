//! Inspect metadata for `split_view`.

use luma::theme::{InteractionState, ThemeMode};
use luma_look_shadcn::{LookContext, LookResolver, ResolvedColor, ResolvedMetric, ShadcnModeTokens};

pub struct SplitViewInspectPalette {
    pub separator: ResolvedColor,
    pub separator_hover: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct SplitViewInspectMetrics {
    pub separator_hitbox_width: ResolvedMetric,
    pub separator_cue_width: ResolvedMetric,
    pub separator_cue_hovered_width: ResolvedMetric,
    pub separator_cue_radius: ResolvedMetric,
    pub separator_cue_inset_y: ResolvedMetric,
}

pub fn inspect_split_view_metrics() -> SplitViewInspectMetrics {
    let table = luma_look_shadcn::tables::metrics::resolve_split_view_metrics();
    table.into()
}

pub fn inspect_split_view_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    enabled: bool,
) -> SplitViewInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "split_view_inspect");
    let colors = luma_look_shadcn::tables::resolve_split_view_colors(&resolver, enabled)
        .unwrap_or_else(|_| luma_look_shadcn::tables::SplitViewColorTable::fallback());
    SplitViewInspectPalette { separator: colors.separator, separator_hover: colors.separator_hover }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::sample_catalog;
    use luma_look_shadcn::ColorSource;

    #[test]
    fn split_view_metadata_covers_enabled_and_disabled() {
        assert_eq!(
            luma_look_shadcn::stylesheet::resolve_split_view_colors_metadata(luma_look_shadcn::embedded_stylesheet())
                .len(),
            2
        );
    }

    #[test]
    fn inspect_enabled_split_view_uses_border_hover() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let palette = inspect_split_view_color_palette(&mode, ThemeMode::Light, true);
        assert!(
            matches!(
                palette.separator_hover.source,
                ColorSource::StateVar { ref base_token, .. } | ColorSource::Algorithmic { ref base_token, .. }
                    if base_token == "border"
            ) || matches!(palette.separator_hover.source, ColorSource::CssVar { ref token } if token == "border-hover")
        );
    }

    #[test]
    fn inspect_split_view_metrics_match_template_constants() {
        let metrics = inspect_split_view_metrics();
        assert!((metrics.separator_hitbox_width.value_px - 20.0).abs() < f32::EPSILON);
        assert!((metrics.separator_cue_hovered_width.value_px - 8.0).abs() < f32::EPSILON);
    }
}

impl From<luma_look_shadcn::tables::metrics::SplitViewMetricTable> for SplitViewInspectMetrics {
    fn from(table: luma_look_shadcn::tables::metrics::SplitViewMetricTable) -> Self {
        Self {
            separator_hitbox_width: table.separator_hitbox_width,
            separator_cue_width: table.separator_cue_width,
            separator_cue_hovered_width: table.separator_cue_hovered_width,
            separator_cue_radius: table.separator_cue_radius,
            separator_cue_inset_y: table.separator_cue_inset_y,
        }
    }
}
