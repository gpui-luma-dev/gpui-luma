//! Inspect metadata for `tabs`.

use luma::theme::{ControlSize, InteractionState, ThemeMode};
use luma_look_shadcn::{LookContext, LookResolver, ResolvedColor, ShadcnModeTokens};

pub struct TabsItemInspectPalette {
    pub label_color: ResolvedColor,
    pub indicator: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct TabsListInspectPalette {
    pub background: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct TabsInspectMetrics {
    pub list_radius: luma_look_shadcn::ResolvedMetric,
    pub list_gap: luma_look_shadcn::ResolvedMetric,
    pub list_padding: luma_look_shadcn::ResolvedMetric,
    pub item_padding_x: luma_look_shadcn::ResolvedMetric,
    pub item_height: luma_look_shadcn::ResolvedMetric,
    pub item_radius: luma_look_shadcn::ResolvedMetric,
    pub indicator_height: luma_look_shadcn::ResolvedMetric,
}

pub fn inspect_tabs_item_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    active: bool,
    state: InteractionState,
) -> TabsItemInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, state);
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "tabs_item_inspect");
    let colors = luma_look_shadcn::tables::resolve_tabs_item_colors(&resolver, active, state.layer(), state.focused)
        .unwrap_or_else(|_| luma_look_shadcn::tables::TabsItemColorTable::fallback());

    TabsItemInspectPalette { label_color: colors.label_color, indicator: colors.indicator }
}

pub fn inspect_tabs_list_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    enabled: bool,
) -> TabsListInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "tabs_list_inspect");
    let colors = luma_look_shadcn::tables::resolve_tabs_list_colors(&resolver, enabled)
        .unwrap_or_else(|_| luma_look_shadcn::tables::TabsListColorTable::fallback());
    let background = if enabled {
        None
    } else {
        Some(colors.disabled_background)
    };

    TabsListInspectPalette { background }
}

pub fn inspect_tabs_metrics(mode: &ShadcnModeTokens, theme_mode: ThemeMode, size: ControlSize) -> TabsInspectMetrics {
    use crate::metrics::{derived_metric, radius_metric, spacing_control_metric};
    use luma_look_shadcn::catalog::SpacingField;

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let catalog = ctx.catalog();
    let list = luma_look_shadcn::paint::tabs_list_look(mode, true, size);
    let item = luma_look_shadcn::paint::tabs_item_look(mode, true, InteractionState::default(), size);

    TabsInspectMetrics {
        list_radius: radius_metric(catalog, size, list.radius),
        list_gap: derived_metric("spacing.s5", list.gap),
        list_padding: derived_metric("list padding", list.padding),
        item_padding_x: spacing_control_metric(catalog, size, SpacingField::PaddingX, item.padding_x),
        item_height: derived_metric("label line_height + spacing.s2", item.height),
        item_radius: radius_metric(catalog, size, item.radius),
        indicator_height: derived_metric("active tab indicator height", item.indicator_height),
    }
}
