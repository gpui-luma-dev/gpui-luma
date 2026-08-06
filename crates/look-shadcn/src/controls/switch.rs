//! Switch property mappings (shadcn Switch):
//!
//! | State    | Track token | Track border     | Thumb token              |
//! |----------|-------------|------------------|--------------------------|
//! | Off      | `input`     | `border`         | `background` / `border`  |
//! | On       | `{style}`   | matches track bg | `{style}-foreground`     |
//! | Disabled | `muted`     | `border`         | `muted-foreground` / `muted` |
//!
//! Hover and pressed do not recolor the track or thumb (shadcn Switch has no hover
//! surface).

use gpui_luma::controls::switch::{SwitchPalette, SwitchScale};
use gpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, MetricTokens, ThemeMode, snap_to_pixel};

use super::apply_button_metrics_typography;

use crate::look_context::LookContext;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::resolve::resolve_color;
use crate::shadow::parse_shadow_token;
use super::button::{ButtonRadiusPreset, resolve_button_radius_preset};
use super::ShadcnButtonStyle;
use crate::mode::ShadcnModeTokens;
use crate::stylesheet::{
    StylesheetConfig, embedded_stylesheet, find_switch_color_rule, resolve_layered_elevation_shadow,
    resolve_switch_color_rule, resolve_switch_metrics,
};

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

/// Corner radius for switch track / thumb (same presets as button / icon button / toggle).
pub fn resolve_switch_radius_preset(preset: ButtonRadiusPreset, metrics: &MetricTokens, track_height: f32) -> f32 {
    resolve_button_radius_preset(preset, metrics, track_height)
}

/// Look-owned switch geometry from `style.toml` (`switch.metrics`).
pub fn switch_scale(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    size: ControlSize,
    scale_factor: f32,
) -> SwitchScale {
    switch_scale_with_radius(mode, theme_mode, style, size, None, scale_factor)
}

pub fn switch_scale_with_radius(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    size: ControlSize,
    radius: Option<ButtonRadiusPreset>,
    scale_factor: f32,
) -> SwitchScale {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let fallback = SwitchScale::compute(size, metrics, scale_factor);
    let stylesheet = embedded_stylesheet();
    let Some(rule) = stylesheet.switch.metrics_for_style(style, size) else {
        return apply_switch_radius(fallback, metrics, radius);
    };
    let resolved = resolve_switch_metrics(rule);
    let track_height = snap_to_pixel(resolved.height, scale_factor);
    let track_padding = snap_to_pixel((resolved.height * (2.0 / 22.0)).max(1.0), scale_factor);
    let scale = SwitchScale {
        track_width: snap_to_pixel(resolved.width, scale_factor),
        track_height,
        track_padding,
        thumb_size: snap_to_pixel(resolved.thumb_size, scale_factor),
        track_radius: metrics.radius.pill,
        gap: snap_to_pixel(metrics.gap(size), scale_factor),
        label_baseline_shift: fallback.label_baseline_shift,
    };
    apply_switch_radius(scale, metrics, radius)
}

fn apply_switch_radius(
    mut scale: SwitchScale,
    metrics: &MetricTokens,
    radius: Option<ButtonRadiusPreset>,
) -> SwitchScale {
    if let Some(preset) = radius {
        scale.track_radius = resolve_switch_radius_preset(preset, metrics, scale.track_height);
    }
    scale
}

pub fn switch_look(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    on: bool,
    state: InteractionState,
    size: ControlSize,
) -> SwitchPalette {
    let content_only = style == ShadcnButtonStyle::ContentOnly;
    let indicator_style = if content_only {
        ShadcnButtonStyle::Primary
    } else {
        style
    };
    let ctx = LookContext::new(mode, theme_mode, state);
    let state = ctx.state;
    let catalog = ctx.catalog();
    let typography = ctx.typography();
    let layer = state.layer();
    let resolver = LookResolver::new(catalog, ctx.theme_mode, "switch");
    let colors = resolve_switch_colors(&resolver, indicator_style, on, state.disabled)
        .unwrap_or_else(|_| SwitchColorTable::fallback());

    let track_background = colors.track_background.hsla();
    let track_border = if state.focused && !state.disabled {
        crate::focus::focus_ring_color(catalog).unwrap_or_else(|err| panic!("switch focus ring: {err}"))
    } else if on && !state.disabled {
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
    SwitchPalette {
        track_background,
        track_border,
        thumb_background,
        thumb_border,
        thumb_shadow: if content_only {
            Vec::new()
        } else {
            switch_elevation_shadow(catalog, embedded_stylesheet(), layer)
        },
        label_color: colors.label_color.hsla(),
        label_typography: {
            let mut label_typography = typography.text.label;
            apply_button_metrics_typography(&mut label_typography, mode, size);
            label_typography
        },
        label_font_family: typography.font.sans.family.clone().into(),
    }
}

fn switch_elevation_shadow(
    catalog: &crate::catalog::CssTokenMap,
    stylesheet: &StylesheetConfig,
    layer: InteractionLayer,
) -> Vec<gpui::BoxShadow> {
    let Some(token) = resolve_layered_elevation_shadow(&stylesheet.switch.elevation_rules, layer) else {
        return Vec::new();
    };
    parse_shadow_token(catalog, &token).unwrap_or_default()
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};

    use crate::controls::button::ShadcnButtonStyle;
    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::switch_look;

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
            ("shadow-sm".into(), "0 1px 3px 0px hsl(0 0% 0% / 0.10), 0 1px 2px -1px hsl(0 0% 0% / 0.10)".into()),
        ]))
    }

    #[test]
    fn switch_look_resolves_stylesheet_shadow() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let look = switch_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            false,
            InteractionState::default(),
            ControlSize::Md,
        );

        assert!(!look.thumb_shadow.is_empty());
    }

    #[test]
    fn disabled_switch_look_has_no_shadow() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let look = switch_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            false,
            InteractionState { disabled: true, ..InteractionState::default() },
            ControlSize::Md,
        );

        assert!(look.thumb_shadow.is_empty());
    }

    #[test]
    fn off_switch_uses_input_track_and_background_thumb() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = switch_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            false,
            InteractionState::default(),
            ControlSize::Md,
        );

        let input = catalog.color("input").expect("input");
        let border = catalog.color("border").expect("border");
        let background = catalog.color("background").expect("background");
        assert_eq!(look.track_background, input);
        assert_eq!(look.track_border, border);
        assert_eq!(look.thumb_background, background);
        assert_eq!(look.thumb_border, border);
    }

    #[test]
    fn on_switch_uses_style_track_and_card_thumb() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = switch_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            true,
            InteractionState::default(),
            ControlSize::Md,
        );

        let primary = catalog.color("primary").expect("primary");
        let primary_foreground = catalog.color("primary-foreground").expect("primary-foreground");
        assert_eq!(look.track_background, primary);
        assert_eq!(look.track_border, primary);
        assert_eq!(look.thumb_background, primary_foreground);
        assert!(look.thumb_background.l > look.track_background.l);
    }

    #[test]
    fn astrovista_light_off_switch_uses_background_thumb_and_border_track() {
        let theme = crate::ShadcnLook::from_built_in_theme("astrovista").expect("astrovista css");
        let mode_tokens = theme.mode_tokens();
        let look = switch_look(
            mode_tokens.as_ref(),
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            false,
            InteractionState::default(),
            ControlSize::Md,
        );

        let catalog = &theme.mode_tokens().catalog;
        let input = catalog.color("input").expect("input");
        let border = catalog.color("border").expect("border");
        let background = catalog.color("background").expect("background");
        let card = catalog.color("card").expect("card");
        assert_eq!(look.track_background, input);
        assert_eq!(look.track_border, border);
        assert_eq!(look.thumb_background, background);
        assert_eq!(look.thumb_border, border);
        assert_ne!(look.thumb_background, card);
        assert!(look.thumb_background.l < look.track_background.l);
    }

    #[test]
    fn astrovista_dark_off_switch_uses_background_thumb() {
        let theme = crate::ShadcnLook::from_built_in_theme("astrovista").expect("astrovista css");
        theme.set_mode(ThemeMode::Dark);
        let mode_tokens = theme.mode_tokens();
        let look = switch_look(
            mode_tokens.as_ref(),
            ThemeMode::Dark,
            ShadcnButtonStyle::Primary,
            false,
            InteractionState::default(),
            ControlSize::Md,
        );

        let catalog = &theme.mode_tokens().catalog;
        let input = catalog.color("input").expect("input");
        let background = catalog.color("background").expect("background");
        assert_eq!(look.track_background, input);
        assert_eq!(look.thumb_background, background);
    }

    #[test]
    fn hover_and_pressed_match_default_track_for_off_switch() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let default = switch_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            false,
            InteractionState::default(),
            ControlSize::Md,
        );
        let hovered = switch_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            false,
            InteractionState { hovered: true, ..InteractionState::default() },
            ControlSize::Md,
        );
        let pressed = switch_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            false,
            InteractionState { hovered: true, pressed: true, ..InteractionState::default() },
            ControlSize::Md,
        );

        assert_eq!(default.track_background, hovered.track_background);
        assert_eq!(default.track_background, pressed.track_background);
        assert_eq!(default.thumb_background, hovered.thumb_background);
    }

    #[test]
    fn primary_and_secondary_share_off_look() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let primary = switch_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            false,
            InteractionState::default(),
            ControlSize::Md,
        );
        let secondary = switch_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Secondary,
            false,
            InteractionState::default(),
            ControlSize::Md,
        );

        assert_eq!(primary.track_background, secondary.track_background);
        assert_eq!(primary.thumb_background, secondary.thumb_background);
    }

    #[test]
    fn switch_variants_share_geometry() {
        use gpui_luma::theme::ControlSize;
        use super::switch_scale;

        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
            let primary = switch_scale(&mode, ThemeMode::Light, ShadcnButtonStyle::Primary, size, 1.0);
            let secondary = switch_scale(&mode, ThemeMode::Light, ShadcnButtonStyle::Secondary, size, 1.0);
            let content_only = switch_scale(&mode, ThemeMode::Light, ShadcnButtonStyle::ContentOnly, size, 1.0);

            assert_eq!(secondary.track_height, primary.track_height, "{size:?}: secondary height");
            assert_eq!(secondary.track_width, primary.track_width, "{size:?}: secondary width");
            assert_eq!(secondary.thumb_size, primary.thumb_size, "{size:?}: secondary thumb");
            assert_eq!(content_only.track_height, primary.track_height, "{size:?}: content-only height");
            assert_eq!(content_only.track_width, primary.track_width, "{size:?}: content-only width");
            assert_eq!(content_only.thumb_size, primary.thumb_size, "{size:?}: content-only thumb");
        }
    }

    #[test]
    fn on_primary_and_secondary_use_distinct_style_colors() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let primary = switch_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            true,
            InteractionState::default(),
            ControlSize::Md,
        );
        let secondary = switch_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Secondary,
            true,
            InteractionState::default(),
            ControlSize::Md,
        );

        assert_eq!(primary.track_background, catalog.color("primary").expect("primary"));
        assert_eq!(secondary.track_background, catalog.color("secondary").expect("secondary"));
        assert_ne!(primary.track_background, secondary.track_background);
    }

    #[test]
    fn switch_radius_presets_match_button_scale() {
        use gpui_luma::theme::ControlSize;
        use super::{resolve_switch_radius_preset, switch_scale};

        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let metrics = mode.metrics;
        let scale = switch_scale(&mode, ThemeMode::Light, ShadcnButtonStyle::Primary, ControlSize::Md, 1.0);

        assert_eq!(
            resolve_switch_radius_preset(
                crate::controls::button::ButtonRadiusPreset::None,
                &metrics,
                scale.track_height
            ),
            metrics.radius.none
        );
        assert_eq!(
            resolve_switch_radius_preset(
                crate::controls::button::ButtonRadiusPreset::Full,
                &metrics,
                scale.track_height
            ),
            scale.track_height / 2.0
        );
        assert!(
            resolve_switch_radius_preset(
                crate::controls::button::ButtonRadiusPreset::Small,
                &metrics,
                scale.track_height
            ) < scale.track_height / 2.0
        );
    }

    #[test]
    fn content_only_switch_keeps_primary_track_without_shadow() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let primary = switch_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            true,
            InteractionState::default(),
            ControlSize::Md,
        );
        let content_only = switch_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::ContentOnly,
            true,
            InteractionState { focused: true, ..InteractionState::default() },
            ControlSize::Md,
        );

        assert_eq!(content_only.track_background, primary.track_background);
        assert_eq!(content_only.track_border, primary.track_border);
        assert_eq!(content_only.thumb_background, primary.thumb_background);
        assert!(content_only.thumb_shadow.is_empty());
    }
}
