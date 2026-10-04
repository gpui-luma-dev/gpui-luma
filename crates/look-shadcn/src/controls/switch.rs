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
use crate::shadow::parse_shadow_token;
use super::button::{ButtonRadiusPreset, resolve_button_radius_preset};
use super::ShadcnButtonStyle;
use crate::mode::ShadcnModeTokens;
use crate::stylesheet::{
    StylesheetConfig, find_switch_color_rule, resolve_layered_elevation_shadow, resolve_switch_color_rule,
    resolve_switch_metrics,
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
    resolve_switch_colors_with_stylesheet(resolver, resolver.stylesheet(), style, on, disabled)
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

/// Switch geometry from the embedded common contract, with legacy metric fallback.
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
    switch_scale_with_stylesheet(mode, mode.stylesheet(), theme_mode, style, size, radius, scale_factor)
}

pub(crate) fn switch_geometry_with_stylesheet(
    mode: &ShadcnModeTokens,
    stylesheet: &StylesheetConfig,
    size: ControlSize,
    scale_factor: f32,
) -> gpui_luma::theme::stylesheet::ResolvedSwitchGeometry {
    let mut fallback = SwitchScale::compute(size, &mode.metrics, scale_factor);
    // Legacy sections remain a compatibility fallback below the common contract.
    if let Some(rule) = stylesheet.switch.metrics_for_size(size) {
        let resolved = resolve_switch_metrics(rule);
        fallback.track_width = snap_to_pixel(resolved.width, scale_factor);
        fallback.track_height = snap_to_pixel(resolved.height, scale_factor);
        fallback.thumb_size = snap_to_pixel(resolved.thumb_size, scale_factor);
    }
    stylesheet.common.switch.resolve_geometry(
        match size {
            ControlSize::Sm => "sm",
            ControlSize::Md => "md",
            ControlSize::Lg => "lg",
        },
        fallback,
        scale_factor,
    )
}

pub(crate) fn switch_scale_with_stylesheet(
    mode: &ShadcnModeTokens,
    stylesheet: &StylesheetConfig,
    theme_mode: ThemeMode,
    _style: ShadcnButtonStyle,
    size: ControlSize,
    radius: Option<ButtonRadiusPreset>,
    scale_factor: f32,
) -> SwitchScale {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let fallback = SwitchScale::compute(size, metrics, scale_factor);
    let geometry = switch_geometry_with_stylesheet(mode, stylesheet, size, scale_factor);
    let scale = geometry.apply_to(fallback);
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

/// Final switch colors shared by runtime and inspection.
#[derive(Clone, Debug)]
pub struct SwitchResolvedColors {
    pub track_background: ResolvedColor,
    pub track_border: ResolvedColor,
    pub thumb_background: ResolvedColor,
    pub thumb_border: ResolvedColor,
    pub label_color: ResolvedColor,
}

pub fn resolve_switch_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    on: bool,
    state: InteractionState,
) -> SwitchResolvedColors {
    let content_only = style == ShadcnButtonStyle::ContentOnly;
    let indicator_style = if content_only {
        ShadcnButtonStyle::Primary
    } else {
        style
    };
    let ctx = LookContext::new(mode, theme_mode, state);
    let catalog = ctx.catalog();
    let resolver = LookResolver::new(catalog, ctx.theme_mode, "switch").with_stylesheet(mode.stylesheet());
    let colors = resolve_switch_colors(&resolver, indicator_style, on, state.disabled)
        .unwrap_or_else(|_| SwitchColorTable::fallback());

    let track_background = colors.track_background;
    let track_border = if state.focused && !state.disabled {
        resolver.resolve_decl("ring").unwrap_or_else(|_| ResolvedColor::fallback_foreground())
    } else if on && !state.disabled {
        track_background.clone()
    } else {
        resolver.resolve_decl("border").unwrap_or_else(|_| ResolvedColor::fallback_foreground())
    };
    let mut thumb_background = colors.thumb_background;
    let mut thumb_border = if on && !state.disabled {
        thumb_background.clone()
    } else {
        colors.thumb_border
    };
    if content_only {
        let value = if theme_mode == ThemeMode::Dark {
            gpui::hsla(0.0, 0.0, 0.0, 1.0)
        } else {
            gpui::hsla(0.0, 0.0, 1.0, 1.0)
        };
        thumb_background =
            ResolvedColor { value, source: crate::ColorSource::Derived { note: "content-only thumb contrast".into() } };
        thumb_border = thumb_background.clone();
    }
    SwitchResolvedColors {
        track_background,
        track_border,
        thumb_background,
        thumb_border,
        label_color: colors.label_color,
    }
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
    let ctx = LookContext::new(mode, theme_mode, state);
    let catalog = ctx.catalog();
    let typography = ctx.typography();
    let layer = state.layer();
    let colors = resolve_switch_palette(mode, theme_mode, style, on, state);
    SwitchPalette {
        track_background: colors.track_background.hsla(),
        track_border: colors.track_border.hsla(),
        thumb_background: colors.thumb_background.hsla(),
        thumb_border: colors.thumb_border.hsla(),
        thumb_shadow: if content_only {
            Vec::new()
        } else {
            switch_elevation_shadow(catalog, mode.stylesheet(), layer)
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
    fn legacy_switch_metrics_remain_a_fallback_below_common_fields() {
        let stylesheet = crate::stylesheet::StylesheetConfig::parse(
            "[switch.metrics.md]\nwidth = 46\nheight = 24\nthumb_size = 20\n[common.switch.sizes.md]\ntrack_width = 50",
        )
        .unwrap();
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).unwrap();
        let scale = super::switch_scale_with_stylesheet(
            &mode,
            &stylesheet,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ControlSize::Md,
            None,
            1.0,
        );
        assert_eq!((scale.track_width, scale.track_height, scale.thumb_size), (50.0, 24.0, 20.0));
        let missing = super::switch_scale_with_stylesheet(
            &mode,
            &crate::stylesheet::StylesheetConfig::default(),
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ControlSize::Md,
            None,
            1.0,
        );
        assert_eq!(missing, gpui_luma::controls::switch::SwitchScale::compute(ControlSize::Md, &mode.metrics, 1.0));
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
        let theme = crate::test_support::built_in_look("astrovista");
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
        let theme = crate::test_support::built_in_look("astrovista");
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
    fn content_only_switch_keeps_primary_track_and_explicit_light_thumb_without_shadow() {
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
        assert_eq!(content_only.thumb_background, gpui::hsla(0.0, 0.0, 1.0, 1.0));
        assert_eq!(content_only.thumb_border, gpui::hsla(0.0, 0.0, 1.0, 1.0));
        assert!(content_only.thumb_shadow.is_empty());
    }

    fn mode_without_border_and_ring() -> ShadcnModeTokens {
        let mut mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        mode.catalog.tokens.remove("ring");
        mode.catalog.tokens.remove("border");
        mode
    }

    #[test]
    fn incomplete_catalog_does_not_panic_on_focus_or_border() {
        let mode = mode_without_border_and_ring();
        let fallback = crate::provenance::ResolvedColor::fallback_foreground().hsla();
        let focused = switch_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            false,
            InteractionState { focused: true, ..InteractionState::default() },
            ControlSize::Md,
        );
        let unfocused = switch_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            false,
            InteractionState::default(),
            ControlSize::Md,
        );

        assert_eq!(focused.track_border, fallback);
        assert_eq!(unfocused.track_border, fallback);
    }
}
