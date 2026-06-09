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
use crate::provenance::{LookResolver, ResolvedColor};
use crate::resolve::resolve_color;
use super::ShadcnButtonStyle;
use crate::mode::ShadcnModeTokens;

use gpui_luma_look_shadcn_macros::declare_look_table;

#[derive(Clone, Debug)]
pub struct CheckboxColorTable {
    pub indicator_background: ResolvedColor,
    pub checkmark_color: ResolvedColor,
    pub label_color: ResolvedColor,
}

impl CheckboxColorTable {
    pub fn fallback() -> Self {
        Self {
            indicator_background: ResolvedColor::transparent(),
            checkmark_color: ResolvedColor::fallback_foreground(),
            label_color: ResolvedColor::fallback_foreground(),
        }
    }
}

declare_look_table! {
    name: resolve_checkbox_colors,
    inputs: {
        style: ShadcnButtonStyle,
        checked: bool,
        layer: InteractionLayer,
    },
    output: CheckboxColorTable { indicator_background, checkmark_color, label_color },
    matrix: [
        [_] | [_] | [InteractionLayer::Disabled] => "muted" | "muted-foreground" | "muted-foreground",

        [_] | [false] | [InteractionLayer::Default] => "@outline_layer" | "foreground" | "foreground",
        [_] | [false] | [InteractionLayer::Hovered] => "@outline_layer" | "foreground" | "foreground",
        [_] | [false] | [InteractionLayer::Pressed] => "@outline_layer" | "foreground" | "foreground",

        [_] | [true] | [InteractionLayer::Default] => "@action_layer" | "@action_foreground" | "foreground",
        [_] | [true] | [InteractionLayer::Hovered] => "@action_layer" | "@action_foreground" | "foreground",
        [_] | [true] | [InteractionLayer::Pressed] => "@action_layer" | "@action_foreground" | "foreground",
    ]
}

pub fn checkbox_appearance(
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

pub fn checkbox_appearance_from_palette(
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

pub fn checkbox_appearance_from_catalog(
    ctx: &AppearanceContext,
    style: ShadcnButtonStyle,
    checked: bool,
) -> anyhow::Result<CheckboxPalette> {
    let state = ctx.state;
    let catalog = ctx.catalog();
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let layer = state.layer();
    let resolver = LookResolver::new(catalog, ctx.theme_mode, "checkbox");
    let colors =
        resolve_checkbox_colors(&resolver, style, checked, layer).unwrap_or_else(|_| CheckboxColorTable::fallback());

    let indicator_border = if checked && !state.disabled {
        colors.indicator_background.hsla()
    } else {
        resolve_color(catalog, "border")?
    };

    Ok(CheckboxPalette {
        control_background: None,
        control_border: None,
        indicator_background: colors.indicator_background.hsla(),
        indicator_border,
        checkmark_color: colors.checkmark_color.hsla(),
        label_color: colors.label_color.hsla(),
        adorner: focus_adorner(catalog, metrics, state.focused)?,
        label_typography: typography.text.label,
        label_font_family: typography.font.sans.family.clone().into(),
    })
}
