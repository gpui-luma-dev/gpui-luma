//! Radio button property mappings (tweakcn / shadcn):
//!
//! | State     | Ring token | Dot token              |
//! |-----------|------------|------------------------|
//! | Unselected| outline    | `foreground`           |
//! | Selected  | `{style}`  | `{style}-foreground`   |
//! | Disabled  | `muted`    | `muted-foreground`     |

use gpui_luma::controls::radio_button::RadioButtonPalette;
use gpui_luma::theme::{InteractionLayer, InteractionState, ThemeMode};

use crate::look_context::LookContext;
use crate::focus::focus_adorner;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::resolve::resolve_color;
use super::ShadcnButtonStyle;
use crate::mode::ShadcnModeTokens;
use crate::stylesheet::{StylesheetConfig, embedded_stylesheet, find_radio_color_rule, resolve_radio_color_rule};

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

pub fn resolve_radio_colors(
    resolver: &LookResolver<'_>,
    style: ShadcnButtonStyle,
    selected: bool,
    layer: InteractionLayer,
) -> anyhow::Result<RadioColorTable> {
    resolve_radio_colors_with_stylesheet(resolver, embedded_stylesheet(), style, selected, layer)
}

pub fn resolve_radio_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
    selected: bool,
    layer: InteractionLayer,
) -> anyhow::Result<RadioColorTable> {
    let rule = find_radio_color_rule(stylesheet, selected, layer)
        .ok_or_else(|| anyhow::anyhow!("no matching radio color rule"))?;
    let colors = resolve_radio_color_rule(resolver, rule, style, layer)?;
    Ok(RadioColorTable {
        indicator_background: colors.indicator_background,
        selection_ring: colors.selection_ring,
        dot_color: colors.dot_color,
        label_color: colors.label_color,
    })
}

pub fn radio_button_look(
    mode: &ShadcnModeTokens,
    style: ShadcnButtonStyle,
    selected: bool,
    state: InteractionState,
) -> RadioButtonPalette {
    let ctx = LookContext::new(mode, ThemeMode::Light, state);
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
        resolve_color(catalog, "border").unwrap_or_else(|err| panic!("radio properties: {err}"))
    };
    let adorner =
        focus_adorner(catalog, metrics, state.focused).unwrap_or_else(|err| panic!("radio properties: {err}"));

    RadioButtonPalette {
        control_background: None,
        control_border: None,
        indicator_background: colors.indicator_background.hsla(),
        indicator_border,
        dot_color: colors.dot_color.hsla(),
        label_color: colors.label_color.hsla(),
        adorner,
        label_typography: typography.text.label,
        label_font_family: typography.font.sans.family.clone().into(),
    }
}
