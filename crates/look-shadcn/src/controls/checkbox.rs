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
use crate::stylesheet::{StylesheetConfig, embedded_stylesheet, find_checkbox_color_rule, resolve_checkbox_color_rule};

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

pub fn resolve_checkbox_colors(
    resolver: &LookResolver<'_>,
    style: ShadcnButtonStyle,
    checked: bool,
    layer: InteractionLayer,
) -> anyhow::Result<CheckboxColorTable> {
    resolve_checkbox_colors_with_stylesheet(resolver, embedded_stylesheet(), style, checked, layer)
}

pub fn resolve_checkbox_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
    checked: bool,
    layer: InteractionLayer,
) -> anyhow::Result<CheckboxColorTable> {
    let rule = find_checkbox_color_rule(stylesheet, checked, layer)
        .ok_or_else(|| anyhow::anyhow!("no matching checkbox color rule"))?;
    let colors = resolve_checkbox_color_rule(resolver, rule, style, layer)?;
    Ok(CheckboxColorTable {
        indicator_background: colors.indicator_background,
        checkmark_color: colors.checkmark_color,
        label_color: colors.label_color,
    })
}

pub fn checkbox_appearance(
    mode: &ShadcnModeTokens,
    style: ShadcnButtonStyle,
    checked: bool,
    state: InteractionState,
) -> CheckboxPalette {
    let ctx = AppearanceContext::new(mode, ThemeMode::Light, state);
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
        resolve_color(catalog, "border").unwrap_or_else(|err| panic!("checkbox properties: {err}"))
    };

    CheckboxPalette {
        control_background: None,
        control_border: None,
        indicator_background: colors.indicator_background.hsla(),
        indicator_border,
        checkmark_color: colors.checkmark_color.hsla(),
        label_color: colors.label_color.hsla(),
        adorner: focus_adorner(catalog, metrics, state.focused)
            .unwrap_or_else(|err| panic!("checkbox properties: {err}")),
        label_typography: typography.text.label,
        label_font_family: typography.font.sans.family.clone().into(),
    }
}
