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

use crate::controls::switch::SwitchAppearance;
use crate::theme::{ControlSize, InteractionLayer, InteractionState, LumaTheme, ThemeMode};

use super::focus::focus_adorner;
use super::resolve::{resolve_action_layer, resolve_color, resolve_label_color};
use super::RadixButtonStyle;
use super::catalog::CssTokenMap;
use super::mode::RadixModeTokens;
use super::palette::{RadixActionRole, RadixPalette};

pub(crate) fn switch_appearance(
    mode: &RadixModeTokens,
    theme_mode: ThemeMode,
    style: RadixButtonStyle,
    on: bool,
    state: InteractionState,
) -> SwitchAppearance {
    if mode.catalog.tokens.is_empty() {
        return switch_appearance_from_palette(
            &mode.palette,
            &mode.metrics,
            &mode.typography,
            theme_mode,
            style,
            on,
            state,
        );
    }

    switch_appearance_from_catalog(&mode.catalog, &mode.metrics, &mode.typography, theme_mode, style, on, state)
        .unwrap_or_else(|err| panic!("switch properties: {err}"))
}

fn switch_appearance_from_palette(
    palette: &RadixPalette,
    metrics: &crate::theme::MetricTokens,
    typography: &crate::theme::LumaTypography,
    theme_mode: ThemeMode,
    style: RadixButtonStyle,
    on: bool,
    state: InteractionState,
) -> SwitchAppearance {
    let size = ControlSize::Md;
    let thumb_shadow = {
        let native = LumaTheme::native();
        native.mode(theme_mode).elevation.thumb.to_box_shadows()
    };
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

    SwitchAppearance {
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
        adorner: super::focus::focus_adorner_from_palette(palette, metrics, state.focused),
        label_typography: typography.text.label,
        label_font_family: typography.font.sans.family.clone().into(),
        width: 42.0,
        height: 22.0,
        thumb_size: metrics.control_height(size) * 0.5,
        padding: 2.0,
        gap: metrics.gap(size),
        radius: metrics.radius.pill,
    }
}

pub(crate) fn switch_appearance_from_catalog(
    catalog: &CssTokenMap,
    metrics: &crate::theme::MetricTokens,
    typography: &crate::theme::LumaTypography,
    theme_mode: ThemeMode,
    style: RadixButtonStyle,
    on: bool,
    state: InteractionState,
) -> anyhow::Result<SwitchAppearance> {
    let size = ControlSize::Md;
    let thumb_shadow = {
        let native = LumaTheme::native();
        native.mode(theme_mode).elevation.thumb.to_box_shadows()
    };

    let track_background = if state.disabled {
        resolve_color(catalog, "muted")?
    } else if on {
        resolve_action_layer(catalog, style, InteractionLayer::Default)?
    } else {
        resolve_color(catalog, "input")?
    };

    let track_border = if on && !state.disabled {
        track_background
    } else {
        resolve_color(catalog, "border")?
    };

    let (thumb_background, thumb_border) = switch_thumb_colors(catalog, style, on, state.disabled)?;

    Ok(SwitchAppearance {
        track_background,
        track_border,
        thumb_background,
        thumb_border,
        thumb_shadow,
        label_color: resolve_label_color(catalog, state.disabled)?,
        adorner: focus_adorner(catalog, metrics, state.focused)?,
        label_typography: typography.text.label,
        label_font_family: typography.font.sans.family.clone().into(),
        width: 42.0,
        height: 22.0,
        thumb_size: metrics.control_height(size) * 0.5,
        padding: 2.0,
        gap: metrics.gap(size),
        radius: metrics.radius.pill,
    })
}

fn switch_thumb_colors(
    catalog: &CssTokenMap,
    style: RadixButtonStyle,
    on: bool,
    disabled: bool,
) -> anyhow::Result<(gpui::Hsla, gpui::Hsla)> {
    if disabled {
        let thumb = resolve_color(catalog, "muted-foreground")?;
        let border = resolve_color(catalog, "muted")?;
        return Ok((thumb, border));
    }

    if on {
        let thumb = super::resolve::resolve_action_foreground(catalog, style)?;
        return Ok((thumb, thumb));
    }

    let thumb = resolve_color(catalog, "background")?;
    let border = resolve_color(catalog, "border")?;
    Ok((thumb, border))
}

fn switch_thumb_surface_palette(
    palette: &RadixPalette,
    on_action: RadixActionRole,
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

    use crate::theme::{InteractionState, ThemeMode};

    use super::super::RadixButtonStyle;
    use super::super::catalog::CssTokenMap;
    use super::super::mode::RadixModeTokens;
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
        let mode = RadixModeTokens::from_catalog(catalog.clone()).expect("catalog");
        let appearance = switch_appearance_from_catalog(
            &catalog,
            &mode.metrics,
            &mode.typography,
            ThemeMode::Light,
            RadixButtonStyle::Primary,
            false,
            InteractionState::default(),
        )
        .expect("switch");

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
        let mode = RadixModeTokens::from_catalog(catalog.clone()).expect("catalog");
        let appearance = switch_appearance_from_catalog(
            &catalog,
            &mode.metrics,
            &mode.typography,
            ThemeMode::Light,
            RadixButtonStyle::Primary,
            true,
            InteractionState::default(),
        )
        .expect("switch");

        let primary = catalog.color("primary").expect("primary");
        let primary_foreground = catalog.color("primary-foreground").expect("primary-foreground");
        assert_eq!(appearance.track_background, primary);
        assert_eq!(appearance.track_border, primary);
        assert_eq!(appearance.thumb_background, primary_foreground);
        assert!(appearance.thumb_background.l > appearance.track_background.l);
    }

    #[test]
    fn astrovista_light_off_switch_uses_background_thumb_and_border_track() {
        let css = include_str!("../../../../../apps/gallery/tweakcn/astrovista.css");
        let theme = crate::theme::RadixTheme::from_css_str(css).expect("astrovista css");
        let appearance = switch_appearance(
            theme.mode_tokens(),
            ThemeMode::Light,
            RadixButtonStyle::Primary,
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
        let css = include_str!("../../../../../apps/gallery/tweakcn/astrovista.css");
        let theme = crate::theme::RadixTheme::from_css_str(css).expect("astrovista css");
        theme.set_mode(ThemeMode::Dark);
        let appearance = switch_appearance(
            theme.mode_tokens(),
            ThemeMode::Dark,
            RadixButtonStyle::Primary,
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
        let mode = RadixModeTokens::from_catalog(catalog.clone()).expect("catalog");
        let default = switch_appearance_from_catalog(
            &catalog,
            &mode.metrics,
            &mode.typography,
            ThemeMode::Light,
            RadixButtonStyle::Primary,
            false,
            InteractionState::default(),
        )
        .expect("default");
        let hovered = switch_appearance_from_catalog(
            &catalog,
            &mode.metrics,
            &mode.typography,
            ThemeMode::Light,
            RadixButtonStyle::Primary,
            false,
            InteractionState { hovered: true, ..InteractionState::default() },
        )
        .expect("hovered");
        let pressed = switch_appearance_from_catalog(
            &catalog,
            &mode.metrics,
            &mode.typography,
            ThemeMode::Light,
            RadixButtonStyle::Primary,
            false,
            InteractionState { hovered: true, pressed: true, ..InteractionState::default() },
        )
        .expect("pressed");

        assert_eq!(default.track_background, hovered.track_background);
        assert_eq!(default.track_background, pressed.track_background);
        assert_eq!(default.thumb_background, hovered.thumb_background);
    }

    #[test]
    fn primary_and_secondary_share_off_appearance() {
        let catalog = sample_catalog();
        let mode = RadixModeTokens::from_catalog(catalog.clone()).expect("catalog");
        let primary = switch_appearance_from_catalog(
            &catalog,
            &mode.metrics,
            &mode.typography,
            ThemeMode::Light,
            RadixButtonStyle::Primary,
            false,
            InteractionState::default(),
        )
        .expect("primary");
        let secondary = switch_appearance_from_catalog(
            &catalog,
            &mode.metrics,
            &mode.typography,
            ThemeMode::Light,
            RadixButtonStyle::Secondary,
            false,
            InteractionState::default(),
        )
        .expect("secondary");

        assert_eq!(primary.track_background, secondary.track_background);
        assert_eq!(primary.thumb_background, secondary.thumb_background);
    }
}
