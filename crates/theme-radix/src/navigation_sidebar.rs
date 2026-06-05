//! Navigation sidebar property mappings (shadcn / tweakcn sidebar tokens):
//!
//! | Part              | Token                              |
//! |-------------------|------------------------------------|
//! | Container bg      | `sidebar`                          |
//! | Container fg      | `sidebar-foreground`               |
//! | Container border  | `sidebar-border`                   |
//! | Section label     | `muted-foreground`                 |
//! | Item fg           | `sidebar-foreground`               |
//! | Item hover bg     | `accent`                           |
//! | Item hover fg     | `accent-foreground`                |
//! | Selected bg       | `sidebar-primary`                  |
//! | Selected fg       | `sidebar-primary-foreground`       |
//! | Focus ring        | `sidebar-ring`                     |

use gpui_luma::controls::navigation_sidebar::{
    NavigationSidebarContainerAppearance, NavigationSidebarItemAppearance, NavigationSidebarSectionAppearance,
};
use gpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, ThemeMode};

use super::context::AppearanceContext;
use super::resolve::{resolve_accent_hover_pair, resolve_color, resolve_color_layer};
use super::catalog::CssTokenMap;
use super::mode::RadixModeTokens;

pub(crate) fn navigation_sidebar_container_appearance(mode: &RadixModeTokens) -> NavigationSidebarContainerAppearance {
    let ctx = AppearanceContext::new(mode, ThemeMode::Light, InteractionState::default());
    if mode.catalog.tokens.is_empty() {
        navigation_sidebar_container_from_palette(&ctx)
    } else {
        navigation_sidebar_container_from_catalog(&ctx)
            .unwrap_or_else(|err| panic!("navigation sidebar container properties: {err}"))
    }
}

pub(crate) fn navigation_sidebar_section_appearance(mode: &RadixModeTokens) -> NavigationSidebarSectionAppearance {
    let ctx = AppearanceContext::new(mode, ThemeMode::Light, InteractionState::default());
    if mode.catalog.tokens.is_empty() {
        navigation_sidebar_section_from_palette(&ctx)
    } else {
        navigation_sidebar_section_from_catalog(&ctx)
            .unwrap_or_else(|err| panic!("navigation sidebar section properties: {err}"))
    }
}

pub(crate) fn navigation_sidebar_branch_appearance(
    mode: &RadixModeTokens,
    state: InteractionState,
    size: ControlSize,
) -> NavigationSidebarItemAppearance {
    let ctx = AppearanceContext::new(mode, ThemeMode::Light, state);
    if mode.catalog.tokens.is_empty() {
        navigation_sidebar_branch_from_palette(&ctx, size)
    } else {
        navigation_sidebar_branch_from_catalog(&ctx, size)
            .unwrap_or_else(|err| panic!("navigation sidebar branch properties: {err}"))
    }
}

pub(crate) fn navigation_sidebar_item_appearance(
    mode: &RadixModeTokens,
    selected: bool,
    state: InteractionState,
    size: ControlSize,
) -> NavigationSidebarItemAppearance {
    let ctx = AppearanceContext::new(mode, ThemeMode::Light, state);
    if mode.catalog.tokens.is_empty() {
        navigation_sidebar_item_from_palette(&ctx, selected, size)
    } else {
        navigation_sidebar_item_from_catalog(&ctx, selected, size)
            .unwrap_or_else(|err| panic!("navigation sidebar item properties: {err}"))
    }
}

fn navigation_sidebar_container_from_palette(ctx: &AppearanceContext) -> NavigationSidebarContainerAppearance {
    let palette = ctx.palette();
    NavigationSidebarContainerAppearance {
        background: palette.panel_background,
        foreground: palette.app_foreground,
        border: palette.border_default,
    }
}

fn navigation_sidebar_section_from_palette(ctx: &AppearanceContext) -> NavigationSidebarSectionAppearance {
    let palette = ctx.palette();
    NavigationSidebarSectionAppearance {
        label_color: palette.app_muted_foreground,
        typography: ctx.typography().text.caption,
        height: 20.0,
    }
}

fn base_item_from_palette(ctx: &AppearanceContext, size: ControlSize) -> NavigationSidebarItemAppearance {
    let state = ctx.state;
    let palette = ctx.palette();
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let size_metrics = metrics.for_size(size);
    let foreground = if state.disabled {
        palette.disabled_foreground
    } else {
        palette.app_foreground
    };

    NavigationSidebarItemAppearance {
        background: None,
        foreground,
        icon_color: foreground,
        focus_ring: state.focused.then_some(palette.focus_ring),
        typography: typography.text.label,
        radius: metrics.radius(size),
        height: 30.0,
        padding_x: 8.0,
        gap: size_metrics.gap,
        icon_size: 16.0,
    }
}

fn navigation_sidebar_branch_from_palette(
    ctx: &AppearanceContext,
    size: ControlSize,
) -> NavigationSidebarItemAppearance {
    let state = ctx.state;
    let palette = ctx.palette();
    let mut appearance = base_item_from_palette(ctx, size);

    appearance.background = match state.layer() {
        InteractionLayer::Disabled | InteractionLayer::Default => None,
        InteractionLayer::Hovered | InteractionLayer::Pressed => Some(palette.ghost.hover_background),
    };

    if matches!(state.layer(), InteractionLayer::Hovered | InteractionLayer::Pressed) && !state.disabled {
        appearance.foreground = palette.primary.foreground;
        appearance.icon_color = palette.primary.foreground;
    }

    appearance
}

fn navigation_sidebar_item_from_palette(
    ctx: &AppearanceContext,
    selected: bool,
    size: ControlSize,
) -> NavigationSidebarItemAppearance {
    let state = ctx.state;
    let palette = ctx.palette();
    let mut appearance = base_item_from_palette(ctx, size);
    let primary = palette.primary;

    appearance.background = match (selected, state.layer()) {
        (_, InteractionLayer::Disabled) => None,
        (true, InteractionLayer::Pressed) => Some(primary.pressed_background),
        (true, InteractionLayer::Hovered) => Some(primary.hover_background),
        (true, InteractionLayer::Default) => Some(primary.background),
        (false, InteractionLayer::Pressed | InteractionLayer::Hovered) => Some(palette.ghost.hover_background),
        (false, InteractionLayer::Default) => None,
    };

    if !selected && !state.disabled && matches!(state.layer(), InteractionLayer::Hovered | InteractionLayer::Pressed) {
        appearance.foreground = palette.primary.foreground;
        appearance.icon_color = palette.primary.foreground;
    }

    if selected && !state.disabled {
        appearance.foreground = primary.foreground;
        appearance.icon_color = primary.foreground;
    }

    appearance
}

fn resolve_sidebar_background(catalog: &CssTokenMap) -> anyhow::Result<gpui::Hsla> {
    catalog.color_first(&["sidebar", "card", "background"])
}

fn resolve_sidebar_foreground(catalog: &CssTokenMap) -> anyhow::Result<gpui::Hsla> {
    catalog.color_first(&["sidebar-foreground", "foreground"])
}

fn resolve_sidebar_border(catalog: &CssTokenMap) -> anyhow::Result<gpui::Hsla> {
    catalog.color_first(&["sidebar-border", "border"])
}

fn resolve_sidebar_primary(catalog: &CssTokenMap) -> anyhow::Result<gpui::Hsla> {
    catalog.color_first(&["sidebar-primary", "primary"])
}

fn resolve_sidebar_primary_foreground(catalog: &CssTokenMap) -> anyhow::Result<gpui::Hsla> {
    catalog.color_first(&["sidebar-primary-foreground", "primary-foreground"])
}

fn resolve_sidebar_ring(catalog: &CssTokenMap) -> anyhow::Result<gpui::Hsla> {
    catalog.color_first(&["sidebar-ring", "ring"])
}

pub(crate) fn navigation_sidebar_container_from_catalog(
    ctx: &AppearanceContext,
) -> anyhow::Result<NavigationSidebarContainerAppearance> {
    let catalog = ctx.catalog();
    Ok(NavigationSidebarContainerAppearance {
        background: resolve_sidebar_background(catalog)?,
        foreground: resolve_sidebar_foreground(catalog)?,
        border: resolve_sidebar_border(catalog)?,
    })
}

pub(crate) fn navigation_sidebar_section_from_catalog(
    ctx: &AppearanceContext,
) -> anyhow::Result<NavigationSidebarSectionAppearance> {
    let catalog = ctx.catalog();
    Ok(NavigationSidebarSectionAppearance {
        label_color: resolve_color(catalog, "muted-foreground")?,
        typography: ctx.typography().text.caption,
        height: 20.0,
    })
}

fn base_item_from_catalog(
    ctx: &AppearanceContext,
    size: ControlSize,
) -> anyhow::Result<NavigationSidebarItemAppearance> {
    let state = ctx.state;
    let catalog = ctx.catalog();
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let size_metrics = metrics.for_size(size);
    let foreground = if state.disabled {
        resolve_color(catalog, "muted-foreground")?
    } else {
        resolve_sidebar_foreground(catalog)?
    };

    Ok(NavigationSidebarItemAppearance {
        background: None,
        foreground,
        icon_color: foreground,
        focus_ring: state.focused.then(|| resolve_sidebar_ring(catalog)).transpose()?,
        typography: typography.text.label,
        radius: metrics.radius(size),
        height: 30.0,
        padding_x: 8.0,
        gap: size_metrics.gap,
        icon_size: 16.0,
    })
}

pub(crate) fn navigation_sidebar_branch_from_catalog(
    ctx: &AppearanceContext,
    size: ControlSize,
) -> anyhow::Result<NavigationSidebarItemAppearance> {
    let state = ctx.state;
    let catalog = ctx.catalog();
    let mut appearance = base_item_from_catalog(ctx, size)?;
    let (hover_background, hover_foreground) = resolve_accent_hover_pair(catalog)?;

    appearance.background = match state.layer() {
        InteractionLayer::Disabled | InteractionLayer::Default => None,
        InteractionLayer::Hovered | InteractionLayer::Pressed => Some(hover_background),
    };

    if matches!(state.layer(), InteractionLayer::Hovered | InteractionLayer::Pressed) && !state.disabled {
        appearance.foreground = hover_foreground;
        appearance.icon_color = hover_foreground;
    }

    Ok(appearance)
}

fn resolve_sidebar_primary_layer(catalog: &CssTokenMap, layer: InteractionLayer) -> anyhow::Result<gpui::Hsla> {
    if catalog.get("sidebar-primary").is_some() {
        resolve_color_layer(catalog, "sidebar-primary", layer, true)
    } else {
        resolve_color_layer(catalog, "primary", layer, true)
    }
}

pub(crate) fn navigation_sidebar_item_from_catalog(
    ctx: &AppearanceContext,
    selected: bool,
    size: ControlSize,
) -> anyhow::Result<NavigationSidebarItemAppearance> {
    let state = ctx.state;
    let catalog = ctx.catalog();
    let mut appearance = base_item_from_catalog(ctx, size)?;
    let layer = state.layer();
    let (hover_background, hover_foreground) = resolve_accent_hover_pair(catalog)?;

    appearance.background = match (selected, layer) {
        (_, InteractionLayer::Disabled) => None,
        (true, InteractionLayer::Pressed | InteractionLayer::Hovered) => {
            Some(resolve_sidebar_primary_layer(catalog, layer)?)
        }
        (true, InteractionLayer::Default) => Some(resolve_sidebar_primary(catalog)?),
        (false, InteractionLayer::Pressed | InteractionLayer::Hovered) => Some(hover_background),
        (false, InteractionLayer::Default) => None,
    };

    if !selected && !state.disabled && matches!(layer, InteractionLayer::Hovered | InteractionLayer::Pressed) {
        appearance.foreground = hover_foreground;
        appearance.icon_color = hover_foreground;
    }

    if selected && !state.disabled {
        let selected_foreground = resolve_sidebar_primary_foreground(catalog)?;
        appearance.foreground = selected_foreground;
        appearance.icon_color = selected_foreground;
    }

    Ok(appearance)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use gpui_luma::theme::{ControlSize, InteractionState};

    use super::super::context::AppearanceContext;
    use super::super::catalog::CssTokenMap;
    use super::super::mode::RadixModeTokens;
    use super::{
        navigation_sidebar_container_from_catalog, navigation_sidebar_item_from_catalog,
        navigation_sidebar_section_from_catalog,
    };

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("primary-foreground".into(), "oklch(1 0 0)".into()),
            ("secondary".into(), "oklch(0.6437 0.1019 187.3840)".into()),
            ("secondary-foreground".into(), "oklch(1 0 0)".into()),
            ("background".into(), "oklch(0.9735 0.0261 90.0953)".into()),
            ("foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("muted".into(), "oklch(0.6979 0.0159 196.7940)".into()),
            ("muted-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("accent-foreground".into(), "oklch(1 0 0)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("sidebar".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("sidebar-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("sidebar-primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("sidebar-primary-foreground".into(), "oklch(1 0 0)".into()),
            ("sidebar-accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("sidebar-accent-foreground".into(), "oklch(1 0 0)".into()),
            ("sidebar-border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("sidebar-ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
        ]))
    }

    #[test]
    fn navigation_sidebar_uses_sidebar_tokens() {
        let catalog = sample_catalog();
        let mode = RadixModeTokens::from_catalog(catalog.clone()).expect("catalog");
        let ctx = AppearanceContext::new(&mode, gpui_luma::theme::ThemeMode::Light, InteractionState::default());
        let container = navigation_sidebar_container_from_catalog(&ctx).expect("container");
        let section = navigation_sidebar_section_from_catalog(&ctx).expect("section");
        let selected = navigation_sidebar_item_from_catalog(&ctx, true, ControlSize::Md).expect("selected item");

        assert_eq!(container.background, catalog.color("sidebar").expect("sidebar"));
        assert_eq!(section.label_color, catalog.color("muted-foreground").expect("muted-foreground"));
        assert_eq!(selected.background, Some(catalog.color("sidebar-primary").expect("sidebar-primary")));
        assert_eq!(
            selected.foreground,
            catalog.color("sidebar-primary-foreground").expect("sidebar-primary-foreground")
        );
    }
}
