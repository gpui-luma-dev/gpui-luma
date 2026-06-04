//! Selector property mappings — ghost trigger (foreground-only hover) + accent item panel.

use crate::controls::selector::SelectorPalette;
use crate::theme::{ControlSize, InteractionLayer, InteractionState, ThemeMode};

use super::context::AppearanceContext;
use super::focus::focus_ring_color;
use super::resolve::{resolve_color, resolve_ghost_trigger_background, resolve_ghost_trigger_foreground};
use super::selector_items_panel::selector_items_panel_appearance;
use super::mode::RadixModeTokens;

pub(crate) fn selector_palette(
    mode: &RadixModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> SelectorPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, state);
    if mode.catalog.tokens.is_empty() {
        selector_palette_from_palette(&ctx)
    } else {
        selector_palette_from_catalog(&ctx).unwrap_or_else(|err| panic!("selector properties: {err}"))
    }
}

fn selector_palette_from_palette(ctx: &AppearanceContext) -> SelectorPalette {
    let state = ctx.state;
    let palette = ctx.palette();
    let typography = ctx.typography();
    let layer = state.layer();
    let ghost = palette.ghost;

    let trigger_background = match layer {
        InteractionLayer::Disabled => palette.disabled_background,
        InteractionLayer::Pressed | InteractionLayer::Hovered | InteractionLayer::Default => ghost.background,
    };

    SelectorPalette {
        trigger_background,
        trigger_foreground: if state.disabled {
            palette.disabled_foreground
        } else if matches!(layer, InteractionLayer::Hovered | InteractionLayer::Pressed) {
            palette.primary.foreground
        } else {
            ghost.foreground
        },
        trigger_border: palette.border_default,
        focus_ring: state.focused.then_some(palette.focus_ring),
        trigger_typography: typography.text.label,
        items_panel: selector_items_panel_appearance(ctx.tokens, ctx.theme_mode, ControlSize::Md),
    }
}

pub(crate) fn selector_palette_from_catalog(ctx: &AppearanceContext) -> anyhow::Result<SelectorPalette> {
    let state = ctx.state;
    let catalog = ctx.catalog();
    let typography = ctx.typography();
    let layer = state.layer();

    Ok(SelectorPalette {
        trigger_background: resolve_ghost_trigger_background(catalog, layer)?,
        trigger_foreground: resolve_ghost_trigger_foreground(catalog, layer, state.disabled)?,
        trigger_border: resolve_color(catalog, "border")?,
        focus_ring: state.focused.then(|| focus_ring_color(catalog)).transpose()?,
        trigger_typography: typography.text.label,
        items_panel: selector_items_panel_appearance(ctx.tokens, ctx.theme_mode, ControlSize::Md),
    })
}
