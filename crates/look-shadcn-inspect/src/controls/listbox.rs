//! Inspect metadata for `listbox`.

use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{AppearanceContext, ColorSource, LookResolver, ResolvedColor, ResolvedMetric, ShadcnModeTokens};

use gpui_luma_look_shadcn::paint::focus_ring_color;

pub struct ListBoxListInspectPalette {
    pub background: ResolvedColor,
    pub border: ResolvedColor,
    pub divider: ResolvedColor,
    pub focus_ring: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct ListBoxRowInspectPalette {
    pub background: ResolvedColor,
    pub label_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ListBoxInspectMetrics {
    pub radius: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub row_gap: ResolvedMetric,
    pub row_min_height: ResolvedMetric,
    pub row_padding_x: ResolvedMetric,
    pub row_padding_y: ResolvedMetric,
}

pub fn inspect_listbox_list_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    enabled: bool,
    focused: bool,
) -> ListBoxListInspectPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    if ctx.catalog().tokens.is_empty() {
        let appearance =
            gpui_luma_look_shadcn::paint::listbox_list_from_palette(&ctx, enabled, focused, ControlSize::Md);
        return ListBoxListInspectPalette {
            background: resolved_from_hsla(
                appearance.background,
                if enabled {
                    ColorSource::CssVar { token: "background".into() }
                } else {
                    ColorSource::CssVar { token: "muted".into() }
                },
            ),
            border: resolved_from_hsla(appearance.border, ColorSource::CssVar { token: "input".into() }),
            divider: resolved_from_hsla(appearance.divider, ColorSource::CssVar { token: "border".into() }),
            focus_ring: focused
                .then(|| resolved_from_hsla(ctx.palette().focus_ring, ColorSource::CssVar { token: "ring".into() })),
        };
    }

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "listbox_list_inspect");
    let colors = gpui_luma_look_shadcn::tables::resolve_listbox_list_colors(&resolver, enabled)
        .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::ListBoxListColorTable::fallback());
    let focus_ring = focused
        .then(|| focus_ring_color(ctx.catalog()))
        .transpose()
        .ok()
        .flatten()
        .map(|color| resolved_from_hsla(color, ColorSource::CssVar { token: "ring".into() }));

    ListBoxListInspectPalette {
        background: colors.background,
        border: colors.border,
        divider: colors.divider,
        focus_ring,
    }
}

pub fn inspect_listbox_row_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> ListBoxRowInspectPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, state);
    if ctx.catalog().tokens.is_empty() {
        let palette = gpui_luma_look_shadcn::paint::listbox_row_from_palette(&ctx, ControlSize::Md);
        return ListBoxRowInspectPalette {
            background: resolved_from_hsla(palette.background, ColorSource::Transparent),
            label_color: resolved_from_hsla(
                palette.label_color,
                if state.disabled {
                    ColorSource::CssVar { token: "muted-foreground".into() }
                } else {
                    ColorSource::CssVar { token: "foreground".into() }
                },
            ),
        };
    }

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "listbox_row_inspect");
    let colors = gpui_luma_look_shadcn::tables::resolve_listbox_row_colors(
        &resolver,
        state.disabled,
        state.focused,
        state.layer(),
    )
    .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::ListBoxRowColorTable::fallback());
    ListBoxRowInspectPalette { background: colors.background, label_color: colors.label_color }
}

pub fn inspect_listbox_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> ListBoxInspectMetrics {
    use gpui_luma::theme::ListRowScale;

    use gpui_luma_look_shadcn::catalog::SpacingField;
    use crate::metrics::{derived_metric, radius_metric, scaffold_control_metric, spacing_control_metric};

    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let list_scale = ListRowScale::compute(size, metrics, 1.0);
    let size_key = crate::metrics::control_size_key(size);

    ListBoxInspectMetrics {
        radius: radius_metric(catalog, size, metrics.radius(size)),
        padding_x: derived_metric("listbox padding x", 6.0),
        padding_y: derived_metric(
            format!("{size_key} list padding y = padding_y × 0.5"),
            metrics.padding_y(size) * 0.5,
        ),
        row_gap: derived_metric(format!("{size_key} list row gap = padding_y × 0.25"), metrics.padding_y(size) * 0.25),
        row_min_height: scaffold_control_metric(size_key, "row_min_height", list_scale.min_height),
        row_padding_x: spacing_control_metric(catalog, size, SpacingField::PaddingX, list_scale.padding_x),
        row_padding_y: spacing_control_metric(catalog, size, SpacingField::PaddingY, list_scale.padding_y),
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
    fn listbox_metadata_covers_list_and_row_tables() {
        assert_eq!(
            gpui_luma_look_shadcn::stylesheet::resolve_listbox_list_colors_metadata(
                gpui_luma_look_shadcn::embedded_stylesheet()
            )
            .len(),
            2
        );
        assert_eq!(
            gpui_luma_look_shadcn::stylesheet::resolve_listbox_row_colors_metadata(
                gpui_luma_look_shadcn::embedded_stylesheet()
            )
            .len(),
            6
        );
    }

    #[test]
    fn inspect_hovered_row_uses_accent_layer() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let palette = inspect_listbox_row_color_palette(
            &mode,
            ThemeMode::Light,
            gpui_luma::theme::InteractionState { hovered: true, ..Default::default() },
        );
        assert!(palette.background.value.a > 0.0);
    }
}
