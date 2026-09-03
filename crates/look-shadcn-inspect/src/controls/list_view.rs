//! Inspect metadata for `list_view`.

use luma::theme::{ControlSize, InteractionState, ThemeMode};
use luma_look_shadcn::{LookContext, LookResolver, ResolvedColor, ResolvedMetric, ShadcnModeTokens};

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
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "list_view_inspect");
    let colors = luma_look_shadcn::tables::resolve_list_view_surface_colors(&resolver, enabled)
        .unwrap_or_else(|_| luma_look_shadcn::tables::ListViewSurfaceColorTable::fallback());
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
    let ctx = LookContext::new(mode, theme_mode, state);
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "list_view_row_inspect");
    let colors = luma_look_shadcn::tables::resolve_list_view_row_colors(
        &resolver,
        selected,
        state.focused,
        state.disabled,
        state.layer(),
    )
    .unwrap_or_else(|_| luma_look_shadcn::tables::ListViewRowColorTable::fallback());
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
    use luma::theme::ListRowScale;

    use luma_look_shadcn::catalog::SpacingField;
    use crate::metrics::{control_size_key, derived_metric, radius_metric, scaffold_control_metric, spacing_control_metric};

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let look = luma_look_shadcn::paint::list_view_look(mode, true, false, size);
    let row_scale = ListRowScale::compute(size, metrics, 1.0);
    let size_key = control_size_key(size);

    ListViewInspectMetrics {
        radius: radius_metric(catalog, size, look.radius),
        padding_x: derived_metric("list padding x", look.padding_x),
        padding_y: spacing_control_metric(catalog, size, SpacingField::PaddingY, look.padding_y),
        row_min_height: scaffold_control_metric(size_key, "control_height", row_scale.min_height),
        row_padding_x: spacing_control_metric(catalog, size, SpacingField::PaddingX, row_scale.padding_x),
        row_padding_y: spacing_control_metric(catalog, size, SpacingField::PaddingY, row_scale.padding_y),
    }
}
