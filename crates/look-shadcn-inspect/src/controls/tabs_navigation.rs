//! Inspect metadata for `tabs_navigation`.

use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{
    AppearanceContext, ColorSource, LookResolver, ResolvedColor, ShadcnModeTokens,
};


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
    pub list_radius: gpui_luma_look_shadcn::ResolvedMetric,
    pub list_gap: gpui_luma_look_shadcn::ResolvedMetric,
    pub list_padding: gpui_luma_look_shadcn::ResolvedMetric,
    pub item_padding_x: gpui_luma_look_shadcn::ResolvedMetric,
    pub item_height: gpui_luma_look_shadcn::ResolvedMetric,
    pub item_radius: gpui_luma_look_shadcn::ResolvedMetric,
    pub indicator_height: gpui_luma_look_shadcn::ResolvedMetric,
}

pub fn inspect_tabs_navigation_item_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    active: bool,
    state: InteractionState,
) -> TabsNavigationItemInspectPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, state);
    if ctx.catalog().tokens.is_empty() {
        let appearance = gpui_luma_look_shadcn::paint::tabs_navigation_item_from_palette(&ctx, active);
        return TabsNavigationItemInspectPalette {
            label_color: resolved_from_hsla(
                appearance.label_color,
                if state.disabled {
                    ColorSource::CssVar { token: "muted-foreground".into() }
                } else if active {
                    ColorSource::CssVar { token: "primary".into() }
                } else {
                    ColorSource::CssVar { token: "foreground".into() }
                },
            ),
            indicator: appearance.indicator.map(|color| {
                resolved_from_hsla(
                    color,
                    if state.focused {
                        ColorSource::CssVar { token: "ring".into() }
                    } else {
                        ColorSource::CssVar { token: "primary".into() }
                    },
                )
            }),
        };
    }

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "tabs_navigation_item_inspect");
    let colors = gpui_luma_look_shadcn::tables::resolve_tabs_navigation_item_colors(&resolver, active, state.layer(), state.focused)
        .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::TabsNavigationItemColorTable::fallback());

    TabsNavigationItemInspectPalette { label_color: colors.label_color, indicator: colors.indicator }
}

pub fn inspect_tabs_navigation_list_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    enabled: bool,
) -> TabsNavigationListInspectPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    if ctx.catalog().tokens.is_empty() {
        let appearance = gpui_luma_look_shadcn::paint::tabs_navigation_list_from_palette(&ctx, enabled);
        let background = match appearance.background {
            Some(color) => Some(resolved_from_hsla(color, ColorSource::CssVar { token: "muted".into() })),
            None => None,
        };
        return TabsNavigationListInspectPalette { background };
    }

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "tabs_navigation_list_inspect");
    let colors = gpui_luma_look_shadcn::tables::resolve_tabs_navigation_list_colors(&resolver, enabled)
        .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::TabsNavigationListColorTable::fallback());
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
    use gpui_luma_look_shadcn::catalog::SpacingField;

    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    let catalog = ctx.catalog();
    let list = if ctx.catalog().tokens.is_empty() {
        gpui_luma_look_shadcn::paint::tabs_navigation_list_from_palette(&ctx, true)
    } else {
        gpui_luma_look_shadcn::paint::tabs_navigation_list_from_catalog(&ctx, true).unwrap_or_else(|_| gpui_luma_look_shadcn::paint::tabs_navigation_list_from_palette(&ctx, true))
    };
    let item = if ctx.catalog().tokens.is_empty() {
        gpui_luma_look_shadcn::paint::tabs_navigation_item_from_palette(&ctx, true)
    } else {
        gpui_luma_look_shadcn::paint::tabs_navigation_item_from_catalog(&ctx, true).unwrap_or_else(|_| gpui_luma_look_shadcn::paint::tabs_navigation_item_from_palette(&ctx, true))
    };

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

fn resolved_from_hsla(value: gpui::Hsla, source: ColorSource) -> ResolvedColor {
    ResolvedColor { value, source }
}

