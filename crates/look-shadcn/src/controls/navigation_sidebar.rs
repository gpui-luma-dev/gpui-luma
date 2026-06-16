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

use crate::appearance_context::AppearanceContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::look::ShadcnLook;
use crate::stylesheet::{
    StylesheetConfig, embedded_stylesheet, find_navigation_sidebar_branch_color_rule,
    find_navigation_sidebar_container_color_rule, find_navigation_sidebar_item_color_rule,
    find_navigation_sidebar_section_color_rule, resolve_navigation_sidebar_branch_color_rule,
    resolve_navigation_sidebar_container_color_rule, resolve_navigation_sidebar_item_color_rule,
    resolve_navigation_sidebar_section_color_rule,
};
use crate::tokens::ShadcnTextSize;

#[derive(Clone, Debug)]
pub struct NavigationSidebarContainerColorTable {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
}

impl NavigationSidebarContainerColorTable {
    pub fn fallback() -> Self {
        Self {
            background: ResolvedColor::transparent(),
            foreground: ResolvedColor::fallback_foreground(),
            border: ResolvedColor::fallback_foreground(),
        }
    }
}

pub fn resolve_navigation_sidebar_container_colors(
    resolver: &LookResolver<'_>,
    _present: bool,
) -> anyhow::Result<NavigationSidebarContainerColorTable> {
    resolve_navigation_sidebar_container_colors_with_stylesheet(resolver, embedded_stylesheet())
}

pub fn resolve_navigation_sidebar_container_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
) -> anyhow::Result<NavigationSidebarContainerColorTable> {
    let rule = find_navigation_sidebar_container_color_rule(stylesheet)
        .ok_or_else(|| anyhow::anyhow!("no matching navigation sidebar container color rule"))?;
    let colors = resolve_navigation_sidebar_container_color_rule(resolver, rule)?;
    Ok(NavigationSidebarContainerColorTable {
        background: colors.background,
        foreground: colors.foreground,
        border: colors.border,
    })
}

#[derive(Clone, Debug)]
pub struct NavigationSidebarSectionColorTable {
    pub label_color: ResolvedColor,
}

impl NavigationSidebarSectionColorTable {
    pub fn fallback() -> Self {
        Self { label_color: ResolvedColor::fallback_foreground() }
    }
}

pub fn resolve_navigation_sidebar_section_colors(
    resolver: &LookResolver<'_>,
    _present: bool,
) -> anyhow::Result<NavigationSidebarSectionColorTable> {
    resolve_navigation_sidebar_section_colors_with_stylesheet(resolver, embedded_stylesheet())
}

pub fn resolve_navigation_sidebar_section_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
) -> anyhow::Result<NavigationSidebarSectionColorTable> {
    let rule = find_navigation_sidebar_section_color_rule(stylesheet)
        .ok_or_else(|| anyhow::anyhow!("no matching navigation sidebar section color rule"))?;
    let colors = resolve_navigation_sidebar_section_color_rule(resolver, rule)?;
    Ok(NavigationSidebarSectionColorTable { label_color: colors.label_color })
}

#[derive(Clone, Debug)]
pub struct NavigationSidebarBranchColorTable {
    pub foreground: ResolvedColor,
    pub icon_color: ResolvedColor,
    pub background: Option<ResolvedColor>,
}

impl NavigationSidebarBranchColorTable {
    pub fn fallback() -> Self {
        Self {
            foreground: ResolvedColor::fallback_foreground(),
            icon_color: ResolvedColor::fallback_foreground(),
            background: None,
        }
    }
}

pub fn resolve_navigation_sidebar_branch_colors(
    resolver: &LookResolver<'_>,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<NavigationSidebarBranchColorTable> {
    resolve_navigation_sidebar_branch_colors_with_stylesheet(resolver, embedded_stylesheet(), disabled, layer)
}

pub fn resolve_navigation_sidebar_branch_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<NavigationSidebarBranchColorTable> {
    let rule = find_navigation_sidebar_branch_color_rule(stylesheet, disabled, layer)
        .ok_or_else(|| anyhow::anyhow!("no matching navigation sidebar branch color rule"))?;
    let colors = resolve_navigation_sidebar_branch_color_rule(resolver, rule, layer)?;
    Ok(NavigationSidebarBranchColorTable {
        foreground: colors.foreground,
        icon_color: colors.icon_color,
        background: colors.background,
    })
}

#[derive(Clone, Debug)]
pub struct NavigationSidebarItemColorTable {
    pub foreground: ResolvedColor,
    pub icon_color: ResolvedColor,
    pub background: Option<ResolvedColor>,
}

impl NavigationSidebarItemColorTable {
    pub fn fallback() -> Self {
        Self {
            foreground: ResolvedColor::fallback_foreground(),
            icon_color: ResolvedColor::fallback_foreground(),
            background: None,
        }
    }
}

pub fn resolve_navigation_sidebar_item_colors(
    resolver: &LookResolver<'_>,
    selected: bool,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<NavigationSidebarItemColorTable> {
    resolve_navigation_sidebar_item_colors_with_stylesheet(resolver, embedded_stylesheet(), selected, disabled, layer)
}

pub fn resolve_navigation_sidebar_item_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    selected: bool,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<NavigationSidebarItemColorTable> {
    let rule = find_navigation_sidebar_item_color_rule(stylesheet, selected, disabled, layer)
        .ok_or_else(|| anyhow::anyhow!("no matching navigation sidebar item color rule"))?;
    let colors = resolve_navigation_sidebar_item_color_rule(resolver, rule, layer)?;
    Ok(NavigationSidebarItemColorTable {
        foreground: colors.foreground,
        icon_color: colors.icon_color,
        background: colors.background,
    })
}

pub fn navigation_sidebar_container_appearance(mode: &ShadcnModeTokens) -> NavigationSidebarContainerAppearance {
    let ctx = AppearanceContext::new(mode, ThemeMode::Light, InteractionState::default());
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "navigation_sidebar_container");
    let colors = resolve_navigation_sidebar_container_colors(&resolver, true)
        .unwrap_or_else(|_| NavigationSidebarContainerColorTable::fallback());
    NavigationSidebarContainerAppearance {
        background: colors.background.hsla(),
        foreground: colors.foreground.hsla(),
        border: colors.border.hsla(),
    }
}

pub fn navigation_sidebar_section_appearance(theme: &ShadcnLook) -> NavigationSidebarSectionAppearance {
    let tokens = theme.mode_tokens();
    let ctx = AppearanceContext::new(tokens.as_ref(), theme.mode(), InteractionState::default());
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "navigation_sidebar_section");
    let colors = resolve_navigation_sidebar_section_colors(&resolver, true)
        .unwrap_or_else(|_| NavigationSidebarSectionColorTable::fallback());
    NavigationSidebarSectionAppearance {
        label_color: colors.label_color.hsla(),
        typography: theme.typography_scale(ShadcnTextSize::Xs),
        height: 20.0,
    }
}

fn base_item_appearance(
    _theme: &ShadcnLook,
    ctx: &AppearanceContext,
    size: ControlSize,
) -> NavigationSidebarItemAppearance {
    let state = ctx.state;
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let size_metrics = metrics.for_size(size);
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "navigation_sidebar_item_base");
    let focus_ring = state
        .focused
        .then(|| resolver.resolve_first_decl(&["sidebar-ring", "ring"]))
        .transpose()
        .unwrap_or_else(|err| panic!("navigation sidebar item properties: {err}"))
        .map(|color| color.hsla());
    let colors = resolve_navigation_sidebar_branch_colors(&resolver, state.disabled, state.layer())
        .unwrap_or_else(|_| NavigationSidebarBranchColorTable::fallback());

    NavigationSidebarItemAppearance {
        background: None,
        foreground: colors.foreground.hsla(),
        icon_color: colors.icon_color.hsla(),
        focus_ring,
        typography: typography.text.label,
        radius: metrics.radius(size),
        height: 30.0,
        padding_x: 8.0,
        gap: size_metrics.gap,
        icon_size: 16.0,
    }
}

pub fn navigation_sidebar_branch_appearance(
    theme: &ShadcnLook,
    state: InteractionState,
    size: ControlSize,
) -> NavigationSidebarItemAppearance {
    let tokens = theme.mode_tokens();
    let ctx = AppearanceContext::new(tokens.as_ref(), theme.mode(), state);
    let mut appearance = base_item_appearance(theme, &ctx, size);
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "navigation_sidebar_branch");
    let colors = resolve_navigation_sidebar_branch_colors(&resolver, state.disabled, state.layer())
        .unwrap_or_else(|_| NavigationSidebarBranchColorTable::fallback());

    appearance.background = colors.background.map(|color| color.hsla());
    appearance.foreground = colors.foreground.hsla();
    appearance.icon_color = colors.icon_color.hsla();

    appearance
}

pub fn navigation_sidebar_item_appearance(
    theme: &ShadcnLook,
    selected: bool,
    state: InteractionState,
    size: ControlSize,
) -> NavigationSidebarItemAppearance {
    let tokens = theme.mode_tokens();
    let ctx = AppearanceContext::new(tokens.as_ref(), theme.mode(), state);
    let mut appearance = base_item_appearance(theme, &ctx, size);
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "navigation_sidebar_item");
    let colors = resolve_navigation_sidebar_item_colors(&resolver, selected, state.disabled, state.layer())
        .unwrap_or_else(|_| NavigationSidebarItemColorTable::fallback());

    appearance.background = colors.background.map(|color| color.hsla());
    appearance.foreground = colors.foreground.hsla();
    appearance.icon_color = colors.icon_color.hsla();

    appearance
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};

    use crate::catalog::CssTokenMap;
    use crate::look::ShadcnLook;
    use crate::mode::ShadcnModeTokens;
    use super::{
        navigation_sidebar_container_appearance, navigation_sidebar_item_appearance,
        navigation_sidebar_section_appearance,
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
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = ShadcnLook::from_css_str(
            ":root { --background: oklch(0.9735 0.0261 90.0953); --foreground: oklch(0.3092 0.0518 219.6516); --card: oklch(0.9306 0.0260 92.4020); --card-foreground: oklch(0.3092 0.0518 219.6516); --muted: oklch(0.6979 0.0159 196.7940); --muted-foreground: oklch(0.3092 0.0518 219.6516); --border: oklch(0.6537 0.0197 205.2618); --input: oklch(0.6537 0.0197 205.2618); --ring: oklch(0.5924 0.2025 355.8943); --sidebar: oklch(0.9306 0.0260 92.4020); --sidebar-foreground: oklch(0.3092 0.0518 219.6516); --sidebar-primary: oklch(0.5924 0.2025 355.8943); --sidebar-primary-foreground: oklch(1 0 0); --sidebar-accent: oklch(0.5808 0.1732 39.5003); --sidebar-accent-foreground: oklch(1 0 0); --sidebar-border: oklch(0.6537 0.0197 205.2618); --sidebar-ring: oklch(0.5924 0.2025 355.8943); }"
        ).expect("look");
        let container = navigation_sidebar_container_appearance(&mode);
        let section = navigation_sidebar_section_appearance(&look);
        let selected = navigation_sidebar_item_appearance(&look, true, InteractionState::default(), ControlSize::Md);

        assert_eq!(container.background, catalog.color("sidebar").expect("sidebar"));
        assert_eq!(section.label_color, catalog.color("muted-foreground").expect("muted-foreground"));
        assert_eq!(selected.background, Some(catalog.color("sidebar-primary").expect("sidebar-primary")));
        assert_eq!(
            selected.foreground,
            catalog.color("sidebar-primary-foreground").expect("sidebar-primary-foreground")
        );
    }
}
