//! Split view property mappings:
//!
//! | Part              | Token              |
//! |-------------------|--------------------|
//! | Separator         | `border`           |
//! | Separator (hover) | `border-hover`     |
//! | Disabled          | `muted-foreground` |

use gpui_luma::controls::split_view::SplitViewAppearance;
use gpui_luma::theme::{InteractionState, ThemeMode};

use crate::appearance_context::AppearanceContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, ResolvedColor};

use gpui_luma_look_shadcn_macros::declare_look_table;

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

declare_look_table! {
    name: resolve_split_view_colors,
    inputs: {
        enabled: bool,
    },
    output: SplitViewColorTable { separator, separator_hover },
    matrix: [
        [true]  => "border" | "border-hover",
        [false] => "muted-foreground" | "muted-foreground",
    ]
}

pub fn split_view_appearance(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    _hovered: bool,
    enabled: bool,
) -> SplitViewAppearance {
    let state = if enabled {
        InteractionState::default()
    } else {
        InteractionState { disabled: true, ..InteractionState::default() }
    };
    let ctx = AppearanceContext::new(mode, theme_mode, state);
    if mode.catalog.tokens.is_empty() {
        split_view_from_palette(&ctx, enabled)
    } else {
        split_view_from_catalog(&ctx, enabled).unwrap_or_else(|err| panic!("split view properties: {err}"))
    }
}

pub fn split_view_from_palette(ctx: &AppearanceContext, enabled: bool) -> SplitViewAppearance {
    let palette = ctx.palette();
    let separator = if enabled {
        palette.border_default
    } else {
        palette.disabled_foreground
    };
    let separator_hover = if enabled {
        palette.accent_background
    } else {
        palette.disabled_foreground
    };

    SplitViewAppearance { separator, separator_hover }
}

fn split_view_from_catalog(ctx: &AppearanceContext, enabled: bool) -> anyhow::Result<SplitViewAppearance> {
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "split_view");
    let colors = resolve_split_view_colors(&resolver, enabled).unwrap_or_else(|_| SplitViewColorTable::fallback());

    Ok(SplitViewAppearance { separator: colors.separator.hsla(), separator_hover: colors.separator_hover.hsla() })
}

#[cfg(test)]
mod tests {


    use std::collections::BTreeMap;

    use gpui_luma::theme::ThemeMode;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use crate::provenance::ColorSource;
    use super::{resolve_split_view_colors_metadata, split_view_appearance};

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
        let appearance = split_view_appearance(&mode, ThemeMode::Light, false, true);
        assert_eq!(appearance.separator, catalog.color("border").expect("border"));
    }

}
