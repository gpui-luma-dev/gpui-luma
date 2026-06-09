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

use gpui_luma_look_shadcn_macros::declare_look_table;

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

declare_look_table! {
    name: resolve_navigation_sidebar_container_colors,
    inputs: {
        present: bool,
    },
    output: NavigationSidebarContainerColorTable { background, foreground, border },
    matrix: [
        [true] => "first(sidebar,card,background)" | "first(sidebar-foreground,foreground)" | "first(sidebar-border,border)",
        [false] => "first(sidebar,card,background)" | "first(sidebar-foreground,foreground)" | "first(sidebar-border,border)",
    ]
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

declare_look_table! {
    name: resolve_navigation_sidebar_section_colors,
    inputs: {
        present: bool,
    },
    output: NavigationSidebarSectionColorTable { label_color },
    matrix: [
        [true] => "muted-foreground",
        [false] => "muted-foreground",
    ]
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

declare_look_table! {
    name: resolve_navigation_sidebar_branch_colors,
    inputs: {
        disabled: bool,
        layer: InteractionLayer,
    },
    output: NavigationSidebarBranchColorTable { foreground, icon_color, background },
    matrix: [
        [true] | [_] => "muted-foreground" | "muted-foreground" | None,

        [false] | [InteractionLayer::Default] => "first(sidebar-foreground,foreground)" | "first(sidebar-foreground,foreground)" | None,
        [false] | [InteractionLayer::Disabled] => "first(sidebar-foreground,foreground)" | "first(sidebar-foreground,foreground)" | None,
        [false] | [InteractionLayer::Hovered] => "accent-foreground" | "accent-foreground" | "accent",
        [false] | [InteractionLayer::Pressed] => "accent-foreground" | "accent-foreground" | "accent",
    ]
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

declare_look_table! {
    name: resolve_navigation_sidebar_item_colors,
    inputs: {
        selected: bool,
        disabled: bool,
        layer: InteractionLayer,
    },
    output: NavigationSidebarItemColorTable { foreground, icon_color, background },
    matrix: [
        [true] | [true] | [_] => "muted-foreground" | "muted-foreground" | None,
        [false] | [true] | [_] => "muted-foreground" | "muted-foreground" | None,

        [true] | [false] | [InteractionLayer::Default] => "first(sidebar-primary-foreground,primary-foreground)" | "first(sidebar-primary-foreground,primary-foreground)" | "first(sidebar-primary,primary)",
        [true] | [false] | [InteractionLayer::Hovered] => "first(sidebar-primary-foreground,primary-foreground)" | "first(sidebar-primary-foreground,primary-foreground)" | "first_layer(sidebar-primary,primary)",
        [true] | [false] | [InteractionLayer::Pressed] => "first(sidebar-primary-foreground,primary-foreground)" | "first(sidebar-primary-foreground,primary-foreground)" | "first_layer(sidebar-primary,primary)",
        [true] | [false] | [InteractionLayer::Disabled] => "first(sidebar-primary-foreground,primary-foreground)" | "first(sidebar-primary-foreground,primary-foreground)" | "first(sidebar-primary,primary)",

        [false] | [false] | [InteractionLayer::Default] => "first(sidebar-foreground,foreground)" | "first(sidebar-foreground,foreground)" | None,
        [false] | [false] | [InteractionLayer::Disabled] => "first(sidebar-foreground,foreground)" | "first(sidebar-foreground,foreground)" | None,
        [false] | [false] | [InteractionLayer::Hovered] => "accent-foreground" | "accent-foreground" | "accent",
        [false] | [false] | [InteractionLayer::Pressed] => "accent-foreground" | "accent-foreground" | "accent",
    ]
}

pub fn navigation_sidebar_container_appearance(mode: &ShadcnModeTokens) -> NavigationSidebarContainerAppearance {
    let ctx = AppearanceContext::new(mode, ThemeMode::Light, InteractionState::default());
    if mode.catalog.tokens.is_empty() {
        navigation_sidebar_container_from_palette(&ctx)
    } else {
        navigation_sidebar_container_from_catalog(&ctx)
            .unwrap_or_else(|err| panic!("navigation sidebar container properties: {err}"))
    }
}

pub fn navigation_sidebar_section_appearance(mode: &ShadcnModeTokens) -> NavigationSidebarSectionAppearance {
    let ctx = AppearanceContext::new(mode, ThemeMode::Light, InteractionState::default());
    if mode.catalog.tokens.is_empty() {
        navigation_sidebar_section_from_palette(&ctx)
    } else {
        navigation_sidebar_section_from_catalog(&ctx)
            .unwrap_or_else(|err| panic!("navigation sidebar section properties: {err}"))
    }
}

pub fn navigation_sidebar_branch_appearance(
    mode: &ShadcnModeTokens,
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

pub fn navigation_sidebar_item_appearance(
    mode: &ShadcnModeTokens,
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

pub fn navigation_sidebar_container_from_palette(ctx: &AppearanceContext) -> NavigationSidebarContainerAppearance {
    let palette = ctx.palette();
    NavigationSidebarContainerAppearance {
        background: palette.panel_background,
        foreground: palette.app_foreground,
        border: palette.border_default,
    }
}

pub fn navigation_sidebar_section_from_palette(ctx: &AppearanceContext) -> NavigationSidebarSectionAppearance {
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

pub fn navigation_sidebar_branch_from_palette(
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

pub fn navigation_sidebar_item_from_palette(
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

pub fn navigation_sidebar_container_from_catalog(
    ctx: &AppearanceContext,
) -> anyhow::Result<NavigationSidebarContainerAppearance> {
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "navigation_sidebar_container");
    let colors = resolve_navigation_sidebar_container_colors(&resolver, true)
        .unwrap_or_else(|_| NavigationSidebarContainerColorTable::fallback());
    Ok(NavigationSidebarContainerAppearance {
        background: colors.background.hsla(),
        foreground: colors.foreground.hsla(),
        border: colors.border.hsla(),
    })
}

pub fn navigation_sidebar_section_from_catalog(
    ctx: &AppearanceContext,
) -> anyhow::Result<NavigationSidebarSectionAppearance> {
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "navigation_sidebar_section");
    let colors = resolve_navigation_sidebar_section_colors(&resolver, true)
        .unwrap_or_else(|_| NavigationSidebarSectionColorTable::fallback());
    Ok(NavigationSidebarSectionAppearance {
        label_color: colors.label_color.hsla(),
        typography: ctx.typography().text.caption,
        height: 20.0,
    })
}

fn base_item_from_catalog(
    ctx: &AppearanceContext,
    size: ControlSize,
) -> anyhow::Result<NavigationSidebarItemAppearance> {
    let state = ctx.state;
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let size_metrics = metrics.for_size(size);
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "navigation_sidebar_item_base");
    let focus_ring = state
        .focused
        .then(|| resolver.resolve_first_decl(&["sidebar-ring", "ring"]))
        .transpose()?
        .map(|color| color.hsla());

    let colors = resolve_navigation_sidebar_branch_colors(&resolver, state.disabled, state.layer())
        .unwrap_or_else(|_| NavigationSidebarBranchColorTable::fallback());

    Ok(NavigationSidebarItemAppearance {
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
    })
}

pub fn navigation_sidebar_branch_from_catalog(
    ctx: &AppearanceContext,
    size: ControlSize,
) -> anyhow::Result<NavigationSidebarItemAppearance> {
    let state = ctx.state;
    let mut appearance = base_item_from_catalog(ctx, size)?;
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "navigation_sidebar_branch");
    let colors = resolve_navigation_sidebar_branch_colors(&resolver, state.disabled, state.layer())
        .unwrap_or_else(|_| NavigationSidebarBranchColorTable::fallback());

    appearance.background = colors.background.map(|color| color.hsla());
    appearance.foreground = colors.foreground.hsla();
    appearance.icon_color = colors.icon_color.hsla();

    Ok(appearance)
}

pub fn navigation_sidebar_item_from_catalog(
    ctx: &AppearanceContext,
    selected: bool,
    size: ControlSize,
) -> anyhow::Result<NavigationSidebarItemAppearance> {
    let state = ctx.state;
    let mut appearance = base_item_from_catalog(ctx, size)?;
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "navigation_sidebar_item");
    let colors = resolve_navigation_sidebar_item_colors(&resolver, selected, state.disabled, state.layer())
        .unwrap_or_else(|_| NavigationSidebarItemColorTable::fallback());

    appearance.background = colors.background.map(|color| color.hsla());
    appearance.foreground = colors.foreground.hsla();
    appearance.icon_color = colors.icon_color.hsla();

    Ok(appearance)
}

#[cfg(test)]
mod tests {


    use std::collections::BTreeMap;

    use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};

    use crate::appearance_context::AppearanceContext;
    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::{
        navigation_sidebar_container_from_catalog, navigation_sidebar_item_from_catalog,
        navigation_sidebar_section_from_catalog, resolve_navigation_sidebar_container_colors_metadata,
        resolve_navigation_sidebar_item_colors_metadata,
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
        let ctx = AppearanceContext::new(&mode, ThemeMode::Light, InteractionState::default());
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
