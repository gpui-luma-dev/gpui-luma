//! Inspect metadata for `split_view`.

use gpui_luma::theme::{InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{AppearanceContext, ColorSource, LookResolver, ResolvedColor, ResolvedMetric, ShadcnModeTokens};

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
    use crate::metrics::derived_metric;

    SplitViewInspectMetrics {
        separator_hitbox_width: derived_metric("split view separator hitbox width", 20.0),
        separator_cue_width: derived_metric("split view separator cue width", 4.0),
        separator_cue_hovered_width: derived_metric("split view separator cue hovered width", 8.0),
        separator_cue_radius: derived_metric("split view separator cue radius", 4.0),
        separator_cue_inset_y: derived_metric("split view separator cue inset y", 16.0),
    }
}

pub fn inspect_split_view_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    enabled: bool,
) -> SplitViewInspectPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    if ctx.catalog().tokens.is_empty() {
        let appearance = gpui_luma_look_shadcn::paint::split_view_from_palette(&ctx, enabled);
        return SplitViewInspectPalette {
            separator: resolved_from_hsla(
                appearance.separator,
                if enabled {
                    ColorSource::CssVar { token: "border".into() }
                } else {
                    ColorSource::CssVar { token: "muted-foreground".into() }
                },
            ),
            separator_hover: resolved_from_hsla(
                appearance.separator_hover,
                if enabled {
                    ColorSource::CssVar { token: "accent".into() }
                } else {
                    ColorSource::CssVar { token: "muted-foreground".into() }
                },
            ),
        };
    }

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "split_view_inspect");
    let colors = gpui_luma_look_shadcn::tables::resolve_split_view_colors(&resolver, enabled)
        .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::SplitViewColorTable::fallback());
    SplitViewInspectPalette { separator: colors.separator, separator_hover: colors.separator_hover }
}

fn resolved_from_hsla(value: gpui::Hsla, source: ColorSource) -> ResolvedColor {
    ResolvedColor { value, source }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{retro_arcade_catalog, sample_catalog};

    #[test]
    fn split_view_metadata_covers_enabled_and_disabled() {
        assert_eq!(
            gpui_luma_look_shadcn::stylesheet::resolve_split_view_colors_metadata(
                gpui_luma_look_shadcn::embedded_stylesheet()
            )
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
