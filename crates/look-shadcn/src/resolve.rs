use gpui::{Hsla, hsla};

use gpui_luma::theme::{InteractionLayer, ThemeMode};

use super::ShadcnButtonStyle;
use super::catalog::CssTokenMap;
use super::color::with_alpha;
use super::state_color::{algorithmic_state_color, catalog_state_color};

pub(crate) fn resolve_color(catalog: &CssTokenMap, token: &str) -> anyhow::Result<Hsla> {
    catalog.color(token)
}

pub(crate) fn resolve_color_layer(
    catalog: &CssTokenMap,
    token: &str,
    layer: InteractionLayer,
    filled: bool,
    theme_mode: ThemeMode,
) -> anyhow::Result<Hsla> {
    if let Some(color) = catalog_state_color(catalog, token, layer) {
        return Ok(color);
    }
    let base = resolve_color(catalog, token)?;
    Ok(algorithmic_state_color(base, layer, theme_mode, filled))
}

pub(crate) fn resolve_color_first_layer(
    catalog: &CssTokenMap,
    tokens: &[&str],
    layer: InteractionLayer,
    filled: bool,
    theme_mode: ThemeMode,
) -> anyhow::Result<Hsla> {
    let base = catalog.color_first(tokens)?;
    if let Some(color) = tokens.iter().find_map(|token| catalog_state_color(catalog, token, layer)) {
        return Ok(color);
    }
    Ok(algorithmic_state_color(base, layer, theme_mode, filled))
}

pub(crate) fn resolve_action_layer(
    catalog: &CssTokenMap,
    style: ShadcnButtonStyle,
    layer: InteractionLayer,
    theme_mode: ThemeMode,
) -> anyhow::Result<Hsla> {
    let (background, _) = super::action::style_token_pair(style);
    resolve_color_layer(catalog, background, layer, true, theme_mode)
}

pub(crate) fn resolve_action_foreground(catalog: &CssTokenMap, style: ShadcnButtonStyle) -> anyhow::Result<Hsla> {
    let (_, foreground) = super::action::style_token_pair(style);
    resolve_color(catalog, foreground)
}

pub(crate) fn resolve_outline_layer(
    catalog: &CssTokenMap,
    layer: InteractionLayer,
    theme_mode: ThemeMode,
) -> anyhow::Result<Hsla> {
    match layer {
        InteractionLayer::Disabled => resolve_color(catalog, "muted"),
        InteractionLayer::Pressed => resolve_color(catalog, "muted"),
        InteractionLayer::Hovered => resolve_color_first_layer(catalog, &["accent", "muted"], layer, false, theme_mode),
        InteractionLayer::Default => {
            resolve_color_first_layer(catalog, &["card", "background"], layer, false, theme_mode)
        }
    }
}

pub(crate) fn resolve_label_color(catalog: &CssTokenMap, disabled: bool) -> anyhow::Result<Hsla> {
    if disabled {
        resolve_color(catalog, "muted-foreground")
    } else {
        resolve_color(catalog, "foreground")
    }
}

#[allow(dead_code)]
pub(crate) fn resolve_ghost_background(catalog: &CssTokenMap, layer: InteractionLayer) -> anyhow::Result<Hsla> {
    match layer {
        InteractionLayer::Disabled => resolve_color(catalog, "muted"),
        InteractionLayer::Pressed => resolve_color(catalog, "muted"),
        InteractionLayer::Hovered => catalog.color_first(&["accent", "muted"]),
        InteractionLayer::Default => Ok(hsla(0.0, 0.0, 0.0, 0.0)),
    }
}

pub(crate) fn resolve_popover_background(catalog: &CssTokenMap) -> anyhow::Result<Hsla> {
    catalog.color_first(&["popover", "card", "background"])
}

pub(crate) fn resolve_popover_foreground(catalog: &CssTokenMap) -> anyhow::Result<Hsla> {
    catalog.color_first(&["popover-foreground", "foreground"])
}

pub(crate) fn resolve_accent_foreground(catalog: &CssTokenMap) -> anyhow::Result<Hsla> {
    catalog.color_first(&["accent-foreground", "foreground"])
}

/// List/menu row hover: `accent` background and `accent-foreground` label.
pub(crate) fn resolve_accent_hover_pair(catalog: &CssTokenMap) -> anyhow::Result<(Hsla, Hsla)> {
    Ok((catalog.color_first(&["accent", "muted"])?, resolve_accent_foreground(catalog)?))
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

/// Faint accent tint for list-style row hovers (`bg-accent/40`).
pub(crate) fn resolve_accent_whisper(catalog: &CssTokenMap, alpha: f32) -> anyhow::Result<Hsla> {
    Ok(with_alpha(catalog.color_first(&["accent", "muted"])?, alpha))
}

/// Pressed state derived from a hover tint without component-local color math.
pub(crate) fn resolve_whisper_pressed(hover: Hsla, theme_mode: ThemeMode) -> Hsla {
    algorithmic_state_color(hover, InteractionLayer::Pressed, theme_mode, false)
}
