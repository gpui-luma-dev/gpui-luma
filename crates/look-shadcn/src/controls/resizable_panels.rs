//! Resizable panels property mappings:
//!
//! | Part          | Token              |
//! |---------------|--------------------|
//! | Divider line  | `border`           |
//! | Panel border  | `border`           |
//! | Handle grip   | `border`           |
//! | Grip emphasis | `accent`           |
//! | Disabled      | `muted-foreground` |

use gpui_luma::controls::resizable_panels::ResizablePanelsAppearance;
use gpui_luma::theme::{InteractionLayer, InteractionState, ThemeMode};

use crate::appearance_context::AppearanceContext;
use crate::resolve::{resolve_color, resolve_color_layer};
use crate::mode::ShadcnModeTokens;

pub(crate) fn resizable_panels_appearance(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> ResizablePanelsAppearance {
    let ctx = AppearanceContext::new(mode, theme_mode, state);
    if mode.catalog.tokens.is_empty() {
        resizable_panels_from_palette(&ctx)
    } else {
        resizable_panels_from_catalog(&ctx).unwrap_or_else(|err| panic!("resizable panels properties: {err}"))
    }
}

fn resizable_panels_from_palette(ctx: &AppearanceContext) -> ResizablePanelsAppearance {
    let palette = ctx.palette();
    let disabled = ctx.state.layer() == InteractionLayer::Disabled;

    ResizablePanelsAppearance {
        border: palette.border_default,
        divider: if disabled {
            palette.disabled_foreground
        } else {
            palette.border_default
        },
        grip: if disabled {
            palette.disabled_foreground
        } else {
            palette.border_default
        },
        grip_emphasis: if disabled {
            palette.disabled_foreground
        } else {
            palette.accent_background
        },
        disabled_opacity: if disabled { 0.45 } else { 1.0 },
    }
}

fn resizable_panels_from_catalog(ctx: &AppearanceContext) -> anyhow::Result<ResizablePanelsAppearance> {
    let catalog = ctx.catalog();
    let state = ctx.state;
    let layer = state.layer();

    let border = resolve_color(catalog, "border")?;
    let divider = if state.disabled {
        resolve_color(catalog, "muted-foreground")?
    } else {
        border
    };
    let grip = if state.disabled {
        resolve_color(catalog, "muted-foreground")?
    } else {
        border
    };
    let grip_emphasis = if state.disabled {
        resolve_color(catalog, "muted-foreground")?
    } else {
        resolve_color_layer(catalog, "accent", layer, false, ctx.theme_mode)?
    };

    Ok(ResizablePanelsAppearance {
        border,
        divider,
        grip,
        grip_emphasis,
        disabled_opacity: if state.disabled { 0.45 } else { 1.0 },
    })
}
