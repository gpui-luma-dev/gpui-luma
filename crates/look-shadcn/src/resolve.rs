use gpui::{Hsla, hsla};

use gpui_luma::theme::{InteractionLayer, ThemeMode};

use super::catalog::CssTokenMap;
use super::state_color::algorithmic_state_color;

pub(crate) fn resolve_color(catalog: &CssTokenMap, token: &str) -> anyhow::Result<Hsla> {
    catalog.color(token)
}

pub(crate) fn resolve_label_color(catalog: &CssTokenMap, disabled: bool) -> anyhow::Result<Hsla> {
    if disabled {
        resolve_color(catalog, "muted-foreground")
    } else {
        resolve_color(catalog, "foreground")
    }
}

pub(crate) fn resolve_accent_foreground(catalog: &CssTokenMap) -> anyhow::Result<Hsla> {
    catalog.color_first(&["accent-foreground", "foreground"])
}

/// Ghost-style trigger (selector, popup, context menu): hover keeps transparent background.
pub(crate) fn resolve_ghost_trigger_background(catalog: &CssTokenMap, layer: InteractionLayer) -> anyhow::Result<Hsla> {
    match layer {
        InteractionLayer::Disabled => resolve_color(catalog, "muted"),
        InteractionLayer::Pressed | InteractionLayer::Hovered | InteractionLayer::Default => {
            Ok(hsla(0.0, 0.0, 0.0, 0.0))
        }
    }
}

pub(crate) fn resolve_ghost_trigger_foreground(
    catalog: &CssTokenMap,
    layer: InteractionLayer,
    disabled: bool,
) -> anyhow::Result<Hsla> {
    if disabled {
        resolve_color(catalog, "muted-foreground")
    } else if matches!(layer, InteractionLayer::Hovered | InteractionLayer::Pressed) {
        resolve_accent_foreground(catalog)
    } else {
        resolve_label_color(catalog, false)
    }
}

/// Pressed state derived from a hover tint without component-local color math.
pub(crate) fn resolve_whisper_pressed(hover: Hsla, theme_mode: ThemeMode) -> Hsla {
    algorithmic_state_color(hover, InteractionLayer::Pressed, theme_mode, false)
}
