//! Checkbox property mappings (tweakcn / shadcn):
//!
//! | State    | Indicator token | Checkmark token        |
//! |----------|-----------------|------------------------|
//! | Unchecked| outline surface | `foreground`           |
//! | Checked  | `{style}`       | `{style}-foreground`   |
//! | Disabled | `muted`         | `muted-foreground`     |

use gpui_luma::controls::checkbox::CheckboxPalette;
use gpui_luma::theme::{InteractionLayer, InteractionState, ThemeMode};

use crate::appearance_context::AppearanceContext;
use crate::focus::focus_adorner;
use crate::resolve::{
    resolve_action_foreground, resolve_action_layer, resolve_color, resolve_label_color, resolve_outline_layer,
};
use super::ShadcnButtonStyle;
use crate::catalog::CssTokenMap;
use crate::mode::ShadcnModeTokens;

pub(crate) fn checkbox_appearance(
    mode: &ShadcnModeTokens,
    style: ShadcnButtonStyle,
    checked: bool,
    state: InteractionState,
) -> CheckboxPalette {
    let ctx = AppearanceContext::new(mode, ThemeMode::Light, state);
    if mode.catalog.tokens.is_empty() {
        return checkbox_appearance_from_palette(&ctx, style, checked);
    }

    checkbox_appearance_from_catalog(&ctx, style, checked).unwrap_or_else(|err| panic!("checkbox properties: {err}"))
}

fn checkbox_appearance_from_palette(
    ctx: &AppearanceContext,
    style: ShadcnButtonStyle,
    checked: bool,
) -> CheckboxPalette {
    let state = ctx.state;
    let palette = ctx.palette();
    let metrics = ctx.metrics();
    let typography = ctx.typography();
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
        adorner: crate::focus::focus_adorner_from_palette(palette, metrics, state.focused),
        label_typography: typography.text.label,
        label_font_family: typography.font.sans.family.clone().into(),
    }
}

pub(crate) fn checkbox_appearance_from_catalog(
    ctx: &AppearanceContext,
    style: ShadcnButtonStyle,
    checked: bool,
) -> anyhow::Result<CheckboxPalette> {
    let state = ctx.state;
    let catalog = ctx.catalog();
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let layer = state.layer();

    let indicator_background = match (checked, layer) {
        (_, InteractionLayer::Disabled) => resolve_color(catalog, "muted")?,
        (true, _) => resolve_action_layer(catalog, style, layer, ctx.theme_mode)?,
        (false, _) => resolve_outline_layer(catalog, layer, ctx.theme_mode)?,
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
    style: ShadcnButtonStyle,
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
