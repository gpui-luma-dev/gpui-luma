//! Radio button property mappings (tweakcn / shadcn):
//!
//! | State     | Ring token | Dot token              |
//! |-----------|------------|------------------------|
//! | Unselected| outline    | `foreground`           |
//! | Selected  | `{style}`  | `{style}-foreground`   |
//! | Disabled  | `muted`    | `muted-foreground`     |

use crate::controls::radio_button::RadioButtonPalette;
use crate::theme::{InteractionLayer, InteractionState};

use super::focus::focus_adorner;
use super::resolve::{
    resolve_action_foreground, resolve_action_layer, resolve_color, resolve_label_color, resolve_outline_layer,
};
use super::RadixButtonStyle;
use super::catalog::CssTokenMap;
use super::mode::RadixModeTokens;
use super::palette::RadixPalette;

pub(crate) fn radio_button_appearance(
    mode: &RadixModeTokens,
    style: RadixButtonStyle,
    selected: bool,
    state: InteractionState,
) -> RadioButtonPalette {
    if mode.catalog.tokens.is_empty() {
        return radio_button_appearance_from_palette(
            &mode.palette,
            &mode.metrics,
            &mode.typography,
            style,
            selected,
            state,
        );
    }

    radio_button_appearance_from_catalog(&mode.catalog, &mode.metrics, &mode.typography, style, selected, state)
        .unwrap_or_else(|err| panic!("radio properties: {err}"))
}

fn radio_button_appearance_from_palette(
    palette: &RadixPalette,
    metrics: &crate::theme::MetricTokens,
    typography: &crate::theme::LumaTypography,
    style: RadixButtonStyle,
    selected: bool,
    state: InteractionState,
) -> RadioButtonPalette {
    let layer = state.layer();
    let checked_action = palette.action(style);
    let outline = palette.outline;

    let indicator_background = match layer {
        InteractionLayer::Disabled => palette.disabled_background,
        InteractionLayer::Pressed => outline.pressed_background,
        InteractionLayer::Hovered => outline.hover_background,
        InteractionLayer::Default => outline.background,
    };

    let selected_color = match layer {
        InteractionLayer::Disabled => palette.disabled_foreground,
        InteractionLayer::Pressed => checked_action.pressed_background,
        InteractionLayer::Hovered => checked_action.hover_background,
        InteractionLayer::Default => checked_action.background,
    };

    RadioButtonPalette {
        control_background: None,
        control_border: None,
        indicator_background,
        indicator_border: if selected && !state.disabled {
            selected_color
        } else {
            palette.border_default
        },
        dot_color: if state.disabled {
            palette.disabled_foreground
        } else if selected {
            checked_action.foreground
        } else {
            palette.app_foreground
        },
        label_color: if state.disabled {
            palette.disabled_foreground
        } else {
            palette.app_foreground
        },
        adorner: super::focus::focus_adorner_from_palette(palette, metrics, state.focused),
        label_typography: typography.text.label,
        label_font_family: typography.font.sans.family.clone().into(),
    }
}

pub(crate) fn radio_button_appearance_from_catalog(
    catalog: &CssTokenMap,
    metrics: &crate::theme::MetricTokens,
    typography: &crate::theme::LumaTypography,
    style: RadixButtonStyle,
    selected: bool,
    state: InteractionState,
) -> anyhow::Result<RadioButtonPalette> {
    let layer = state.layer();

    let indicator_background = match layer {
        InteractionLayer::Disabled => resolve_color(catalog, "muted")?,
        _ => resolve_outline_layer(catalog, layer)?,
    };

    let selected_color = match layer {
        InteractionLayer::Disabled => resolve_color(catalog, "muted-foreground")?,
        _ => resolve_action_layer(catalog, style, layer)?,
    };

    let indicator_border = if selected && !state.disabled {
        selected_color
    } else {
        resolve_color(catalog, "border")?
    };

    Ok(RadioButtonPalette {
        control_background: None,
        control_border: None,
        indicator_background,
        indicator_border,
        dot_color: dot_color(catalog, style, selected, state.disabled)?,
        label_color: resolve_label_color(catalog, state.disabled)?,
        adorner: focus_adorner(catalog, metrics, state.focused)?,
        label_typography: typography.text.label,
        label_font_family: typography.font.sans.family.clone().into(),
    })
}

fn dot_color(
    catalog: &CssTokenMap,
    style: RadixButtonStyle,
    selected: bool,
    disabled: bool,
) -> anyhow::Result<gpui::Hsla> {
    if disabled {
        return resolve_color(catalog, "muted-foreground");
    }
    if selected {
        return resolve_action_foreground(catalog, style);
    }
    resolve_color(catalog, "foreground")
}
