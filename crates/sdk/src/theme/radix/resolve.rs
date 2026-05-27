use gpui::{Hsla, hsla};

use crate::theme::InteractionLayer;

use super::RadixButtonStyle;
use super::catalog::CssTokenMap;
use super::color::darken;

pub(crate) fn resolve_color(catalog: &CssTokenMap, token: &str) -> anyhow::Result<Hsla> {
    catalog.color(token)
}

pub(crate) fn resolve_color_layer(
    catalog: &CssTokenMap,
    token: &str,
    layer: InteractionLayer,
    filled: bool,
) -> anyhow::Result<Hsla> {
    let base = resolve_color(catalog, token)?;
    Ok(match layer {
        InteractionLayer::Disabled => base,
        InteractionLayer::Pressed if filled => darken(base, 0.12),
        InteractionLayer::Hovered if filled => darken(base, 0.06),
        InteractionLayer::Pressed => darken(base, 0.08),
        InteractionLayer::Hovered => darken(base, 0.04),
        InteractionLayer::Default => base,
    })
}

pub(crate) fn resolve_color_first_layer(
    catalog: &CssTokenMap,
    tokens: &[&str],
    layer: InteractionLayer,
    filled: bool,
) -> anyhow::Result<Hsla> {
    let base = catalog.color_first(tokens)?;
    Ok(match layer {
        InteractionLayer::Disabled => base,
        InteractionLayer::Pressed if filled => darken(base, 0.12),
        InteractionLayer::Hovered if filled => darken(base, 0.06),
        InteractionLayer::Pressed => darken(base, 0.08),
        InteractionLayer::Hovered => darken(base, 0.04),
        InteractionLayer::Default => base,
    })
}

pub(crate) fn resolve_action_layer(
    catalog: &CssTokenMap,
    style: RadixButtonStyle,
    layer: InteractionLayer,
) -> anyhow::Result<Hsla> {
    let (background, _) = super::action::style_token_pair(style);
    resolve_color_layer(catalog, background, layer, true)
}

pub(crate) fn resolve_action_foreground(catalog: &CssTokenMap, style: RadixButtonStyle) -> anyhow::Result<Hsla> {
    let (_, foreground) = super::action::style_token_pair(style);
    resolve_color(catalog, foreground)
}

pub(crate) fn resolve_outline_layer(catalog: &CssTokenMap, layer: InteractionLayer) -> anyhow::Result<Hsla> {
    match layer {
        InteractionLayer::Disabled => resolve_color(catalog, "muted"),
        InteractionLayer::Pressed => resolve_color(catalog, "muted"),
        InteractionLayer::Hovered => resolve_color_first_layer(catalog, &["accent", "muted"], layer, false),
        InteractionLayer::Default => resolve_color_first_layer(catalog, &["card", "background"], layer, false),
    }
}

pub(crate) fn resolve_label_color(catalog: &CssTokenMap, disabled: bool) -> anyhow::Result<Hsla> {
    if disabled {
        resolve_color(catalog, "muted-foreground")
    } else {
        resolve_color(catalog, "foreground")
    }
}

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

pub(crate) fn resolve_accent_hover_pair(catalog: &CssTokenMap) -> anyhow::Result<(Hsla, Hsla)> {
    Ok((
        catalog.color_first(&["accent", "muted"])?,
        catalog.color_first(&["accent-foreground", "foreground"])?,
    ))
}
