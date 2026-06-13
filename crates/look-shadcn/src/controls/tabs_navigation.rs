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

pub fn tabs_navigation_list_appearance(
    mode: &ShadcnModeTokens,
    enabled: bool,
    size: ControlSize,
) -> TabsNavigationListAppearance {
    let ctx = AppearanceContext::new(mode, ThemeMode::Light, InteractionState::default());
    let metrics = ctx.metrics();
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "tabs_navigation_list");
    let colors = resolve_tabs_navigation_list_colors(&resolver, enabled)
        .unwrap_or_else(|_| TabsNavigationListColorTable::fallback());

    TabsNavigationListAppearance {
        background: if enabled {
            None
        } else {
            Some(colors.disabled_background.hsla())
        },
        border: None,
        radius: metrics.radius(size),
        padding: 0.0,
        gap: metrics.gap(size),
    }
}

pub fn tabs_navigation_item_appearance(
    mode: &ShadcnModeTokens,
    active: bool,
    state: InteractionState,
    size: ControlSize,
) -> TabsNavigationItemAppearance {
    let ctx = AppearanceContext::new(mode, ThemeMode::Light, state);
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let layer = state.layer();
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "tabs_navigation_item");
    let colors = resolve_tabs_navigation_item_colors(&resolver, active, layer, state.focused)
        .unwrap_or_else(|_| TabsNavigationItemColorTable::fallback());
    let label_typography = match size {
        ControlSize::Sm => typography.text.caption,
        ControlSize::Md => typography.text.label,
        ControlSize::Lg => typography.text.body,
    };

    TabsNavigationItemAppearance {
        label_color: colors.label_color.hsla(),
        indicator: colors.indicator.map(|color| color.hsla()),
        label_typography,
        radius: metrics.radius(size),
        padding_x: metrics.padding_x(size),
        height: label_typography.line_height + metrics.padding_y(size) * 2.0,
        indicator_height: 2.0,
    }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::tabs_navigation_item_appearance;

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
        let active = tabs_navigation_item_appearance(&mode, true, InteractionState::default(), ControlSize::Md);
        let inactive = tabs_navigation_item_appearance(&mode, false, InteractionState::default(), ControlSize::Md);

        assert_eq!(active.label_color, catalog.color("primary").expect("primary"));
        assert_eq!(inactive.label_color, catalog.color("foreground").expect("foreground"));
        assert_eq!(active.indicator, Some(catalog.color("primary").expect("primary")));
    }

    #[test]
    fn tab_typography_changes_with_size() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog, ThemeMode::Light).expect("catalog");
        let small = tabs_navigation_item_appearance(&mode, false, InteractionState::default(), ControlSize::Sm);
        let medium = tabs_navigation_item_appearance(&mode, false, InteractionState::default(), ControlSize::Md);
        let large = tabs_navigation_item_appearance(&mode, false, InteractionState::default(), ControlSize::Lg);

        assert!(small.label_typography.size < medium.label_typography.size);
        assert!(medium.label_typography.size < large.label_typography.size);
    }
}
