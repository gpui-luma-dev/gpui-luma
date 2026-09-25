//! Radio button property mappings (tweakcn / shadcn):
//!
//! Indicator variants (per-style default in `style.toml` `[radio.indicator_defaults]`):
//!
//! | Variant | Selected look |
//! |---------|----------------|
//! | `filled` | `@action_layer` fill + `@action_foreground` dot |
//! | `ring`   | `@outline_layer` fill + `@action_layer` ring + `@action_foreground` dot |
//! | `dot`    | `@outline_layer` fill + `border` ring + `@action_layer` dot |

use luma::controls::radio_button::RadioButtonPalette;
use luma::theme::{ControlSize, InteractionLayer, InteractionState, ThemeMode};

use super::apply_button_metrics_typography;

use crate::look_context::LookContext;
use crate::provenance::{LookResolver, ResolvedColor};
use super::ShadcnButtonStyle;
use super::choice_indicator::choice_indicator_color_layer;
use crate::mode::ShadcnModeTokens;
use crate::shadow::parse_shadow_token;
use crate::stylesheet::{
    StylesheetConfig, embedded_stylesheet, find_radio_color_rule, resolve_layered_elevation_shadow,
    resolve_radio_color_rule,
};

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
    let rule = find_radio_color_rule(stylesheet, style, selected, layer)
        .ok_or_else(|| anyhow::anyhow!("no matching radio color rule"))?;
    let colors = resolve_radio_color_rule(resolver, rule, style, layer)?;
    Ok(RadioColorTable {
        indicator_background: colors.indicator_background,
        selection_ring: colors.selection_ring,
        dot_color: colors.dot_color,
        label_color: colors.label_color,
    })
}

/// Effective colors after style normalization, interaction policy, focus, and fallbacks.
#[derive(Clone, Debug)]
pub struct RadioResolvedColors {
    pub indicator_background: ResolvedColor,
    pub indicator_border: ResolvedColor,
    pub dot_color: ResolvedColor,
    pub label_color: ResolvedColor,
}

/// Resolves the same final colors used by rendering and inspection.
pub fn resolve_radio_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    selected: bool,
    state: InteractionState,
) -> RadioResolvedColors {
    let indicator_style = if style == ShadcnButtonStyle::ContentOnly {
        ShadcnButtonStyle::Primary
    } else {
        style
    };
    let resolver = LookResolver::new(&mode.catalog, theme_mode, "radio");
    let colors = resolve_radio_colors(&resolver, indicator_style, selected, choice_indicator_color_layer(state))
        .unwrap_or_else(|_| RadioColorTable::fallback());
    let indicator_border =
        super::choice_indicator::resolve_indicator_border(&resolver, style, selected, state, &colors.selection_ring);
    RadioResolvedColors {
        indicator_background: colors.indicator_background,
        indicator_border,
        dot_color: colors.dot_color,
        label_color: colors.label_color,
    }
}

pub fn radio_button_look(
    mode: &ShadcnModeTokens,
    style: ShadcnButtonStyle,
    selected: bool,
    state: InteractionState,
    size: ControlSize,
) -> RadioButtonPalette {
    let content_only = style == ShadcnButtonStyle::ContentOnly;
    let ctx = LookContext::new(mode, ThemeMode::Light, state);
    let state = ctx.state;
    let catalog = ctx.catalog();
    let typography = ctx.typography();
    let layer = choice_indicator_color_layer(state);
    let colors = resolve_radio_palette(mode, ctx.theme_mode, style, selected, state);

    RadioButtonPalette {
        control_background: None,
        control_border: None,
        indicator_background: colors.indicator_background.hsla(),
        indicator_border: colors.indicator_border.hsla(),
        dot_color: colors.dot_color.hsla(),
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
            radio_elevation_shadow(catalog, embedded_stylesheet(), layer)
        },
    }
}

fn radio_elevation_shadow(
    catalog: &crate::catalog::CssTokenMap,
    stylesheet: &StylesheetConfig,
    layer: InteractionLayer,
) -> Option<Vec<gpui::BoxShadow>> {
    let token = resolve_layered_elevation_shadow(&stylesheet.radio.elevation_rules, layer)?;
    let shadows = parse_shadow_token(catalog, &token).ok()?;
    if shadows.is_empty() { None } else { Some(shadows) }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::resolve::resolve_color;
    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use luma::theme::ThemeMode;

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
    fn radio_look_resolves_stylesheet_shadow() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let look =
            radio_button_look(&mode, ShadcnButtonStyle::Primary, false, InteractionState::default(), ControlSize::Md);

        assert!(look.indicator_shadow.as_ref().is_some_and(|shadows| !shadows.is_empty()));
    }

    #[test]
    fn primary_selected_filled_uses_action_fill_and_foreground_dot() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let look =
            radio_button_look(&mode, ShadcnButtonStyle::Primary, true, InteractionState::default(), ControlSize::Md);
        let primary = resolve_color(&retro_arcade_catalog(), "primary").expect("primary");
        let primary_fg = resolve_color(&retro_arcade_catalog(), "primary-foreground").expect("primary-foreground");

        assert_eq!(look.indicator_background, primary);
        assert_eq!(look.indicator_border, primary);
        assert_eq!(look.dot_color, primary_fg);
    }

    #[test]
    fn secondary_selected_ring_uses_outline_fill_and_action_ring() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let catalog = retro_arcade_catalog();
        let look =
            radio_button_look(&mode, ShadcnButtonStyle::Secondary, true, InteractionState::default(), ControlSize::Md);
        let secondary = resolve_color(&catalog, "secondary").expect("secondary");
        let secondary_fg = resolve_color(&catalog, "secondary-foreground").expect("secondary-fg");
        let card = resolve_color(&catalog, "card").expect("card");

        assert_eq!(look.indicator_background, card);
        assert_eq!(look.indicator_border, secondary);
        assert_eq!(look.dot_color, secondary_fg);
    }

    #[test]
    fn disabled_radio_look_has_no_shadow() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let look = radio_button_look(
            &mode,
            ShadcnButtonStyle::Primary,
            false,
            InteractionState { disabled: true, ..InteractionState::default() },
            ControlSize::Md,
        );

        assert!(look.indicator_shadow.is_none());
    }

    #[test]
    fn radio_hover_does_not_recolor_indicator_or_label() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let default =
            radio_button_look(&mode, ShadcnButtonStyle::Primary, false, InteractionState::default(), ControlSize::Md);
        let hovered = radio_button_look(
            &mode,
            ShadcnButtonStyle::Primary,
            false,
            InteractionState { hovered: true, ..InteractionState::default() },
            ControlSize::Md,
        );

        assert_eq!(default.indicator_background, hovered.indicator_background);
        assert_eq!(default.dot_color, hovered.dot_color);
        assert_eq!(default.label_color, hovered.label_color);
    }

    #[test]
    fn content_only_radio_keeps_primary_indicator_without_shadow() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let primary =
            radio_button_look(&mode, ShadcnButtonStyle::Primary, true, InteractionState::default(), ControlSize::Md);
        let content_only = radio_button_look(
            &mode,
            ShadcnButtonStyle::ContentOnly,
            true,
            InteractionState { focused: true, ..InteractionState::default() },
            ControlSize::Md,
        );

        assert_eq!(content_only.indicator_background, primary.indicator_background);
        assert_eq!(content_only.indicator_border, primary.indicator_border);
        assert_eq!(content_only.dot_color, primary.dot_color);
        assert!(content_only.indicator_shadow.is_none());
    }

    fn mode_without_border_and_ring() -> ShadcnModeTokens {
        let mut mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        mode.catalog.tokens.remove("ring");
        mode.catalog.tokens.remove("border");
        mode
    }

    #[test]
    fn incomplete_catalog_does_not_panic_on_focus_or_border() {
        let mode = mode_without_border_and_ring();
        let fallback = crate::provenance::ResolvedColor::fallback_foreground().hsla();
        let focused = radio_button_look(
            &mode,
            ShadcnButtonStyle::Primary,
            false,
            InteractionState { focused: true, ..InteractionState::default() },
            ControlSize::Md,
        );
        let unfocused =
            radio_button_look(&mode, ShadcnButtonStyle::Primary, false, InteractionState::default(), ControlSize::Md);

        assert_eq!(focused.indicator_border, fallback);
        assert_eq!(unfocused.indicator_border, fallback);
    }
}
