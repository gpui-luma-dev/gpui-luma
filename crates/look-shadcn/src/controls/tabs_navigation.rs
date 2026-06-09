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

use crate::appearance_context::AppearanceContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::stylesheet::{
    StylesheetConfig, embedded_stylesheet, find_tabs_navigation_item_color_rule, find_tabs_navigation_list_color_rule,
    resolve_tabs_navigation_item_color_rule, resolve_tabs_navigation_list_color_rule,
};

#[derive(Clone, Debug)]
pub struct TabsNavigationListColorTable {
    pub disabled_background: ResolvedColor,
}

impl TabsNavigationListColorTable {
    pub fn fallback() -> Self {
        Self { disabled_background: ResolvedColor::transparent() }
    }
}

pub fn resolve_tabs_navigation_list_colors(
    resolver: &LookResolver<'_>,
    enabled: bool,
) -> anyhow::Result<TabsNavigationListColorTable> {
    resolve_tabs_navigation_list_colors_with_stylesheet(resolver, embedded_stylesheet(), enabled)
}

pub fn resolve_tabs_navigation_list_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    enabled: bool,
) -> anyhow::Result<TabsNavigationListColorTable> {
    let rule = find_tabs_navigation_list_color_rule(stylesheet, enabled)
        .ok_or_else(|| anyhow::anyhow!("no matching tabs navigation list color rule"))?;
    let colors = resolve_tabs_navigation_list_color_rule(resolver, rule)?;
    Ok(TabsNavigationListColorTable { disabled_background: colors.disabled_background })
}

#[derive(Clone, Debug)]
pub struct TabsNavigationItemColorTable {
    pub label_color: ResolvedColor,
    pub indicator: Option<ResolvedColor>,
}

impl TabsNavigationItemColorTable {
    pub fn fallback() -> Self {
        Self { label_color: ResolvedColor::fallback_foreground(), indicator: None }
    }
}

pub fn resolve_tabs_navigation_item_colors(
    resolver: &LookResolver<'_>,
    active: bool,
    layer: InteractionLayer,
    focused: bool,
) -> anyhow::Result<TabsNavigationItemColorTable> {
    resolve_tabs_navigation_item_colors_with_stylesheet(resolver, embedded_stylesheet(), active, layer, focused)
}

pub fn resolve_tabs_navigation_item_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    active: bool,
    layer: InteractionLayer,
    focused: bool,
) -> anyhow::Result<TabsNavigationItemColorTable> {
    let rule = find_tabs_navigation_item_color_rule(stylesheet, active, layer, focused)
        .ok_or_else(|| anyhow::anyhow!("no matching tabs navigation item color rule"))?;
    let colors = resolve_tabs_navigation_item_color_rule(resolver, rule, layer)?;
    Ok(TabsNavigationItemColorTable { label_color: colors.label_color, indicator: colors.indicator })
}

pub fn tabs_navigation_list_appearance(mode: &ShadcnModeTokens, enabled: bool) -> TabsNavigationListAppearance {
    let ctx = AppearanceContext::new(mode, ThemeMode::Light, InteractionState::default());
    if mode.catalog.tokens.is_empty() {
        tabs_navigation_list_from_palette(&ctx, enabled)
    } else {
        tabs_navigation_list_from_catalog(&ctx, enabled)
            .unwrap_or_else(|err| panic!("tabs navigation list properties: {err}"))
    }
}

pub fn tabs_navigation_item_appearance(
    mode: &ShadcnModeTokens,
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

pub fn tabs_navigation_list_from_palette(ctx: &AppearanceContext, enabled: bool) -> TabsNavigationListAppearance {
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

pub fn tabs_navigation_item_from_palette(ctx: &AppearanceContext, active: bool) -> TabsNavigationItemAppearance {
    let state = ctx.state;
    let palette = ctx.palette();
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let size = ControlSize::Md;
    let layer = state.layer();

    let active_color = match layer {
        InteractionLayer::Disabled => palette.disabled_foreground,
        layer => ctx.resolve_color_state(crate::tokens::ShadcnToken::Primary, layer),
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

pub fn tabs_navigation_list_from_catalog(
    ctx: &AppearanceContext,
    enabled: bool,
) -> anyhow::Result<TabsNavigationListAppearance> {
    let catalog = ctx.catalog();
    let metrics = ctx.metrics();
    let size = ControlSize::Md;
    let resolver = LookResolver::new(catalog, ctx.theme_mode, "tabs_navigation_list");
    let colors = resolve_tabs_navigation_list_colors(&resolver, enabled)
        .unwrap_or_else(|_| TabsNavigationListColorTable::fallback());

    Ok(TabsNavigationListAppearance {
        background: if enabled {
            None
        } else {
            Some(colors.disabled_background.hsla())
        },
        border: None,
        radius: metrics.radius(size),
        padding: 0.0,
        gap: metrics.spacing.s5,
    })
}

pub fn tabs_navigation_item_from_catalog(
    ctx: &AppearanceContext,
    active: bool,
) -> anyhow::Result<TabsNavigationItemAppearance> {
    let state = ctx.state;
    let catalog = ctx.catalog();
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let size = ControlSize::Md;
    let layer = state.layer();
    let resolver = LookResolver::new(catalog, ctx.theme_mode, "tabs_navigation_item");
    let colors = resolve_tabs_navigation_item_colors(&resolver, active, layer, state.focused)
        .unwrap_or_else(|_| TabsNavigationItemColorTable::fallback());

    Ok(TabsNavigationItemAppearance {
        label_color: colors.label_color.hsla(),
        indicator: colors.indicator.map(|color| color.hsla()),
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

    use gpui_luma::theme::{InteractionState, ThemeMode};

    use crate::appearance_context::AppearanceContext;
    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::{tabs_navigation_item_from_catalog};

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
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let ctx = AppearanceContext::new(&mode, gpui_luma::theme::ThemeMode::Light, InteractionState::default());
        let active = tabs_navigation_item_from_catalog(&ctx, true).expect("active tab");
        let inactive = tabs_navigation_item_from_catalog(&ctx, false).expect("inactive tab");

        assert_eq!(active.label_color, catalog.color("primary").expect("primary"));
        assert_eq!(inactive.label_color, catalog.color("foreground").expect("foreground"));
        assert_eq!(active.indicator, Some(catalog.color("primary").expect("primary")));
    }
}
