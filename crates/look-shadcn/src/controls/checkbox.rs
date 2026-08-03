//! Checkbox property mappings (tweakcn / shadcn):
//!
//! | State    | Indicator token | Checkmark token        |
//! |----------|-----------------|------------------------|
//! | Unchecked| outline surface | `foreground`           |
//! | Checked  | `{style}`       | `{style}-foreground`   |
//! | Disabled | `muted`         | `muted-foreground`     |

use gpui_luma::controls::checkbox::CheckboxPalette;
use gpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, ThemeMode};

use super::apply_button_metrics_typography;

use crate::look_context::LookContext;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::resolve::resolve_color;
use super::ShadcnButtonStyle;
use super::choice_indicator::choice_indicator_color_layer;
use crate::mode::ShadcnModeTokens;
use crate::shadow::parse_shadow_token;
use crate::stylesheet::{
    StylesheetConfig, embedded_stylesheet, find_checkbox_color_rule, resolve_checkbox_color_rule,
    resolve_layered_elevation_shadow,
};

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

pub fn checkbox_look(
    mode: &ShadcnModeTokens,
    style: ShadcnButtonStyle,
    checked: bool,
    state: InteractionState,
    size: ControlSize,
) -> CheckboxPalette {
    let content_only = style == ShadcnButtonStyle::ContentOnly;
    let indicator_style = if content_only {
        ShadcnButtonStyle::Primary
    } else {
        style
    };
    let ctx = LookContext::new(mode, ThemeMode::Light, state);
    let state = ctx.state;
    let catalog = ctx.catalog();
    let typography = ctx.typography();
    let layer = choice_indicator_color_layer(state);
    let resolver = LookResolver::new(catalog, ctx.theme_mode, "checkbox");
    let colors = resolve_checkbox_colors(&resolver, indicator_style, checked, layer)
        .unwrap_or_else(|_| CheckboxColorTable::fallback());

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
        label_typography: {
            let mut label_typography = typography.text.label;
            apply_button_metrics_typography(&mut label_typography, mode, size);
            label_typography
        },
        label_font_family: typography.font.sans.family.clone().into(),
        indicator_shadow: if content_only {
            None
        } else {
            checkbox_elevation_shadow(catalog, embedded_stylesheet(), layer)
        },
    }
}

fn checkbox_elevation_shadow(
    catalog: &crate::catalog::CssTokenMap,
    stylesheet: &StylesheetConfig,
    layer: InteractionLayer,
) -> Option<Vec<gpui::BoxShadow>> {
    let token = resolve_layered_elevation_shadow(&stylesheet.checkbox.elevation_rules, layer)?;
    let shadows = parse_shadow_token(catalog, &token).ok()?;
    if shadows.is_empty() { None } else { Some(shadows) }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use gpui_luma::theme::ThemeMode;

    fn retro_arcade_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "hsl(330.9554 64.0816% 51.9608%)".into()),
            ("primary-foreground".into(), "hsl(0 0% 100%)".into()),
            ("secondary".into(), "hsl(175.4622 58.6207% 39.8039%)".into()),
            ("secondary-foreground".into(), "hsl(0 0% 100%)".into()),
            ("background".into(), "hsl(43.8462 86.6667% 94.1176%)".into()),
            ("foreground".into(), "hsl(192.2034 80.8219% 14.3137%)".into()),
            ("muted".into(), "hsl(180 6.9307% 60.3922%)".into()),
            ("muted-foreground".into(), "hsl(192.2034 80.8219% 14.3137%)".into()),
            ("accent".into(), "hsl(17.5691 80.4444% 44.1176%)".into()),
            ("accent-foreground".into(), "hsl(0 0% 100%)".into()),
            ("destructive".into(), "hsl(1.0405 71.1934% 52.3529%)".into()),
            ("destructive-foreground".into(), "hsl(0 0% 100%)".into()),
            ("border".into(), "hsl(186.3158 8.2969% 55.0980%)".into()),
            ("input".into(), "hsl(186.3158 8.2969% 55.0980%)".into()),
            ("ring".into(), "hsl(330.9554 64.0816% 51.9608%)".into()),
            ("card".into(), "hsl(45.6000 42.3729% 88.4314%)".into()),
            ("radius".into(), "0.25rem".into()),
            ("spacing".into(), "0.25rem".into()),
            ("font-sans".into(), "ui-sans-serif, system-ui, 'Outfit', sans-serif".into()),
            ("shadow-xs".into(), "0 1px 3px 0px hsl(0 0% 0% / 0.05)".into()),
            ("shadow-sm".into(), "0 1px 3px 0px hsl(0 0% 0% / 0.10), 0 1px 2px -1px hsl(0 0% 0% / 0.10)".into()),
        ]))
    }

    #[test]
    fn checkbox_look_resolves_stylesheet_shadow() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let look =
            checkbox_look(&mode, ShadcnButtonStyle::Primary, false, InteractionState::default(), ControlSize::Md);

        assert!(look.indicator_shadow.as_ref().is_some_and(|shadows| !shadows.is_empty()));
    }

    #[test]
    fn disabled_checkbox_look_has_no_shadow() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let look = checkbox_look(
            &mode,
            ShadcnButtonStyle::Primary,
            false,
            InteractionState { disabled: true, ..InteractionState::default() },
            ControlSize::Md,
        );

        assert!(look.indicator_shadow.is_none());
    }

    #[test]
    fn checkbox_hover_does_not_recolor_indicator_or_label() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let default =
            checkbox_look(&mode, ShadcnButtonStyle::Primary, false, InteractionState::default(), ControlSize::Md);
        let hovered = checkbox_look(
            &mode,
            ShadcnButtonStyle::Primary,
            false,
            InteractionState { hovered: true, ..InteractionState::default() },
            ControlSize::Md,
        );

        assert_eq!(default.indicator_background, hovered.indicator_background);
        assert_eq!(default.checkmark_color, hovered.checkmark_color);
        assert_eq!(default.label_color, hovered.label_color);
    }

    #[test]
    fn content_only_checkbox_keeps_primary_indicator_without_shadow() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let primary =
            checkbox_look(&mode, ShadcnButtonStyle::Primary, true, InteractionState::default(), ControlSize::Md);
        let content_only = checkbox_look(
            &mode,
            ShadcnButtonStyle::ContentOnly,
            true,
            InteractionState { focused: true, ..InteractionState::default() },
            ControlSize::Md,
        );

        assert_eq!(content_only.indicator_background, primary.indicator_background);
        assert_eq!(content_only.indicator_border, primary.indicator_border);
        assert_eq!(content_only.checkmark_color, primary.checkmark_color);
        assert!(content_only.indicator_shadow.is_none());
    }
}
