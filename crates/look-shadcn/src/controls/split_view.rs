//! Split view property mappings:
//!
//! | Part              | Token              |
//! |-------------------|--------------------|
//! | Separator         | `border`           |
//! | Separator (hover) | `border-hover`     |
//! | Disabled          | `muted-foreground` |

use gpui_luma::controls::split_view::SplitViewLook;
use gpui_luma::theme::{InteractionState, ThemeMode};

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::stylesheet::{StylesheetConfig, find_split_view_color_rule, resolve_split_view_color_rule};

#[derive(Clone, Debug)]
pub struct SplitViewColorTable {
    pub separator: ResolvedColor,
    pub separator_hover: ResolvedColor,
}

impl SplitViewColorTable {
    pub fn fallback() -> Self {
        Self { separator: ResolvedColor::transparent(), separator_hover: ResolvedColor::fallback_foreground() }
    }
}

pub fn resolve_split_view_colors(resolver: &LookResolver<'_>, enabled: bool) -> anyhow::Result<SplitViewColorTable> {
    resolve_split_view_colors_with_stylesheet(resolver, resolver.stylesheet(), enabled)
}

pub fn resolve_split_view_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    enabled: bool,
) -> anyhow::Result<SplitViewColorTable> {
    let rule = find_split_view_color_rule(stylesheet, enabled)
        .ok_or_else(|| anyhow::anyhow!("no matching split view color rule"))?;
    let colors = resolve_split_view_color_rule(resolver, rule)?;
    Ok(SplitViewColorTable { separator: colors.separator, separator_hover: colors.separator_hover })
}

pub fn split_view_look(mode: &ShadcnModeTokens, theme_mode: ThemeMode, _hovered: bool, enabled: bool) -> SplitViewLook {
    let state = if enabled {
        InteractionState::default()
    } else {
        InteractionState { disabled: true, ..InteractionState::default() }
    };
    let ctx = LookContext::new(mode, theme_mode, state);
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "split_view").with_stylesheet(mode.stylesheet());
    let colors = resolve_split_view_colors(&resolver, enabled).unwrap_or_else(|_| SplitViewColorTable::fallback());

    SplitViewLook { separator: colors.separator.hsla(), separator_hover: colors.separator_hover.hsla() }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use gpui_luma::theme::ThemeMode;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::split_view_look;

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
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
        ]))
    }

    #[test]
    fn enabled_split_view_uses_border_tokens() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = split_view_look(&mode, ThemeMode::Light, false, true);
        assert_eq!(look.separator, catalog.color("border").expect("border"));
    }
}
