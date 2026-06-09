//! Context menu property mappings:
//!
//! | Part   | Token                              |
//! |--------|------------------------------------|
//! | Target | ghost (accent-foreground on hover) |
//! | Menu   | floating menu surface              |

use gpui_luma::controls::context_menu::ContextMenuAppearance;
use gpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, ThemeMode};

use crate::appearance_context::AppearanceContext;
use super::floating_menu::floating_menu_appearance;
use crate::focus::focus_ring_color;
use crate::resolve::{resolve_color, resolve_ghost_trigger_background, resolve_ghost_trigger_foreground};
use crate::mode::ShadcnModeTokens;

pub fn context_menu_appearance(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> ContextMenuAppearance {
    let ctx = AppearanceContext::new(mode, theme_mode, state);
    if mode.catalog.tokens.is_empty() {
        context_menu_appearance_from_palette(&ctx)
    } else {
        context_menu_appearance_from_catalog(&ctx).unwrap_or_else(|err| panic!("context menu properties: {err}"))
    }
}

pub fn context_menu_appearance_from_palette(ctx: &AppearanceContext) -> ContextMenuAppearance {
    let state = ctx.state;
    let palette = ctx.palette();
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let size = ControlSize::Md;
    let layer = state.layer();
    let ghost = palette.ghost;

    let target_background = match layer {
        InteractionLayer::Disabled => palette.disabled_background,
        InteractionLayer::Pressed | InteractionLayer::Hovered | InteractionLayer::Default => ghost.background,
    };

    ContextMenuAppearance {
        target_background,
        target_foreground: if state.disabled {
            palette.disabled_foreground
        } else if matches!(layer, InteractionLayer::Hovered | InteractionLayer::Pressed) {
            palette.primary.foreground
        } else {
            ghost.foreground
        },
        target_border: palette.border_default,
        focus_ring: state.focused.then_some(palette.focus_ring),
        target_typography: typography.text.label,
        target_radius: metrics.radius(size),
        target_padding_x: metrics.padding_x(size),
        target_padding_y: metrics.padding_y(size),
        target_min_width: 200.0,
        floating_menu: floating_menu_appearance(ctx.tokens, ctx.theme_mode, size),
    }
}

pub fn context_menu_appearance_from_catalog(ctx: &AppearanceContext) -> anyhow::Result<ContextMenuAppearance> {
    let state = ctx.state;
    let catalog = ctx.catalog();
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let size = ControlSize::Md;
    let layer = state.layer();

    Ok(ContextMenuAppearance {
        target_background: resolve_ghost_trigger_background(catalog, layer)?,
        target_foreground: resolve_ghost_trigger_foreground(catalog, layer, state.disabled)?,
        target_border: resolve_color(catalog, "border")?,
        focus_ring: state.focused.then(|| focus_ring_color(catalog)).transpose()?,
        target_typography: typography.text.label,
        target_radius: metrics.radius(size),
        target_padding_x: metrics.padding_x(size),
        target_padding_y: metrics.padding_y(size),
        target_min_width: 200.0,
        floating_menu: floating_menu_appearance(ctx.tokens, ctx.theme_mode, size),
    })
}

