//! Inspect metadata for `accordion`.

use gpui_luma::theme::{InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{AppearanceContext, LookResolver, ResolvedColor, ResolvedMetric, ShadcnModeTokens};

pub struct AccordionTriggerInspectPalette {
    pub background: Option<ResolvedColor>,
    pub foreground: ResolvedColor,
    pub border_color: ResolvedColor,
    pub icon_color: ResolvedColor,
    pub chevron_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct AccordionContentInspectPalette {
    pub background: Option<ResolvedColor>,
    pub foreground: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct AccordionInspectMetrics {
    pub trigger_height: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub content_padding_y: ResolvedMetric,
    pub radius: ResolvedMetric,
    pub inner_gap: ResolvedMetric,
    pub icon_size: ResolvedMetric,
    pub chevron_size: ResolvedMetric,
}

pub fn inspect_accordion_trigger_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> AccordionTriggerInspectPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, state);
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "accordion_trigger_inspect");
    let colors =
        gpui_luma_look_shadcn::tables::resolve_accordion_trigger_colors(&resolver, state.disabled, state.layer())
            .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::AccordionTriggerColorTable::fallback());
    AccordionTriggerInspectPalette {
        background: colors.background,
        foreground: colors.foreground,
        border_color: colors.border_color,
        icon_color: colors.icon_color,
        chevron_color: colors.chevron_color,
    }
}

pub fn inspect_accordion_content_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    expanded: bool,
) -> AccordionContentInspectPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "accordion_content_inspect");
    let colors = gpui_luma_look_shadcn::tables::resolve_accordion_content_colors(&resolver, expanded)
        .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::AccordionContentColorTable::fallback());
    AccordionContentInspectPalette { background: colors.background, foreground: colors.foreground }
}

pub fn inspect_accordion_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: gpui_luma::theme::ControlSize,
) -> AccordionInspectMetrics {
    use gpui_luma::controls::accordion::AccordionScale;

    use gpui_luma_look_shadcn::catalog::SpacingField;
    use crate::metrics::{control_size_key, derived_metric, radius_metric, scaffold_control_metric, spacing_control_metric};

    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let scale = AccordionScale::compute(size, metrics, 1.0);
    let size_key = control_size_key(size);

    AccordionInspectMetrics {
        trigger_height: scaffold_control_metric(size_key, "control_height", scale.trigger_height),
        padding_x: spacing_control_metric(catalog, size, SpacingField::PaddingX, scale.padding_x),
        padding_y: spacing_control_metric(catalog, size, SpacingField::PaddingY, scale.padding_y),
        content_padding_y: derived_metric(
            format!("{size_key} content padding y = padding_y × 1.5"),
            scale.content_padding_y,
        ),
        radius: radius_metric(catalog, size, scale.radius),
        inner_gap: spacing_control_metric(catalog, size, SpacingField::Gap, scale.inner_gap),
        icon_size: derived_metric("accordion icon size", scale.icon_size),
        chevron_size: derived_metric("accordion chevron size", scale.chevron_size),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{retro_arcade_catalog, sample_catalog};
    use gpui_luma_look_shadcn::ColorSource;

    #[test]
    fn accordion_metadata_covers_trigger_and_content_tables() {
        assert_eq!(
            gpui_luma_look_shadcn::stylesheet::resolve_accordion_trigger_colors_metadata(
                gpui_luma_look_shadcn::embedded_stylesheet()
            )
            .len(),
            5
        );
        assert_eq!(
            gpui_luma_look_shadcn::stylesheet::resolve_accordion_content_colors_metadata(
                gpui_luma_look_shadcn::embedded_stylesheet()
            )
            .len(),
            2
        );
    }

    #[test]
    fn inspect_hovered_trigger_uses_accent_pair() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let palette = inspect_accordion_trigger_color_palette(
            &mode,
            ThemeMode::Light,
            InteractionState { hovered: true, ..InteractionState::default() },
        );
        let background = palette.background.expect("hovered trigger should paint accent background");
        assert!(matches!(background.source, ColorSource::CssVar { ref token } if token == "accent"));
        assert!(matches!(
            palette.foreground.source,
            ColorSource::CssVar { ref token } if token == "accent-foreground"
        ));
    }
}
