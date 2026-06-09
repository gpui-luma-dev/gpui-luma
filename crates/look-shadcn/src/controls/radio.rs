//! Radio button property mappings (tweakcn / shadcn):
//!
//! | State     | Ring token | Dot token              |
//! |-----------|------------|------------------------|
//! | Unselected| outline    | `foreground`           |
//! | Selected  | `{style}`  | `{style}-foreground`   |
//! | Disabled  | `muted`    | `muted-foreground`     |

use gpui_luma::controls::radio_button::RadioButtonPalette;
use gpui_luma::theme::{InteractionLayer, InteractionState, ThemeMode};

use crate::appearance_context::AppearanceContext;
use crate::focus::focus_adorner;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::resolve::resolve_color;
use super::ShadcnButtonStyle;
use crate::mode::ShadcnModeTokens;

use gpui_luma_look_shadcn_macros::declare_look_table;

#[derive(Clone, Debug)]
pub struct RadioColorTable {
    pub indicator_background: ResolvedColor,
    pub selection_ring: ResolvedColor,
    pub dot_color: ResolvedColor,
    pub label_color: ResolvedColor,
}

impl RadioColorTable {
    pub fn fallback() -> Self {
        Self {
            indicator_background: ResolvedColor::transparent(),
            selection_ring: ResolvedColor::fallback_foreground(),
            dot_color: ResolvedColor::fallback_foreground(),
            label_color: ResolvedColor::fallback_foreground(),
        }
    }
}

declare_look_table! {
    name: resolve_radio_colors,
    inputs: {
        style: ShadcnButtonStyle,
        selected: bool,
        layer: InteractionLayer,
    },
    output: RadioColorTable { indicator_background, selection_ring, dot_color, label_color },
    matrix: [
        [_] | [_] | [InteractionLayer::Disabled] => "muted" | "muted-foreground" | "muted-foreground" | "muted-foreground",

        [_] | [false] | [InteractionLayer::Default] => "@outline_layer" | "border" | "foreground" | "foreground",
        [_] | [false] | [InteractionLayer::Hovered] => "@outline_layer" | "border" | "foreground" | "foreground",
        [_] | [false] | [InteractionLayer::Pressed] => "@outline_layer" | "border" | "foreground" | "foreground",

        [_] | [true] | [InteractionLayer::Default] => "@outline_layer" | "@action_layer" | "@action_foreground" | "foreground",
        [_] | [true] | [InteractionLayer::Hovered] => "@outline_layer" | "@action_layer" | "@action_foreground" | "foreground",
        [_] | [true] | [InteractionLayer::Pressed] => "@outline_layer" | "@action_layer" | "@action_foreground" | "foreground",
    ]
}

pub fn radio_button_appearance(
    mode: &ShadcnModeTokens,
    style: ShadcnButtonStyle,
    selected: bool,
    state: InteractionState,
) -> RadioButtonPalette {
    let ctx = AppearanceContext::new(mode, ThemeMode::Light, state);
    if mode.catalog.tokens.is_empty() {
        return radio_button_appearance_from_palette(&ctx, style, selected);
    }

    radio_button_appearance_from_catalog(&ctx, style, selected).unwrap_or_else(|err| panic!("radio properties: {err}"))
}

pub fn radio_button_appearance_from_palette(
    ctx: &AppearanceContext,
    style: ShadcnButtonStyle,
    selected: bool,
) -> RadioButtonPalette {
    let state = ctx.state;
    let palette = ctx.palette();
    let metrics = ctx.metrics();
    let typography = ctx.typography();
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
        adorner: crate::focus::focus_adorner_from_palette(palette, metrics, state.focused),
        label_typography: typography.text.label,
        label_font_family: typography.font.sans.family.clone().into(),
    }
}

pub fn radio_button_appearance_from_catalog(
    ctx: &AppearanceContext,
    style: ShadcnButtonStyle,
    selected: bool,
) -> anyhow::Result<RadioButtonPalette> {
    let state = ctx.state;
    let catalog = ctx.catalog();
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let layer = state.layer();
    let resolver = LookResolver::new(catalog, ctx.theme_mode, "radio");
    let colors =
        resolve_radio_colors(&resolver, style, selected, layer).unwrap_or_else(|_| RadioColorTable::fallback());

    let indicator_border = if selected && !state.disabled {
        colors.selection_ring.hsla()
    } else {
        resolve_color(catalog, "border")?
    };

    Ok(RadioButtonPalette {
        control_background: None,
        control_border: None,
        indicator_background: colors.indicator_background.hsla(),
        indicator_border,
        dot_color: colors.dot_color.hsla(),
        label_color: colors.label_color.hsla(),
        adorner: focus_adorner(catalog, metrics, state.focused)?,
        label_typography: typography.text.label,
        label_font_family: typography.font.sans.family.clone().into(),
    })
}
