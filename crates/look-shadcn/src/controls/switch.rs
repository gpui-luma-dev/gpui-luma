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
use gpui_luma::theme::{InteractionLayer, InteractionState, ThemeMode};

use crate::appearance_context::AppearanceContext;
use crate::elevation::thumb_shadow;
use crate::focus::focus_adorner;
use crate::resolve::{resolve_action_layer, resolve_color, resolve_label_color};
use super::ShadcnButtonStyle;
use crate::catalog::CssTokenMap;
use crate::mode::ShadcnModeTokens;
use crate::palette::{ShadcnActionRole, ShadcnPalette};

pub(crate) fn switch_appearance(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    on: bool,
    state: InteractionState,
) -> SwitchPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, state);
    if mode.catalog.tokens.is_empty() {
        return switch_appearance_from_palette(&ctx, style, on);
    }

    switch_appearance_from_catalog(&ctx, style, on).unwrap_or_else(|err| panic!("switch properties: {err}"))
}

fn switch_appearance_from_palette(ctx: &AppearanceContext, style: ShadcnButtonStyle, on: bool) -> SwitchPalette {
    let state = ctx.state;
    let palette = ctx.palette();
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let thumb_shadow = thumb_shadow(ctx.theme_mode);
    let on_action = palette.action(style);

    let track_background = if state.disabled {
        palette.disabled_background
    } else if on {
        on_action.background
    } else {
        palette.input_background
    };

    let track_border = if on && !state.disabled {
        track_background
    } else {
        palette.border_default
    };

    let (thumb_background, thumb_border) = switch_thumb_surface_palette(palette, on_action, on, state.disabled);

    SwitchPalette {
        track_background,
        track_border,
        thumb_background,
        thumb_border,
        thumb_shadow,
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

pub(crate) fn switch_appearance_from_catalog(
    ctx: &AppearanceContext,
    style: ShadcnButtonStyle,
    on: bool,
) -> anyhow::Result<SwitchPalette> {
    let state = ctx.state;
    let catalog = ctx.catalog();
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let thumb_shadow = thumb_shadow(ctx.theme_mode);

    let track_background = if state.disabled {
        resolve_color(catalog, "muted")?
    } else if on {
        resolve_action_layer(catalog, style, InteractionLayer::Default, ctx.theme_mode)?
    } else {
        resolve_color(catalog, "input")?
    };

    let track_border = if on && !state.disabled {
        track_background
    } else {
        resolve_color(catalog, "border")?
    };

    let (thumb_background, thumb_border) = switch_thumb_colors(catalog, style, on, state.disabled)?;

    Ok(SwitchPalette {
        track_background,
        track_border,
        thumb_background,
        thumb_border,
        thumb_shadow,
        label_color: resolve_label_color(catalog, state.disabled)?,
        adorner: focus_adorner(catalog, metrics, state.focused)?,
        label_typography: typography.text.label,
        label_font_family: typography.font.sans.family.clone().into(),
    })
}

fn switch_thumb_colors(
    catalog: &CssTokenMap,
    style: ShadcnButtonStyle,
    on: bool,
    disabled: bool,
) -> anyhow::Result<(gpui::Hsla, gpui::Hsla)> {
    if disabled {
        let thumb = resolve_color(catalog, "muted-foreground")?;
        let border = resolve_color(catalog, "muted")?;
        return Ok((thumb, border));
    }

    if on {
        let thumb = crate::resolve::resolve_action_foreground(catalog, style)?;
        return Ok((thumb, thumb));
    }

    let thumb = resolve_color(catalog, "background")?;
    let border = resolve_color(catalog, "border")?;
    Ok((thumb, border))
}

fn switch_thumb_surface_palette(
    palette: &ShadcnPalette,
    on_action: ShadcnActionRole,
    on: bool,
    disabled: bool,
) -> (gpui::Hsla, gpui::Hsla) {
    if disabled {
        return (palette.disabled_foreground, palette.disabled_background);
    }

    if on {
        let thumb = on_action.foreground;
        return (thumb, thumb);
    }

    (palette.app_background, palette.border_default)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use gpui_luma::theme::{InteractionState, ThemeMode};

    use crate::appearance_context::AppearanceContext;
    use crate::controls::button::ShadcnButtonStyle;
    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::switch_appearance_from_catalog;
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
        let ctx = AppearanceContext::new(&mode, ThemeMode::Light, InteractionState::default());
        let appearance = switch_appearance_from_catalog(&ctx, ShadcnButtonStyle::Primary, false).expect("switch");

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
        let ctx = AppearanceContext::new(&mode, ThemeMode::Light, InteractionState::default());
        let appearance = switch_appearance_from_catalog(&ctx, ShadcnButtonStyle::Primary, true).expect("switch");

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
        let default = switch_appearance_from_catalog(
            &AppearanceContext::new(&mode, ThemeMode::Light, InteractionState::default()),
            ShadcnButtonStyle::Primary,
            false,
        )
        .expect("default");
        let hovered = switch_appearance_from_catalog(
            &AppearanceContext::new(
                &mode,
                ThemeMode::Light,
                InteractionState { hovered: true, ..InteractionState::default() },
            ),
            ShadcnButtonStyle::Primary,
            false,
        )
        .expect("hovered");
        let pressed = switch_appearance_from_catalog(
            &AppearanceContext::new(
                &mode,
                ThemeMode::Light,
                InteractionState { hovered: true, pressed: true, ..InteractionState::default() },
            ),
            ShadcnButtonStyle::Primary,
            false,
        )
        .expect("pressed");

        assert_eq!(default.track_background, hovered.track_background);
        assert_eq!(default.track_background, pressed.track_background);
        assert_eq!(default.thumb_background, hovered.thumb_background);
    }

    #[test]
    fn primary_and_secondary_share_off_appearance() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let ctx = AppearanceContext::new(&mode, ThemeMode::Light, InteractionState::default());
        let primary = switch_appearance_from_catalog(&ctx, ShadcnButtonStyle::Primary, false).expect("primary");
        let secondary = switch_appearance_from_catalog(&ctx, ShadcnButtonStyle::Secondary, false).expect("secondary");

        assert_eq!(primary.track_background, secondary.track_background);
        assert_eq!(primary.thumb_background, secondary.thumb_background);
    }
}
