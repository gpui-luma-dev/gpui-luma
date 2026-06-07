//! Split view property mappings:
//!
//! | Part              | Token              |
//! |-------------------|--------------------|
//! | Separator         | `border`           |
//! | Separator (hover) | `border` (strong)  |
//! | Disabled          | `muted-foreground` |

use gpui_luma::controls::split_view::SplitViewAppearance;
use gpui_luma::theme::ThemeMode;

use crate::appearance_context::AppearanceContext;
use crate::mode::ShadcnModeTokens;
use crate::resolve::{resolve_color, resolve_color_layer};

pub(crate) fn split_view_appearance(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    hovered: bool,
    enabled: bool,
) -> SplitViewAppearance {
    let state = if enabled {
        gpui_luma::theme::InteractionState::default()
    } else {
        gpui_luma::theme::InteractionState { disabled: true, ..Default::default() }
    };
    let ctx = AppearanceContext::new(mode, theme_mode, state);
    if mode.catalog.tokens.is_empty() {
        split_view_from_palette(&ctx, hovered, enabled)
    } else {
        split_view_from_catalog(&ctx, hovered, enabled).unwrap_or_else(|err| panic!("split view properties: {err}"))
    }
}

fn split_view_from_palette(ctx: &AppearanceContext, _hovered: bool, enabled: bool) -> SplitViewAppearance {
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

fn split_view_from_catalog(
    ctx: &AppearanceContext,
    _hovered: bool,
    enabled: bool,
) -> anyhow::Result<SplitViewAppearance> {
    let catalog = ctx.catalog();
    let layer = ctx.state.layer();
    let separator = if enabled {
        resolve_color(catalog, "border")?
    } else {
        resolve_color(catalog, "muted-foreground")?
    };
    let separator_hover = if enabled {
        resolve_color_layer(catalog, "border", layer, true, ctx.theme_mode)?
    } else {
        resolve_color(catalog, "muted-foreground")?
    };

    Ok(SplitViewAppearance { separator, separator_hover })
}
