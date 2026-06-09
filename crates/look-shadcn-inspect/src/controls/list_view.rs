//! Inspect metadata for `list_view`.

use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{AppearanceContext, ColorSource, LookResolver, ResolvedColor, ResolvedMetric, ShadcnModeTokens};

pub struct ListViewInspectPalette {
    pub background: ResolvedColor,
    pub border: ResolvedColor,
    pub header_background: ResolvedColor,
    pub header_label_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ListViewRowInspectPalette {
    pub background: ResolvedColor,
    pub label_color: ResolvedColor,
    pub divider: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ListViewInspectMetrics {
    pub radius: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub row_min_height: ResolvedMetric,
    pub row_padding_x: ResolvedMetric,
    pub row_padding_y: ResolvedMetric,
}

pub fn inspect_list_view_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    enabled: bool,
) -> ListViewInspectPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    if ctx.catalog().tokens.is_empty() {
        let appearance =
            gpui_luma_look_shadcn::paint::list_view_appearance_from_palette(&ctx, enabled, false, ControlSize::Md);
        return ListViewInspectPalette {
            background: resolved_from_hsla(appearance.background, ColorSource::CssVar { token: "background".into() }),
            border: resolved_from_hsla(appearance.border, ColorSource::CssVar { token: "input".into() }),
            header_background: resolved_from_hsla(
                appearance.header_background,
                ColorSource::CssVar { token: "muted".into() },
            ),
            header_label_color: resolved_from_hsla(
                appearance.header_label_color,
                ColorSource::CssVar { token: "muted-foreground".into() },
            ),
        };
    }

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "list_view_inspect");
    let colors = gpui_luma_look_shadcn::tables::resolve_list_view_surface_colors(&resolver, enabled)
        .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::ListViewSurfaceColorTable::fallback());
    ListViewInspectPalette {
        background: colors.background,
        border: colors.border,
        header_background: colors.header_background,
        header_label_color: colors.header_label_color,
    }
}

pub fn inspect_list_view_row_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    selected: bool,
    state: InteractionState,
) -> ListViewRowInspectPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, state);
    if ctx.catalog().tokens.is_empty() {
        let palette = gpui_luma_look_shadcn::paint::list_view_row_from_palette(&ctx, selected, ControlSize::Md);
        return ListViewRowInspectPalette {
            background: resolved_from_hsla(palette.background, ColorSource::Transparent),
            label_color: resolved_from_hsla(palette.label_color, ColorSource::CssVar { token: "foreground".into() }),
            divider: resolved_from_hsla(palette.divider, ColorSource::CssVar { token: "border".into() }),
        };
    }

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "list_view_row_inspect");
    let colors = gpui_luma_look_shadcn::tables::resolve_list_view_row_colors(
        &resolver,
        selected,
        state.focused,
        state.disabled,
        state.layer(),
    )
    .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::ListViewRowColorTable::fallback());
    ListViewRowInspectPalette {
        background: colors.background,
        label_color: colors.label_color,
        divider: colors.divider,
    }
}

pub fn inspect_list_view_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> ListViewInspectMetrics {
    use gpui_luma::theme::ListRowScale;

    use gpui_luma_look_shadcn::catalog::SpacingField;
    use crate::metrics::{control_size_key, derived_metric, radius_metric, scaffold_control_metric, spacing_control_metric};

    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let appearance = if ctx.catalog().tokens.is_empty() {
        gpui_luma_look_shadcn::paint::list_view_appearance_from_palette(&ctx, true, false, size)
    } else {
        gpui_luma_look_shadcn::paint::list_view_appearance_from_catalog(&ctx, true, false, size).unwrap_or_else(|_| {
            gpui_luma_look_shadcn::paint::list_view_appearance_from_palette(&ctx, true, false, size)
        })
    };
    let row_scale = ListRowScale::compute(size, metrics, 1.0);
    let size_key = control_size_key(size);

    ListViewInspectMetrics {
        radius: radius_metric(catalog, size, appearance.radius),
        padding_x: derived_metric("list padding x", appearance.padding_x),
        padding_y: spacing_control_metric(catalog, size, SpacingField::PaddingY, appearance.padding_y),
        row_min_height: scaffold_control_metric(size_key, "control_height", row_scale.min_height),
        row_padding_x: spacing_control_metric(catalog, size, SpacingField::PaddingX, row_scale.padding_x),
        row_padding_y: spacing_control_metric(catalog, size, SpacingField::PaddingY, row_scale.padding_y),
    }
}

fn resolved_from_hsla(value: gpui::Hsla, source: ColorSource) -> ResolvedColor {
    ResolvedColor { value, source }
}
