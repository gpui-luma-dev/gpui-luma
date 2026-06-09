//! Inspect metadata for `control_group`.

use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{AppearanceContext, ColorSource, LookResolver, ResolvedColor, ResolvedMetric, ShadcnModeTokens};

pub struct ControlGroupListInspectPalette {
    pub background: ResolvedColor,
    pub border: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ControlGroupInspectMetrics {
    pub radius: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub gap: ResolvedMetric,
}

pub fn inspect_control_group_list_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    enabled: bool,
) -> ControlGroupListInspectPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    if ctx.catalog().tokens.is_empty() {
        let appearance = gpui_luma_look_shadcn::paint::control_group_list_from_palette(&ctx, enabled);
        return ControlGroupListInspectPalette {
            background: resolved_from_hsla(
                appearance.background,
                if enabled {
                    ColorSource::CssVar { token: "muted".into() }
                } else {
                    ColorSource::CssVar { token: "muted-foreground".into() }
                },
            ),
            border: resolved_from_hsla(appearance.border, ColorSource::CssVar { token: "border".into() }),
        };
    }

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "control_group_list_inspect");
    let colors = gpui_luma_look_shadcn::tables::resolve_control_group_list_colors(&resolver, enabled)
        .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::ControlGroupListColorTable::fallback());
    ControlGroupListInspectPalette { background: colors.background, border: colors.border }
}

pub fn inspect_control_group_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> ControlGroupInspectMetrics {
    use crate::metrics::{derived_metric, radius_metric};

    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    let catalog = ctx.catalog();
    let appearance = gpui_luma_look_shadcn::paint::control_group_list_appearance(mode, true);

    ControlGroupInspectMetrics {
        radius: radius_metric(catalog, size, appearance.radius),
        padding_x: derived_metric("control group padding x", appearance.padding_x),
        padding_y: derived_metric("control group padding y", appearance.padding_y),
        gap: derived_metric("control group gap", appearance.gap),
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
    fn control_group_metadata_covers_enabled_and_disabled() {
        assert_eq!(
            gpui_luma_look_shadcn::stylesheet::resolve_control_group_list_colors_metadata(
                gpui_luma_look_shadcn::embedded_stylesheet()
            )
            .len(),
            2
        );
    }

    #[test]
    fn inspect_disabled_list_uses_muted_foreground_background() {
        let catalog = sample_catalog();
        let muted_foreground = catalog.color("muted-foreground").expect("muted-foreground");
        let mode = ShadcnModeTokens::from_catalog(catalog, ThemeMode::Light).expect("catalog");
        let palette = inspect_control_group_list_color_palette(&mode, ThemeMode::Light, false);
        assert_eq!(palette.background.value, muted_foreground);
    }
}
