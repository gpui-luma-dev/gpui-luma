//! Selector property mappings — ghost trigger (popup menu) + selector items panel.

use crate::controls::selector::SelectorPalette;
use crate::theme::{ControlSize, InteractionLayer, InteractionState, ThemeMode};

use super::focus::focus_ring_color;
use super::resolve::{resolve_color, resolve_ghost_background, resolve_label_color};
use super::selector_items_panel::selector_items_panel_appearance;
use super::catalog::CssTokenMap;
use super::mode::RadixModeTokens;
use super::palette::RadixPalette;

pub(crate) fn selector_palette(
    mode: &RadixModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> SelectorPalette {
    if mode.catalog.tokens.is_empty() {
        selector_palette_from_palette(&mode.palette, mode, theme_mode, state)
    } else {
        selector_palette_from_catalog(&mode.catalog, mode, theme_mode, state)
            .unwrap_or_else(|err| panic!("selector properties: {err}"))
    }
}

fn selector_palette_from_palette(
    palette: &RadixPalette,
    mode: &RadixModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> SelectorPalette {
    let typography = &mode.typography;
    let size = ControlSize::Md;
    let layer = state.layer();
    let ghost = palette.ghost;

    let trigger_background = match layer {
        InteractionLayer::Disabled => palette.disabled_background,
        InteractionLayer::Pressed => ghost.pressed_background,
        InteractionLayer::Hovered => ghost.hover_background,
        InteractionLayer::Default => ghost.background,
    };

    SelectorPalette {
        trigger_background,
        trigger_foreground: if state.disabled {
            palette.disabled_foreground
        } else {
            ghost.foreground
        },
        trigger_border: palette.border_default,
        focus_ring: state.focused.then_some(palette.focus_ring),
        trigger_typography: typography.text.label,
        items_panel: selector_items_panel_appearance(mode, theme_mode, size),
    }
}

pub(crate) fn selector_palette_from_catalog(
    catalog: &CssTokenMap,
    mode: &RadixModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> anyhow::Result<SelectorPalette> {
    let typography = &mode.typography;
    let layer = state.layer();

    Ok(SelectorPalette {
        trigger_background: resolve_ghost_background(catalog, layer)?,
        trigger_foreground: resolve_label_color(catalog, state.disabled)?,
        trigger_border: resolve_color(catalog, "border")?,
        focus_ring: state.focused.then(|| focus_ring_color(catalog)).transpose()?,
        trigger_typography: typography.text.label,
        items_panel: selector_items_panel_appearance(mode, theme_mode, ControlSize::Md),
    })
}
