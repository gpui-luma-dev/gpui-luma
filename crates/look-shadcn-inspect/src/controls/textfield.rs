//! Inspect metadata for `textfield`.

use gpui_luma::theme::{InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{LookContext, LookResolver, ResolvedColor, ResolvedMetric, ShadcnModeTokens};

use gpui_luma_look_shadcn::ShadcnTextFieldStyle;
use gpui_luma::controls::textfield::TextFieldState;

pub struct TextFieldInspectPalette {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
    pub placeholder: ResolvedColor,
    pub icon: ResolvedColor,
    pub selection_background: ResolvedColor,
    pub selection_foreground: ResolvedColor,
    pub caret: ResolvedColor,
    pub focus_ring: Option<ResolvedColor>,
}

pub fn inspect_textfield_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnTextFieldStyle,
    state: TextFieldState,
    enabled: bool,
) -> TextFieldInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "textfield_inspect");
    let colors =
        gpui_luma_look_shadcn::tables::resolve_textfield_colors(&resolver, style, enabled, state.invalid, theme_mode)
            .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::TextFieldColorTable::fallback());
    let focus_ring = (enabled && state.focus_visible).then(|| resolver.resolve_decl("ring")).transpose().ok().flatten();

    TextFieldInspectPalette {
        background: colors.background,
        foreground: colors.foreground,
        border: colors.border,
        placeholder: colors.placeholder,
        icon: colors.icon,
        selection_background: colors.selection_background,
        selection_foreground: colors.selection_foreground,
        caret: colors.caret,
        focus_ring,
    }
}

#[derive(Clone, Debug)]
pub struct TextFieldInspectMetrics {
    pub min_height: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub radius: ResolvedMetric,
    pub border_width: ResolvedMetric,
    pub focus_ring_width: ResolvedMetric,
    pub focus_ring_offset: ResolvedMetric,
}

pub fn inspect_textfield_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: gpui_luma::theme::ControlSize,
) -> TextFieldInspectMetrics {
    use gpui_luma::theme::StandardBoxScale;

    use crate::metrics::{
        border_width_metric, control_size_key, focus_ring_offset_metric, focus_ring_width_metric, radius_metric,
        scaffold_control_metric, spacing_control_metric,
    };
    use gpui_luma_look_shadcn::catalog::SpacingField;

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let catalog = ctx.catalog();
    let metrics = ctx.metrics();
    let scale = StandardBoxScale::compute(size, metrics, 1.0);

    TextFieldInspectMetrics {
        min_height: scaffold_control_metric(control_size_key(size), "control_height", scale.height),
        padding_x: spacing_control_metric(catalog, size, SpacingField::PaddingX, scale.padding_x),
        padding_y: spacing_control_metric(catalog, size, SpacingField::PaddingY, scale.padding_y),
        radius: radius_metric(catalog, size, scale.radius),
        border_width: border_width_metric(metrics),
        focus_ring_width: focus_ring_width_metric(metrics),
        focus_ring_offset: focus_ring_offset_metric(metrics),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::sample_catalog;

    #[test]
    fn inspect_soft_textfield_uses_muted_token() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let palette = super::inspect_textfield_color_palette(
            &mode,
            ThemeMode::Light,
            ShadcnTextFieldStyle::Soft,
            TextFieldState::default(),
            true,
        );
        assert!(matches!(
            palette.background.source,
            gpui_luma_look_shadcn::ColorSource::CssVar { ref token } if token == "muted"
        ));
        assert!(matches!(palette.border.source, gpui_luma_look_shadcn::ColorSource::Transparent));
    }

    #[test]
    fn textfield_metadata_covers_style_and_state_rows() {
        assert_eq!(
            gpui_luma_look_shadcn::stylesheet::resolve_textfield_colors_metadata(
                gpui_luma_look_shadcn::embedded_stylesheet()
            )
            .len(),
            15
        );
    }
}
