//! Inspect metadata for `tabs_navigation`.

use luma::theme::{ControlSize, InteractionState, ThemeMode};
use luma_look_shadcn::{LookContext, LookResolver, ResolvedColor, ShadcnModeTokens};

pub struct TabsNavigationItemInspectPalette {
    pub label_color: ResolvedColor,
    pub indicator: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct TabsNavigationListInspectPalette {
    pub background: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct TabsNavigationInspectMetrics {
    pub list_radius: luma_look_shadcn::ResolvedMetric,
    pub list_gap: luma_look_shadcn::ResolvedMetric,
    pub list_padding: luma_look_shadcn::ResolvedMetric,
    pub item_padding_x: luma_look_shadcn::ResolvedMetric,
    pub item_height: luma_look_shadcn::ResolvedMetric,
    pub item_radius: luma_look_shadcn::ResolvedMetric,
    pub indicator_height: luma_look_shadcn::ResolvedMetric,
}

pub fn inspect_tabs_navigation_item_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    active: bool,
    state: InteractionState,
) -> TabsNavigationItemInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, state);
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "tabs_navigation_item_inspect");
    let colors =
        luma_look_shadcn::tables::resolve_tabs_navigation_item_colors(&resolver, active, state.layer(), state.focused)
            .unwrap_or_else(|_| luma_look_shadcn::tables::TabsNavigationItemColorTable::fallback());

    TabsNavigationItemInspectPalette { label_color: colors.label_color, indicator: colors.indicator }
}

pub fn inspect_tabs_navigation_list_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    enabled: bool,
) -> TabsNavigationListInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "tabs_navigation_list_inspect");
    let colors = luma_look_shadcn::tables::resolve_tabs_navigation_list_colors(&resolver, enabled)
        .unwrap_or_else(|_| luma_look_shadcn::tables::TabsNavigationListColorTable::fallback());
    let background = if enabled {
        None
    } else {
        Some(colors.disabled_background)
    };

    TabsNavigationListInspectPalette { background }
}

pub fn inspect_tabs_navigation_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> TabsNavigationInspectMetrics {
    use crate::metrics::{derived_metric, radius_metric, spacing_control_metric};
    use luma_look_shadcn::catalog::SpacingField;

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let catalog = ctx.catalog();
    let list = luma_look_shadcn::paint::tabs_navigation_list_look(mode, true, size);
    let item = luma_look_shadcn::paint::tabs_navigation_item_look(mode, true, InteractionState::default(), size);

    TabsNavigationInspectMetrics {
        list_radius: radius_metric(catalog, size, list.radius),
        list_gap: derived_metric("spacing.s5", list.gap),
        list_padding: derived_metric("list padding", list.padding),
        item_padding_x: spacing_control_metric(catalog, size, SpacingField::PaddingX, item.padding_x),
        item_height: derived_metric("label line_height + spacing.s2", item.height),
        item_radius: radius_metric(catalog, size, item.radius),
        indicator_height: derived_metric("active tab indicator height", item.indicator_height),
    }
}
