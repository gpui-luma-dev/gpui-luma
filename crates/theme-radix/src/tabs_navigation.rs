//! Tabs navigation property mappings:
//!
//! | Part              | Token                         |
//! |-------------------|-------------------------------|
//! | Inactive label    | `foreground`                  |
//! | Active label      | `primary` (interaction layers)|
//! | Active indicator  | `primary` / `ring` on focus   |
//! | Disabled label    | `muted-foreground`            |
//! | Disabled list bg  | `muted`                       |

use gpui_luma::controls::tabs_navigation::{TabsNavigationItemAppearance, TabsNavigationListAppearance};
use gpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, ThemeMode};

use super::context::AppearanceContext;
use super::focus::focus_ring_color;
use super::resolve::{resolve_color, resolve_color_layer, resolve_label_color};
use super::mode::RadixModeTokens;

pub(crate) fn tabs_navigation_list_appearance(mode: &RadixModeTokens, enabled: bool) -> TabsNavigationListAppearance {
    let ctx = AppearanceContext::new(mode, ThemeMode::Light, InteractionState::default());
    if mode.catalog.tokens.is_empty() {
        tabs_navigation_list_from_palette(&ctx, enabled)
    } else {
        tabs_navigation_list_from_catalog(&ctx, enabled)
            .unwrap_or_else(|err| panic!("tabs navigation list properties: {err}"))
    }
}

pub(crate) fn tabs_navigation_item_appearance(
    mode: &RadixModeTokens,
    active: bool,
    state: InteractionState,
) -> TabsNavigationItemAppearance {
    let ctx = AppearanceContext::new(mode, ThemeMode::Light, state);
    if mode.catalog.tokens.is_empty() {
        tabs_navigation_item_from_palette(&ctx, active)
    } else {
        tabs_navigation_item_from_catalog(&ctx, active)
            .unwrap_or_else(|err| panic!("tabs navigation item properties: {err}"))
    }
}

fn tabs_navigation_list_from_palette(ctx: &AppearanceContext, enabled: bool) -> TabsNavigationListAppearance {
    let palette = ctx.palette();
    let metrics = ctx.metrics();
    let size = ControlSize::Md;

    TabsNavigationListAppearance {
        background: (!enabled).then_some(palette.disabled_background),
        border: None,
        radius: metrics.radius(size),
        padding: 0.0,
        gap: metrics.spacing.s5,
    }
}

fn tabs_navigation_item_from_palette(ctx: &AppearanceContext, active: bool) -> TabsNavigationItemAppearance {
    let state = ctx.state;
    let palette = ctx.palette();
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let size = ControlSize::Md;
    let layer = state.layer();
    let primary = palette.primary;

    let active_color = match layer {
        InteractionLayer::Disabled => palette.disabled_foreground,
        InteractionLayer::Pressed => primary.pressed_background,
        InteractionLayer::Hovered => primary.hover_background,
        InteractionLayer::Default => primary.background,
    };

    TabsNavigationItemAppearance {
        label_color: match (active, state.disabled) {
            (_, true) => palette.disabled_foreground,
            (true, false) => active_color,
            (false, false) => palette.app_foreground,
        },
        indicator: active.then_some(if state.focused {
            palette.focus_ring
        } else {
            active_color
        }),
        label_typography: typography.text.label,
        radius: metrics.radius(size),
        padding_x: metrics.padding_x(size),
        height: typography.text.label.line_height + metrics.spacing.s2,
        indicator_height: 2.0,
    }
}

pub(crate) fn tabs_navigation_list_from_catalog(
    ctx: &AppearanceContext,
    enabled: bool,
) -> anyhow::Result<TabsNavigationListAppearance> {
    let catalog = ctx.catalog();
    let metrics = ctx.metrics();
    let size = ControlSize::Md;

    Ok(TabsNavigationListAppearance {
        background: if enabled {
            None
        } else {
            Some(resolve_color(catalog, "muted")?)
        },
        border: None,
        radius: metrics.radius(size),
        padding: 0.0,
        gap: metrics.spacing.s5,
    })
}

pub(crate) fn tabs_navigation_item_from_catalog(
    ctx: &AppearanceContext,
    active: bool,
) -> anyhow::Result<TabsNavigationItemAppearance> {
    let state = ctx.state;
    let catalog = ctx.catalog();
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let size = ControlSize::Md;
    let layer = state.layer();

    let active_color = resolve_color_layer(catalog, "primary", layer, true)?;
    let inactive_color = resolve_label_color(catalog, state.disabled)?;

    Ok(TabsNavigationItemAppearance {
        label_color: if state.disabled {
            resolve_color(catalog, "muted-foreground")?
        } else if active {
            active_color
        } else {
            inactive_color
        },
        indicator: if active {
            Some(if state.focused {
                focus_ring_color(catalog)?
            } else {
                active_color
            })
        } else {
            None
        },
        label_typography: typography.text.label,
        radius: metrics.radius(size),
        padding_x: metrics.padding_x(size),
        height: typography.text.label.line_height + metrics.spacing.s2,
        indicator_height: 2.0,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use gpui_luma::theme::InteractionState;

    use super::super::context::AppearanceContext;
    use super::super::catalog::CssTokenMap;
    use super::super::mode::RadixModeTokens;
    use super::tabs_navigation_item_from_catalog;

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
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
        ]))
    }

    #[test]
    fn active_tab_uses_primary_and_inactive_uses_foreground() {
        let catalog = sample_catalog();
        let mode = RadixModeTokens::from_catalog(catalog.clone()).expect("catalog");
        let ctx = AppearanceContext::new(&mode, gpui_luma::theme::ThemeMode::Light, InteractionState::default());
        let active = tabs_navigation_item_from_catalog(&ctx, true).expect("active tab");
        let inactive = tabs_navigation_item_from_catalog(&ctx, false).expect("inactive tab");

        assert_eq!(active.label_color, catalog.color("primary").expect("primary"));
        assert_eq!(inactive.label_color, catalog.color("foreground").expect("foreground"));
        assert_eq!(active.indicator, Some(catalog.color("primary").expect("primary")));
    }
}
