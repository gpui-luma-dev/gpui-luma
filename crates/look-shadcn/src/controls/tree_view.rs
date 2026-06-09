//! Tree view row tokens (shadcn / tweakcn sidebar branch styling):
//!
//! | Part           | Token                              |
//! |----------------|------------------------------------|
//! | Row label      | `sidebar-foreground`               |
//! | Row hover bg   | `sidebar-accent` (layer)         |
//! | Row pressed bg | `accent` (layer)                   |
//! | Disabled label | `muted-foreground`                 |
//! | Icon / chevron | `sidebar-foreground`               |

use gpui_luma::controls::tree_view::TreeViewPalette;
use gpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, ThemeMode};

use crate::appearance_context::AppearanceContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::stylesheet::{
    StylesheetConfig, embedded_stylesheet, find_tree_view_row_color_rule, resolve_tree_view_row_color_rule,
};

use super::navigation_sidebar::navigation_sidebar_branch_appearance;

#[derive(Clone, Debug)]
pub struct TreeViewRowColorTable {
    pub foreground: ResolvedColor,
    pub icon_color: ResolvedColor,
    pub chevron_color: ResolvedColor,
    pub background: Option<ResolvedColor>,
}

impl TreeViewRowColorTable {
    pub fn fallback() -> Self {
        Self {
            foreground: ResolvedColor::fallback_foreground(),
            icon_color: ResolvedColor::fallback_foreground(),
            chevron_color: ResolvedColor::fallback_foreground(),
            background: None,
        }
    }
}

pub fn resolve_tree_view_row_colors(
    resolver: &LookResolver<'_>,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<TreeViewRowColorTable> {
    resolve_tree_view_row_colors_with_stylesheet(resolver, embedded_stylesheet(), disabled, layer)
}

pub fn resolve_tree_view_row_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<TreeViewRowColorTable> {
    let rule = find_tree_view_row_color_rule(stylesheet, disabled, layer)
        .ok_or_else(|| anyhow::anyhow!("no matching tree view row color rule"))?;
    let colors = resolve_tree_view_row_color_rule(resolver, rule, layer)?;
    Ok(TreeViewRowColorTable {
        foreground: colors.foreground,
        icon_color: colors.icon_color,
        chevron_color: colors.chevron_color,
        background: colors.background,
    })
}

pub fn tree_view_row_palette(mode: &ShadcnModeTokens, _selected: bool, state: InteractionState) -> TreeViewPalette {
    let ctx = AppearanceContext::new(mode, ThemeMode::Light, state);
    if mode.catalog.tokens.is_empty() {
        tree_view_row_from_palette(mode, state, &ctx)
    } else {
        tree_view_row_from_catalog(&ctx, state).unwrap_or_else(|err| panic!("tree view row properties: {err}"))
    }
}

pub fn tree_view_row_from_palette(
    mode: &ShadcnModeTokens,
    state: InteractionState,
    ctx: &AppearanceContext,
) -> TreeViewPalette {
    let appearance = navigation_sidebar_branch_appearance(mode, state, ControlSize::Md);
    let typography = ctx.typography();

    TreeViewPalette {
        background: appearance.background,
        foreground: appearance.foreground,
        icon_color: appearance.icon_color,
        chevron_color: appearance.icon_color,
        adorner: None,
        typography: typography.text.label,
        font_family: typography.font.sans.family.clone().into(),
    }
}

fn tree_view_row_from_catalog(ctx: &AppearanceContext, state: InteractionState) -> anyhow::Result<TreeViewPalette> {
    let typography = ctx.typography();
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "tree_view_row");
    let colors = resolve_tree_view_row_colors(&resolver, state.disabled, state.layer())
        .unwrap_or_else(|_| TreeViewRowColorTable::fallback());

    Ok(TreeViewPalette {
        background: colors.background.map(|color| color.hsla()),
        foreground: colors.foreground.hsla(),
        icon_color: colors.icon_color.hsla(),
        chevron_color: colors.chevron_color.hsla(),
        adorner: None,
        typography: typography.text.label,
        font_family: typography.font.sans.family.clone().into(),
    })
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use gpui_luma::theme::{InteractionLayer, ThemeMode};

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use crate::provenance::LookResolver;
    use super::{resolve_tree_view_row_colors, tree_view_row_palette};

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("primary-foreground".into(), "oklch(1 0 0)".into()),
            ("secondary".into(), "oklch(0.6437 0.1019 187.3840)".into()),
            ("secondary-foreground".into(), "oklch(1 0 0)".into()),
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("accent-foreground".into(), "oklch(1 0 0)".into()),
            ("background".into(), "oklch(0.9735 0.0261 90.0953)".into()),
            ("foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("muted".into(), "oklch(0.6979 0.0159 196.7940)".into()),
            ("muted-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("sidebar-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("sidebar-accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
        ]))
    }

    #[test]
    fn tree_view_hovered_row_uses_sidebar_accent_background() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let palette = tree_view_row_palette(
            &mode,
            false,
            gpui_luma::theme::InteractionState { hovered: true, ..Default::default() },
        );
        let resolver = LookResolver::new(&catalog, ThemeMode::Light, "test");
        let expected = resolve_tree_view_row_colors(&resolver, false, InteractionLayer::Hovered)
            .expect("row colors")
            .background
            .map(|color| color.hsla());
        assert_eq!(palette.background, expected);
    }
}
