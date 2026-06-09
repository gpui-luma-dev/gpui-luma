//! Switch property mappings (shadcn Switch):
//!
//! | State    | Track token | Track border     | Thumb token              |
//! |----------|-------------|------------------|--------------------------|
//! | Off      | `input`     | `border`         | `background` / `border`  |
//! | On       | `{style}`   | matches track bg | `{style}-foreground`     |
//! | Disabled | `muted`     | `border`         | `muted-foreground` / `muted` |
//!
//! Hover and pressed do not recolor the track or thumb (shadcn Switch has no hover
//! surface). Only `focused` adds a focus ring via the adorner.

use gpui_luma::controls::switch::SwitchPalette;
use gpui_luma::theme::{InteractionState, ThemeMode};

use crate::appearance_context::AppearanceContext;
use crate::elevation::thumb_shadow;
use crate::focus::focus_adorner;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::resolve::resolve_color;
use super::ShadcnButtonStyle;
use crate::mode::ShadcnModeTokens;
use crate::stylesheet::{StylesheetConfig, embedded_stylesheet, find_switch_color_rule, resolve_switch_color_rule};

#[derive(Clone, Debug)]
pub struct SwitchColorTable {
    pub track_background: ResolvedColor,
    pub thumb_background: ResolvedColor,
    pub thumb_border: ResolvedColor,
    pub label_color: ResolvedColor,
}

impl SwitchColorTable {
    pub fn fallback() -> Self {
        Self {
            track_background: ResolvedColor::transparent(),
            thumb_background: ResolvedColor::fallback_foreground(),
            thumb_border: ResolvedColor::fallback_foreground(),
            label_color: ResolvedColor::fallback_foreground(),
        }
    }
}

pub fn resolve_switch_colors(
    resolver: &LookResolver<'_>,
    style: ShadcnButtonStyle,
    on: bool,
    disabled: bool,
) -> anyhow::Result<SwitchColorTable> {
    resolve_switch_colors_with_stylesheet(resolver, embedded_stylesheet(), style, on, disabled)
}

pub fn resolve_switch_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
    on: bool,
    disabled: bool,
) -> anyhow::Result<SwitchColorTable> {
    let rule = find_switch_color_rule(stylesheet, on, disabled)
        .ok_or_else(|| anyhow::anyhow!("no matching switch color rule"))?;
    let colors = resolve_switch_color_rule(resolver, rule, style)?;
    Ok(SwitchColorTable {
        track_background: colors.track_background,
        thumb_background: colors.thumb_background,
        thumb_border: colors.thumb_border,
        label_color: colors.label_color,
    })
}

pub fn switch_appearance(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    on: bool,
    state: InteractionState,
) -> SwitchPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, state);
    let state = ctx.state;
    let catalog = ctx.catalog();
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let thumb_shadow = thumb_shadow(ctx.theme_mode);
    let resolver = LookResolver::new(catalog, ctx.theme_mode, "switch");
    let colors =
        resolve_switch_colors(&resolver, style, on, state.disabled).unwrap_or_else(|_| SwitchColorTable::fallback());

    let track_background = colors.track_background.hsla();
    let track_border = if on && !state.disabled {
        track_background
    } else {
        resolve_color(catalog, "border").unwrap_or_else(|err| panic!("switch properties: {err}"))
    };
    let thumb_background = colors.thumb_background.hsla();
    let thumb_border = if on && !state.disabled {
        thumb_background
    } else {
        colors.thumb_border.hsla()
    };
    let adorner =
        focus_adorner(catalog, metrics, state.focused).unwrap_or_else(|err| panic!("switch properties: {err}"));

    SwitchPalette {
        track_background,
        track_border,
        thumb_background,
        thumb_border,
        thumb_shadow,
        label_color: colors.label_color.hsla(),
        adorner,
        label_typography: typography.text.label,
        label_font_family: typography.font.sans.family.clone().into(),
    }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use gpui_luma::theme::{InteractionState, ThemeMode};

    use crate::controls::button::ShadcnButtonStyle;
    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::switch_appearance;

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("primary-foreground".into(), "oklch(1 0 0)".into()),
            ("secondary".into(), "oklch(0.6437 0.1019 187.3840)".into()),
            ("secondary-foreground".into(), "oklch(1 0 0)".into()),
            ("background".into(), "oklch(0.9735 0.0261 90.0953)".into()),
            ("foreground".into(), "oklch(0.1 0 0)".into()),
            ("muted".into(), "oklch(0.6979 0.0159 196.7940)".into()),
            ("muted-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.8 0.02 200)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
        ]))
    }

    #[test]
    fn off_switch_uses_input_track_and_background_thumb() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let appearance =
            switch_appearance(&mode, ThemeMode::Light, ShadcnButtonStyle::Primary, false, InteractionState::default());

        let input = catalog.color("input").expect("input");
        let border = catalog.color("border").expect("border");
        let background = catalog.color("background").expect("background");
        assert_eq!(appearance.track_background, input);
        assert_eq!(appearance.track_border, border);
        assert_eq!(appearance.thumb_background, background);
        assert_eq!(appearance.thumb_border, border);
    }

    #[test]
    fn on_switch_uses_style_track_and_card_thumb() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let appearance =
            switch_appearance(&mode, ThemeMode::Light, ShadcnButtonStyle::Primary, true, InteractionState::default());

        let primary = catalog.color("primary").expect("primary");
        let primary_foreground = catalog.color("primary-foreground").expect("primary-foreground");
        assert_eq!(appearance.track_background, primary);
        assert_eq!(appearance.track_border, primary);
        assert_eq!(appearance.thumb_background, primary_foreground);
        assert!(appearance.thumb_background.l > appearance.track_background.l);
    }

    #[test]
    fn astrovista_light_off_switch_uses_background_thumb_and_border_track() {
        let css = include_str!("../../../../apps/gallery/tweakcn/astrovista.css");
        let theme = crate::ShadcnLook::from_css_str(css).expect("astrovista css");
        let appearance = switch_appearance(
            theme.mode_tokens(),
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            false,
            InteractionState::default(),
        );

        let catalog = &theme.mode_tokens().catalog;
        let input = catalog.color("input").expect("input");
        let border = catalog.color("border").expect("border");
        let background = catalog.color("background").expect("background");
        let card = catalog.color("card").expect("card");
        assert_eq!(appearance.track_background, input);
        assert_eq!(appearance.track_border, border);
        assert_eq!(appearance.thumb_background, background);
        assert_eq!(appearance.thumb_border, border);
        assert_ne!(appearance.thumb_background, card);
        assert!(appearance.thumb_background.l < appearance.track_background.l);
    }

    #[test]
    fn astrovista_dark_off_switch_uses_background_thumb() {
        let css = include_str!("../../../../apps/gallery/tweakcn/astrovista.css");
        let theme = crate::ShadcnLook::from_css_str(css).expect("astrovista css");
        theme.set_mode(ThemeMode::Dark);
        let appearance = switch_appearance(
            theme.mode_tokens(),
            ThemeMode::Dark,
            ShadcnButtonStyle::Primary,
            false,
            InteractionState::default(),
        );

        let catalog = &theme.mode_tokens().catalog;
        let input = catalog.color("input").expect("input");
        let background = catalog.color("background").expect("background");
        assert_eq!(appearance.track_background, input);
        assert_eq!(appearance.thumb_background, background);
    }

    #[test]
    fn hover_and_pressed_match_default_track_for_off_switch() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let default =
            switch_appearance(&mode, ThemeMode::Light, ShadcnButtonStyle::Primary, false, InteractionState::default());
        let hovered = switch_appearance(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            false,
            InteractionState { hovered: true, ..InteractionState::default() },
        );
        let pressed = switch_appearance(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            false,
            InteractionState { hovered: true, pressed: true, ..InteractionState::default() },
        );

        assert_eq!(default.track_background, hovered.track_background);
        assert_eq!(default.track_background, pressed.track_background);
        assert_eq!(default.thumb_background, hovered.thumb_background);
    }

    #[test]
    fn primary_and_secondary_share_off_appearance() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let primary =
            switch_appearance(&mode, ThemeMode::Light, ShadcnButtonStyle::Primary, false, InteractionState::default());
        let secondary = switch_appearance(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Secondary,
            false,
            InteractionState::default(),
        );

        assert_eq!(primary.track_background, secondary.track_background);
        assert_eq!(primary.thumb_background, secondary.thumb_background);
    }
}
