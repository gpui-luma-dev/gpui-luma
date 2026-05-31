//! Checkbox property mappings (tweakcn / shadcn):
//!
//! | State    | Indicator token | Checkmark token        |
//! |----------|-----------------|------------------------|
//! | Unchecked| outline surface | `foreground`           |
//! | Checked  | `{style}`       | `{style}-foreground`   |
//! | Disabled | `muted`         | `muted-foreground`     |

use crate::controls::checkbox::CheckboxPalette;
use crate::theme::{InteractionLayer, InteractionState};

use super::focus::focus_adorner;
use super::resolve::{
    resolve_action_foreground, resolve_action_layer, resolve_color, resolve_label_color, resolve_outline_layer,
};
use super::RadixButtonStyle;
use super::catalog::CssTokenMap;
use super::mode::RadixModeTokens;
use super::palette::RadixPalette;

pub(crate) fn checkbox_appearance(
    mode: &RadixModeTokens,
    style: RadixButtonStyle,
    checked: bool,
    state: InteractionState,
) -> CheckboxPalette {
    if mode.catalog.tokens.is_empty() {
        return checkbox_appearance_from_palette(&mode.palette, &mode.metrics, &mode.typography, style, checked, state);
    }

    checkbox_appearance_from_catalog(&mode.catalog, &mode.metrics, &mode.typography, style, checked, state)
        .unwrap_or_else(|err| panic!("checkbox properties: {err}"))
}

fn checkbox_appearance_from_palette(
    palette: &RadixPalette,
    metrics: &crate::theme::MetricTokens,
    typography: &crate::theme::LumaTypography,
    style: RadixButtonStyle,
    checked: bool,
    state: InteractionState,
) -> CheckboxPalette {
    let layer = state.layer();
    let checked_action = palette.action(style);
    let outline = palette.outline;

    let indicator_background = match (checked, layer) {
        (_, InteractionLayer::Disabled) => palette.disabled_background,
        (true, InteractionLayer::Pressed) => checked_action.pressed_background,
        (true, InteractionLayer::Hovered) => checked_action.hover_background,
        (true, InteractionLayer::Default) => checked_action.background,
        (false, InteractionLayer::Pressed) => outline.pressed_background,
        (false, InteractionLayer::Hovered) => outline.hover_background,
        (false, InteractionLayer::Default) => outline.background,
    };

    let checkmark_color = if state.disabled {
        palette.disabled_foreground
    } else if checked {
        checked_action.foreground
    } else {
        palette.app_foreground
    };

    CheckboxPalette {
        control_background: None,
        control_border: None,
        indicator_background,
        indicator_border: if checked && !state.disabled {
            indicator_background
        } else {
            palette.border_default
        },
        checkmark_color,
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

pub(crate) fn checkbox_appearance_from_catalog(
    catalog: &CssTokenMap,
    metrics: &crate::theme::MetricTokens,
    typography: &crate::theme::LumaTypography,
    style: RadixButtonStyle,
    checked: bool,
    state: InteractionState,
) -> anyhow::Result<CheckboxPalette> {
    let layer = state.layer();

    let indicator_background = match (checked, layer) {
        (_, InteractionLayer::Disabled) => resolve_color(catalog, "muted")?,
        (true, _) => resolve_action_layer(catalog, style, layer)?,
        (false, _) => resolve_outline_layer(catalog, layer)?,
    };

    let indicator_border = if checked && !state.disabled {
        indicator_background
    } else {
        resolve_color(catalog, "border")?
    };

    Ok(CheckboxPalette {
        control_background: None,
        control_border: None,
        indicator_background,
        indicator_border,
        checkmark_color: checkmark_color(catalog, style, checked, state.disabled)?,
        label_color: resolve_label_color(catalog, state.disabled)?,
        adorner: focus_adorner(catalog, metrics, state.focused)?,
        label_typography: typography.text.label,
        label_font_family: typography.font.sans.family.clone().into(),
    })
}

fn checkmark_color(
    catalog: &CssTokenMap,
    style: RadixButtonStyle,
    checked: bool,
    disabled: bool,
) -> anyhow::Result<gpui::Hsla> {
    if disabled {
        return resolve_color(catalog, "muted-foreground");
    }
    if checked {
        return resolve_action_foreground(catalog, style);
    }
    resolve_color(catalog, "foreground")
}
