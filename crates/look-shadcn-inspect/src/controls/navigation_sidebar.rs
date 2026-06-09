//! Inspect metadata for `navigation_sidebar`.

use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{AppearanceContext, ColorSource, LookResolver, ResolvedColor, ResolvedMetric, ShadcnModeTokens};

use gpui_luma_look_shadcn::catalog::SpacingField;

pub struct NavigationSidebarContainerInspectPalette {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct NavigationSidebarSectionInspectPalette {
    pub label_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct NavigationSidebarItemInspectPalette {
    pub background: Option<ResolvedColor>,
    pub foreground: ResolvedColor,
    pub icon_color: ResolvedColor,
    pub focus_ring: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct NavigationSidebarInspectMetrics {
    pub section_height: ResolvedMetric,
    pub item_height: ResolvedMetric,
    pub item_padding_x: ResolvedMetric,
    pub item_gap: ResolvedMetric,
    pub item_radius: ResolvedMetric,
    pub item_icon_size: ResolvedMetric,
}

pub fn inspect_navigation_sidebar_container_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
) -> NavigationSidebarContainerInspectPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    if ctx.catalog().tokens.is_empty() {
        let appearance = gpui_luma_look_shadcn::paint::navigation_sidebar_container_from_palette(&ctx);
        return NavigationSidebarContainerInspectPalette {
            background: resolved_from_hsla(appearance.background, ColorSource::CssVar { token: "sidebar".into() }),
            foreground: resolved_from_hsla(
                appearance.foreground,
                ColorSource::CssVar { token: "sidebar-foreground".into() },
            ),
            border: resolved_from_hsla(appearance.border, ColorSource::CssVar { token: "sidebar-border".into() }),
        };
    }

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "navigation_sidebar_container_inspect");
    let colors = gpui_luma_look_shadcn::tables::resolve_navigation_sidebar_container_colors(&resolver, true)
        .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::NavigationSidebarContainerColorTable::fallback());
    NavigationSidebarContainerInspectPalette {
        background: colors.background,
        foreground: colors.foreground,
        border: colors.border,
    }
}

pub fn inspect_navigation_sidebar_section_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
) -> NavigationSidebarSectionInspectPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    if ctx.catalog().tokens.is_empty() {
        let appearance = gpui_luma_look_shadcn::paint::navigation_sidebar_section_from_palette(&ctx);
        return NavigationSidebarSectionInspectPalette {
            label_color: resolved_from_hsla(
                appearance.label_color,
                ColorSource::CssVar { token: "muted-foreground".into() },
            ),
        };
    }

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "navigation_sidebar_section_inspect");
    let colors = gpui_luma_look_shadcn::tables::resolve_navigation_sidebar_section_colors(&resolver, true)
        .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::NavigationSidebarSectionColorTable::fallback());
    NavigationSidebarSectionInspectPalette { label_color: colors.label_color }
}

pub fn inspect_navigation_sidebar_branch_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> NavigationSidebarItemInspectPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, state);
    if ctx.catalog().tokens.is_empty() {
        let appearance = gpui_luma_look_shadcn::paint::navigation_sidebar_branch_from_palette(&ctx, ControlSize::Md);
        return NavigationSidebarItemInspectPalette {
            background: appearance.background.map(|color| resolved_from_hsla(color, ColorSource::Transparent)),
            foreground: resolved_from_hsla(
                appearance.foreground,
                ColorSource::CssVar { token: "sidebar-foreground".into() },
            ),
            icon_color: resolved_from_hsla(
                appearance.icon_color,
                ColorSource::CssVar { token: "sidebar-foreground".into() },
            ),
            focus_ring: appearance
                .focus_ring
                .map(|color| resolved_from_hsla(color, ColorSource::CssVar { token: "sidebar-ring".into() })),
        };
    }

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "navigation_sidebar_branch_inspect");
    let colors = gpui_luma_look_shadcn::tables::resolve_navigation_sidebar_branch_colors(
        &resolver,
        state.disabled,
        state.layer(),
    )
    .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::NavigationSidebarBranchColorTable::fallback());
    let focus_ring = state
        .focused
        .then(|| resolver.resolve_first_decl(&["sidebar-ring", "ring"]))
        .transpose()
        .ok()
        .flatten();

    NavigationSidebarItemInspectPalette {
        background: colors.background,
        foreground: colors.foreground,
        icon_color: colors.icon_color,
        focus_ring,
    }
}

pub fn inspect_navigation_sidebar_item_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    selected: bool,
    state: InteractionState,
) -> NavigationSidebarItemInspectPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, state);
    if ctx.catalog().tokens.is_empty() {
        let appearance =
            gpui_luma_look_shadcn::paint::navigation_sidebar_item_from_palette(&ctx, selected, ControlSize::Md);
        return NavigationSidebarItemInspectPalette {
            background: appearance.background.map(|color| resolved_from_hsla(color, ColorSource::Transparent)),
            foreground: resolved_from_hsla(
                appearance.foreground,
                ColorSource::CssVar { token: "sidebar-foreground".into() },
            ),
            icon_color: resolved_from_hsla(
                appearance.icon_color,
                ColorSource::CssVar { token: "sidebar-foreground".into() },
            ),
            focus_ring: appearance
                .focus_ring
                .map(|color| resolved_from_hsla(color, ColorSource::CssVar { token: "sidebar-ring".into() })),
        };
    }

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "navigation_sidebar_item_inspect");
    let colors = gpui_luma_look_shadcn::tables::resolve_navigation_sidebar_item_colors(
        &resolver,
        selected,
        state.disabled,
        state.layer(),
    )
    .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::NavigationSidebarItemColorTable::fallback());
    let focus_ring = state
        .focused
        .then(|| resolver.resolve_first_decl(&["sidebar-ring", "ring"]))
        .transpose()
        .ok()
        .flatten();

    NavigationSidebarItemInspectPalette {
        background: colors.background,
        foreground: colors.foreground,
        icon_color: colors.icon_color,
        focus_ring,
    }
}

pub fn inspect_navigation_sidebar_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> NavigationSidebarInspectMetrics {
    use crate::metrics::{derived_metric, radius_metric, spacing_control_metric};

    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let size_metrics = metrics.for_size(size);

    NavigationSidebarInspectMetrics {
        section_height: derived_metric("navigation sidebar section height", 20.0),
        item_height: derived_metric("navigation sidebar item height", 30.0),
        item_padding_x: derived_metric("navigation sidebar item padding x", 8.0),
        item_gap: spacing_control_metric(catalog, size, SpacingField::Gap, size_metrics.gap),
        item_radius: radius_metric(catalog, size, metrics.radius(size)),
        item_icon_size: derived_metric("navigation sidebar icon size", 16.0),
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
    fn navigation_sidebar_container_metadata_matches_table() {
        assert_eq!(
            gpui_luma_look_shadcn::stylesheet::resolve_navigation_sidebar_container_colors_metadata(
                gpui_luma_look_shadcn::embedded_stylesheet()
            )
            .len(),
            1
        );
    }

    #[test]
    fn navigation_sidebar_item_metadata_matches_table() {
        assert_eq!(
            gpui_luma_look_shadcn::stylesheet::resolve_navigation_sidebar_item_colors_metadata(
                gpui_luma_look_shadcn::embedded_stylesheet()
            )
            .len(),
            10
        );
    }

    #[test]
    fn inspect_selected_item_uses_sidebar_primary() {
        let catalog = sample_catalog();
        let sidebar_primary = catalog.color("sidebar-primary").expect("sidebar-primary");
        let mode = ShadcnModeTokens::from_catalog(catalog, ThemeMode::Light).expect("catalog");
        let palette =
            inspect_navigation_sidebar_item_color_palette(&mode, ThemeMode::Light, true, InteractionState::default());
        assert_eq!(palette.background.as_ref().map(|color| color.value), Some(sidebar_primary));
    }

    #[test]
    fn inspect_container_palette_uses_sidebar_tokens() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let palette = inspect_navigation_sidebar_container_color_palette(&mode, ThemeMode::Light);
        assert_eq!(palette.background.value, catalog.color("sidebar").expect("sidebar"));
    }
}
