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

use luma::controls::sidebar::{SidebarContainerLook, SidebarItemLook, SidebarSectionLook};
use luma::theme::{ControlSize, InteractionLayer, InteractionState, ThemeMode};

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::look::ShadcnLook;
use crate::stylesheet::{
    StylesheetConfig, embedded_stylesheet, find_sidebar_branch_color_rule, find_sidebar_container_color_rule,
    find_sidebar_item_color_rule, find_sidebar_section_color_rule, resolve_sidebar_branch_color_rule,
    resolve_sidebar_container_color_rule, resolve_sidebar_item_color_rule, resolve_sidebar_section_color_rule,
};
use crate::tokens::ShadcnTextSize;

#[derive(Clone, Debug)]
pub struct SidebarContainerColorTable {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
}

impl SidebarContainerColorTable {
    pub fn fallback() -> Self {
        Self {
            background: ResolvedColor::transparent(),
            foreground: ResolvedColor::fallback_foreground(),
            border: ResolvedColor::fallback_foreground(),
        }
    }
}

pub fn resolve_sidebar_container_colors(
    resolver: &LookResolver<'_>,
    _present: bool,
) -> anyhow::Result<SidebarContainerColorTable> {
    resolve_sidebar_container_colors_with_stylesheet(resolver, embedded_stylesheet())
}

pub fn resolve_sidebar_container_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
) -> anyhow::Result<SidebarContainerColorTable> {
    let rule = find_sidebar_container_color_rule(stylesheet)
        .ok_or_else(|| anyhow::anyhow!("no matching navigation sidebar container color rule"))?;
    let colors = resolve_sidebar_container_color_rule(resolver, rule)?;
    Ok(SidebarContainerColorTable {
        background: colors.background,
        foreground: colors.foreground,
        border: colors.border,
    })
}

#[derive(Clone, Debug)]
pub struct SidebarSectionColorTable {
    pub label_color: ResolvedColor,
}

impl SidebarSectionColorTable {
    pub fn fallback() -> Self {
        Self { label_color: ResolvedColor::fallback_foreground() }
    }
}

pub fn resolve_sidebar_section_colors(
    resolver: &LookResolver<'_>,
    _present: bool,
) -> anyhow::Result<SidebarSectionColorTable> {
    resolve_sidebar_section_colors_with_stylesheet(resolver, embedded_stylesheet())
}

pub fn resolve_sidebar_section_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
) -> anyhow::Result<SidebarSectionColorTable> {
    let rule = find_sidebar_section_color_rule(stylesheet)
        .ok_or_else(|| anyhow::anyhow!("no matching navigation sidebar section color rule"))?;
    let colors = resolve_sidebar_section_color_rule(resolver, rule)?;
    Ok(SidebarSectionColorTable { label_color: colors.label_color })
}

#[derive(Clone, Debug)]
pub struct SidebarBranchColorTable {
    pub foreground: ResolvedColor,
    pub icon_color: ResolvedColor,
    pub background: Option<ResolvedColor>,
}

impl SidebarBranchColorTable {
    pub fn fallback() -> Self {
        Self {
            foreground: ResolvedColor::fallback_foreground(),
            icon_color: ResolvedColor::fallback_foreground(),
            background: None,
        }
    }
}

pub fn resolve_sidebar_branch_colors(
    resolver: &LookResolver<'_>,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<SidebarBranchColorTable> {
    resolve_sidebar_branch_colors_with_stylesheet(resolver, embedded_stylesheet(), disabled, layer)
}

pub fn resolve_sidebar_branch_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<SidebarBranchColorTable> {
    let rule = find_sidebar_branch_color_rule(stylesheet, disabled, layer)
        .ok_or_else(|| anyhow::anyhow!("no matching navigation sidebar branch color rule"))?;
    let colors = resolve_sidebar_branch_color_rule(resolver, rule, layer)?;
    Ok(SidebarBranchColorTable {
        foreground: colors.foreground,
        icon_color: colors.icon_color,
        background: colors.background,
    })
}

#[derive(Clone, Debug)]
pub struct SidebarItemColorTable {
    pub foreground: ResolvedColor,
    pub icon_color: ResolvedColor,
    pub background: Option<ResolvedColor>,
}

impl SidebarItemColorTable {
    pub fn fallback() -> Self {
        Self {
            foreground: ResolvedColor::fallback_foreground(),
            icon_color: ResolvedColor::fallback_foreground(),
            background: None,
        }
    }
}

pub fn resolve_sidebar_item_colors(
    resolver: &LookResolver<'_>,
    selected: bool,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<SidebarItemColorTable> {
    resolve_sidebar_item_colors_with_stylesheet(resolver, embedded_stylesheet(), selected, disabled, layer)
}

pub fn resolve_sidebar_item_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    selected: bool,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<SidebarItemColorTable> {
    let rule = find_sidebar_item_color_rule(stylesheet, selected, disabled, layer)
        .ok_or_else(|| anyhow::anyhow!("no matching navigation sidebar item color rule"))?;
    let colors = resolve_sidebar_item_color_rule(resolver, rule, layer)?;
    Ok(SidebarItemColorTable {
        foreground: colors.foreground,
        icon_color: colors.icon_color,
        background: colors.background,
    })
}

pub fn sidebar_container_look(mode: &ShadcnModeTokens) -> SidebarContainerLook {
    let ctx = LookContext::new(mode, ThemeMode::Light, InteractionState::default());
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "sidebar_container");
    let colors =
        resolve_sidebar_container_colors(&resolver, true).unwrap_or_else(|_| SidebarContainerColorTable::fallback());
    SidebarContainerLook {
        background: colors.background.hsla(),
        foreground: colors.foreground.hsla(),
        border: colors.border.hsla(),
        radius: mode.metrics.radius.lg,
    }
}

pub fn sidebar_section_look(theme: &ShadcnLook) -> SidebarSectionLook {
    let tokens = theme.mode_tokens();
    let ctx = LookContext::new(tokens.as_ref(), theme.mode(), InteractionState::default());
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "sidebar_section");
    let colors =
        resolve_sidebar_section_colors(&resolver, true).unwrap_or_else(|_| SidebarSectionColorTable::fallback());
    SidebarSectionLook {
        label_color: colors.label_color.hsla(),
        typography: theme.typography_scale(ShadcnTextSize::Xs),
        height: 20.0,
    }
}

fn base_item_look(_theme: &ShadcnLook, ctx: &LookContext, size: ControlSize) -> SidebarItemLook {
    let state = ctx.state;
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let size_metrics = metrics.for_size(size);
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "sidebar_item_base");
    let colors = resolve_sidebar_branch_colors(&resolver, state.disabled, state.layer())
        .unwrap_or_else(|_| SidebarBranchColorTable::fallback());

    SidebarItemLook {
        background: None,
        focus_border: (state.focused && !state.disabled).then(|| crate::focus::focus_ring_or_fallback(ctx.catalog())),
        foreground: colors.foreground.hsla(),
        icon_color: colors.icon_color.hsla(),
        typography: typography.text.label,
        radius: metrics.radius(size),
        height: 30.0,
        padding_x: 8.0,
        gap: size_metrics.gap,
        icon_size: 16.0,
    }
}

pub fn sidebar_branch_look(theme: &ShadcnLook, state: InteractionState, size: ControlSize) -> SidebarItemLook {
    let tokens = theme.mode_tokens();
    let ctx = LookContext::new(tokens.as_ref(), theme.mode(), state);
    let mut look = base_item_look(theme, &ctx, size);
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "sidebar_branch");
    let colors = resolve_sidebar_branch_colors(&resolver, state.disabled, state.layer())
        .unwrap_or_else(|_| SidebarBranchColorTable::fallback());

    look.background = colors.background.map(|color| color.hsla());
    look.foreground = colors.foreground.hsla();
    look.icon_color = colors.icon_color.hsla();

    look
}

pub fn sidebar_item_look(
    theme: &ShadcnLook,
    selected: bool,
    state: InteractionState,
    size: ControlSize,
) -> SidebarItemLook {
    let tokens = theme.mode_tokens();
    let ctx = LookContext::new(tokens.as_ref(), theme.mode(), state);
    let mut look = base_item_look(theme, &ctx, size);
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "sidebar_item");
    let colors = resolve_sidebar_item_colors(&resolver, selected, state.disabled, state.layer())
        .unwrap_or_else(|_| SidebarItemColorTable::fallback());

    look.background = colors.background.map(|color| color.hsla());
    look.foreground = colors.foreground.hsla();
    look.icon_color = colors.icon_color.hsla();

    look
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use luma::theme::{ControlSize, InteractionState, ThemeMode};

    use crate::catalog::CssTokenMap;
    use crate::look::ShadcnLook;
    use crate::mode::ShadcnModeTokens;
    use super::{sidebar_container_look, sidebar_item_look, sidebar_section_look};

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
    fn sidebar_uses_sidebar_tokens() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = ShadcnLook::from_css_str(
            ":root { --background: oklch(0.9735 0.0261 90.0953); --foreground: oklch(0.3092 0.0518 219.6516); --card: oklch(0.9306 0.0260 92.4020); --card-foreground: oklch(0.3092 0.0518 219.6516); --primary: oklch(0.5924 0.2025 355.8943); --primary-foreground: oklch(1 0 0); --secondary: oklch(0.6437 0.1019 187.3840); --secondary-foreground: oklch(1 0 0); --accent: oklch(0.5808 0.1732 39.5003); --accent-foreground: oklch(1 0 0); --muted: oklch(0.6979 0.0159 196.7940); --muted-foreground: oklch(0.3092 0.0518 219.6516); --border: oklch(0.6537 0.0197 205.2618); --input: oklch(0.6537 0.0197 205.2618); --ring: oklch(0.5924 0.2025 355.8943); --sidebar: oklch(0.9306 0.0260 92.4020); --sidebar-foreground: oklch(0.3092 0.0518 219.6516); --sidebar-primary: oklch(0.5924 0.2025 355.8943); --sidebar-primary-foreground: oklch(1 0 0); --sidebar-accent: oklch(0.5808 0.1732 39.5003); --sidebar-accent-foreground: oklch(1 0 0); --sidebar-border: oklch(0.6537 0.0197 205.2618); --sidebar-ring: oklch(0.5924 0.2025 355.8943); } .dark { --background: oklch(0.9735 0.0261 90.0953); --foreground: oklch(0.3092 0.0518 219.6516); --card: oklch(0.9306 0.0260 92.4020); --card-foreground: oklch(0.3092 0.0518 219.6516); --primary: oklch(0.5924 0.2025 355.8943); --primary-foreground: oklch(1 0 0); --secondary: oklch(0.6437 0.1019 187.3840); --secondary-foreground: oklch(1 0 0); --accent: oklch(0.5808 0.1732 39.5003); --accent-foreground: oklch(1 0 0); --muted: oklch(0.6979 0.0159 196.7940); --muted-foreground: oklch(0.3092 0.0518 219.6516); --border: oklch(0.6537 0.0197 205.2618); --input: oklch(0.6537 0.0197 205.2618); --ring: oklch(0.5924 0.2025 355.8943); --sidebar: oklch(0.9306 0.0260 92.4020); --sidebar-foreground: oklch(0.3092 0.0518 219.6516); --sidebar-primary: oklch(0.5924 0.2025 355.8943); --sidebar-primary-foreground: oklch(1 0 0); --sidebar-accent: oklch(0.5808 0.1732 39.5003); --sidebar-accent-foreground: oklch(1 0 0); --sidebar-border: oklch(0.6537 0.0197 205.2618); --sidebar-ring: oklch(0.5924 0.2025 355.8943); }"
        ).expect("look");
        let container = sidebar_container_look(&mode);
        let section = sidebar_section_look(&look);
        let selected = sidebar_item_look(&look, true, InteractionState::default(), ControlSize::Md);

        assert_eq!(container.background, catalog.color("sidebar").expect("sidebar"));
        assert_eq!(section.label_color, catalog.color("muted-foreground").expect("muted-foreground"));
        assert_eq!(selected.background, Some(catalog.color("sidebar-primary").expect("sidebar-primary")));
        assert_eq!(
            selected.foreground,
            catalog.color("sidebar-primary-foreground").expect("sidebar-primary-foreground")
        );
    }

    #[test]
    fn focused_sidebar_item_does_not_panic_when_ring_token_is_missing() {
        let mut mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        mode.catalog.tokens.remove("ring");
        let ctx = crate::look_context::LookContext::new(
            &mode,
            ThemeMode::Light,
            InteractionState { focused: true, ..InteractionState::default() },
        );
        let look = super::base_item_look(&ShadcnLook::built_in(), &ctx, ControlSize::Md);

        assert_eq!(look.focus_border, Some(crate::provenance::ResolvedColor::fallback_foreground().hsla()));
    }
}
