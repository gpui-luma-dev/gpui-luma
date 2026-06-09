//! Inspect metadata for `tree_view`.

use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{AppearanceContext, ColorSource, LookResolver, ResolvedColor, ResolvedMetric, ShadcnModeTokens};

pub struct TreeViewRowInspectPalette {
    pub background: Option<ResolvedColor>,
    pub foreground: ResolvedColor,
    pub icon_color: ResolvedColor,
    pub chevron_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct TreeViewInspectMetrics {
    pub row_height: ResolvedMetric,
    pub base_padding_x: ResolvedMetric,
    pub indentation_width: ResolvedMetric,
    pub inner_gap: ResolvedMetric,
    pub radius: ResolvedMetric,
    pub icon_size: ResolvedMetric,
    pub chevron_size: ResolvedMetric,
}

pub fn inspect_tree_view_row_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> TreeViewRowInspectPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, state);
    if ctx.catalog().tokens.is_empty() {
        let palette = gpui_luma_look_shadcn::paint::tree_view_row_from_palette(mode, state, &ctx);
        return TreeViewRowInspectPalette {
            background: palette.background.map(|color| resolved_from_hsla(color, ColorSource::Transparent)),
            foreground: resolved_from_hsla(
                palette.foreground,
                ColorSource::CssVar { token: "sidebar-foreground".into() },
            ),
            icon_color: resolved_from_hsla(
                palette.icon_color,
                ColorSource::CssVar { token: "sidebar-foreground".into() },
            ),
            chevron_color: resolved_from_hsla(
                palette.chevron_color,
                ColorSource::CssVar { token: "sidebar-foreground".into() },
            ),
        };
    }

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "tree_view_row_inspect");
    let colors = gpui_luma_look_shadcn::tables::resolve_tree_view_row_colors(&resolver, state.disabled, state.layer())
        .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::TreeViewRowColorTable::fallback());
    TreeViewRowInspectPalette {
        background: colors.background,
        foreground: colors.foreground,
        icon_color: colors.icon_color,
        chevron_color: colors.chevron_color,
    }
}

pub fn inspect_tree_view_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> TreeViewInspectMetrics {
    use gpui_luma::controls::tree_view::TreeViewScale;

    use gpui_luma_look_shadcn::catalog::SpacingField;
    use crate::metrics::{control_size_key, derived_metric, radius_metric, spacing_control_metric};

    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let scale = TreeViewScale::compute(size, metrics, 1.0);
    let size_key = control_size_key(size);

    TreeViewInspectMetrics {
        row_height: derived_metric(format!("{size_key} tree row height = control_height × 0.85"), scale.row_height),
        base_padding_x: spacing_control_metric(catalog, size, SpacingField::PaddingX, scale.base_padding_x),
        indentation_width: derived_metric("tree indentation width", scale.indentation_width),
        inner_gap: spacing_control_metric(catalog, size, SpacingField::Gap, scale.inner_gap),
        radius: radius_metric(catalog, size, scale.radius),
        icon_size: derived_metric("tree icon size", scale.icon_size),
        chevron_size: derived_metric("tree chevron size", scale.chevron_size),
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
    fn tree_view_metadata_covers_row_table() {
        assert_eq!(
            gpui_luma_look_shadcn::stylesheet::resolve_tree_view_row_colors_metadata(
                gpui_luma_look_shadcn::embedded_stylesheet()
            )
            .len(),
            5
        );
    }

    #[test]
    fn inspect_disabled_row_uses_muted_foreground() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let palette = inspect_tree_view_row_color_palette(
            &mode,
            ThemeMode::Light,
            gpui_luma::theme::InteractionState { disabled: true, ..Default::default() },
        );
        assert!(palette.background.is_none());
        assert!(matches!(
            palette.foreground.source,
            gpui_luma_look_shadcn::ColorSource::CssVar { ref token } if token == "muted-foreground"
        ));
    }
}
