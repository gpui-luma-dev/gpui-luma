use gpui_luma::controls::button_family::{
    ButtonFamilyLook, ButtonFamilyPalette, ButtonFamilyRole, compose_button_family_look,
};
use gpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, StandardBoxScale, ThemeMode, snap_to_pixel};

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::stylesheet::{
    StylesheetConfig, embedded_stylesheet, find_button_color_rule, resolve_button_color_rule,
    resolve_button_metrics_rule,
};

/// Radix-style button look.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShadcnButtonStyle {
    Primary,
    Secondary,
    Outline,
    Ghost,
}

#[derive(Clone, Debug)]
pub struct ButtonColorPalette {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: Option<ResolvedColor>,
}

impl ButtonColorPalette {
    pub fn fallback() -> Self {
        Self {
            background: ResolvedColor::transparent(),
            foreground: ResolvedColor::fallback_foreground(),
            border: None,
        }
    }
}

pub fn resolve_button_colors(
    resolver: &LookResolver<'_>,
    style: ShadcnButtonStyle,
    layer: InteractionLayer,
    theme_mode: ThemeMode,
    selected: bool,
) -> anyhow::Result<ButtonColorPalette> {
    resolve_button_colors_with_stylesheet(resolver, embedded_stylesheet(), style, layer, theme_mode, selected)
}

pub fn resolve_button_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
    layer: InteractionLayer,
    theme_mode: ThemeMode,
    selected: bool,
) -> anyhow::Result<ButtonColorPalette> {
    let rule = find_button_color_rule(stylesheet, style, layer, theme_mode, selected)
        .ok_or_else(|| anyhow::anyhow!("no matching button color rule"))?;

    let colors = resolve_button_color_rule(resolver, rule)?;
    Ok(ButtonColorPalette { background: colors.background, foreground: colors.foreground, border: colors.border })
}

pub fn button_look(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    size: ControlSize,
    state: InteractionState,
) -> ButtonFamilyLook {
    let ctx = LookContext::new(mode, theme_mode, state);
    let stylesheet = embedded_stylesheet();
    let palette = button_palette(&ctx, stylesheet, style, role, size);
    let scale = button_box_scale(&ctx, stylesheet, size, 1.0);
    compose_button_family_look(&palette, role, &scale, ctx.metrics().radius.pill)
}

pub fn button_box_scale(
    ctx: &LookContext,
    stylesheet: &StylesheetConfig,
    size: ControlSize,
    scale_factor: f32,
) -> StandardBoxScale {
    let fallback = StandardBoxScale::compute(size, ctx.metrics(), scale_factor);
    let Some(rule) = stylesheet.button.metrics_for_size(size) else {
        return fallback;
    };

    let metrics = resolve_button_metrics_rule(rule, ctx.metrics(), size);
    StandardBoxScale {
        height: snap_to_pixel(metrics.height, scale_factor),
        padding_x: snap_to_pixel(metrics.padding_horizontal, scale_factor),
        padding_y: fallback.padding_y,
        gap: fallback.gap,
        radius: metrics.corner_radius,
    }
}

pub fn button_palette(
    ctx: &LookContext,
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    size: ControlSize,
) -> ButtonFamilyPalette {
    let style = effective_button_style(style, role);
    let layer = ctx.state.layer();
    let theme_mode = ctx.theme_mode;
    let selected = matches!(role, ButtonFamilyRole::Toggle { selected: true });

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "button_resolver");
    let colors = resolve_button_colors_with_stylesheet(&resolver, stylesheet, style, layer, theme_mode, selected)
        .unwrap_or_else(|_| ButtonColorPalette::fallback());

    let size_metrics = stylesheet
        .button
        .metrics_for_size(size)
        .map(|rule| resolve_button_metrics_rule(rule, ctx.metrics(), size));

    let background = colors.background.hsla();
    let foreground = colors.foreground.hsla();
    let border = colors.border.map(|color| color.hsla());

    let mut typography = ctx.typography().text.label;
    if let Some(metrics) = size_metrics {
        typography.size = metrics.font_size;
    }

    ButtonFamilyPalette {
        background,
        foreground,
        border,
        focus_ring: ctx.palette().focus_ring,
        typography,
        font_family: ctx.typography().font.sans.family.clone().into(),
    }
}

fn effective_button_style(style: ShadcnButtonStyle, role: ButtonFamilyRole) -> ShadcnButtonStyle {
    if matches!(role, ButtonFamilyRole::Toggle { selected: false }) {
        ShadcnButtonStyle::Outline
    } else {
        style
    }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use super::*;
    use gpui_luma::controls::button_family::ButtonFamilyRole;
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
        ]))
    }

    #[test]
    fn primary_hover_background_differs_from_default() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let default = button_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            InteractionState::default(),
        );
        let hovered = button_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            InteractionState { hovered: true, ..InteractionState::default() },
        );

        assert_ne!(default.background, hovered.background);
    }

    #[test]
    fn ghost_light_hover_pairs_accent_fill_with_accent_foreground() {
        let catalog = retro_arcade_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let hovered = button_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Ghost,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            InteractionState { hovered: true, ..InteractionState::default() },
        );

        assert_eq!(hovered.background, catalog.color("accent").expect("accent"));
        assert_eq!(hovered.foreground, catalog.color("accent-foreground").expect("accent-foreground"));
    }

    #[test]
    fn ghost_dark_hover_pairs_accent_half_fill_with_accent_foreground() {
        let catalog = retro_arcade_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Dark).expect("catalog");
        let accent = catalog.color("accent").expect("accent");
        let hovered = button_look(
            &mode,
            ThemeMode::Dark,
            ShadcnButtonStyle::Ghost,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            InteractionState { hovered: true, ..InteractionState::default() },
        );

        assert!((hovered.background.a - 0.50).abs() < f32::EPSILON);
        assert_eq!(hovered.background.h, accent.h);
        assert_eq!(hovered.foreground, catalog.color("accent-foreground").expect("accent-foreground"));
    }

    #[test]
    fn outline_light_hover_uses_accent_fill_and_accent_foreground() {
        let catalog = retro_arcade_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let hovered = button_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Outline,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            InteractionState { hovered: true, ..InteractionState::default() },
        );

        assert_eq!(hovered.background, catalog.color("accent").expect("accent"));
        assert_eq!(hovered.foreground, catalog.color("accent-foreground").expect("accent-foreground"));
    }

    #[test]
    fn outline_dark_hover_uses_input_alpha_with_accent_foreground() {
        let catalog = retro_arcade_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Dark).expect("catalog");
        let input = catalog.color("input").expect("input");
        let hovered = button_look(
            &mode,
            ThemeMode::Dark,
            ShadcnButtonStyle::Outline,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            InteractionState { hovered: true, ..InteractionState::default() },
        );

        assert!((hovered.background.a - 0.50).abs() < f32::EPSILON);
        assert_eq!(hovered.background.h, input.h);
        assert_eq!(hovered.foreground, catalog.color("accent-foreground").expect("accent-foreground"));
        assert_eq!(gpui_luma::controls::button_family::button_family_effective_border(hovered.border), input);
    }

    #[test]
    fn retro_arcade_primary_hover_is_subtle() {
        let catalog = retro_arcade_catalog();
        let light = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("light");
        let dark = ShadcnModeTokens::from_catalog(catalog, ThemeMode::Dark).expect("dark");

        for (mode, theme_mode) in [(&light, ThemeMode::Light), (&dark, ThemeMode::Dark)] {
            let default = button_look(
                mode,
                theme_mode,
                ShadcnButtonStyle::Primary,
                ButtonFamilyRole::Text,
                ControlSize::Md,
                InteractionState::default(),
            );
            let hovered = button_look(
                mode,
                theme_mode,
                ShadcnButtonStyle::Primary,
                ButtonFamilyRole::Text,
                ControlSize::Md,
                InteractionState { hovered: true, ..InteractionState::default() },
            );
            assert!((default.background.l - hovered.background.l).abs() < 0.08);
        }
    }

    #[test]
    fn palette_typography_uses_stylesheet_font_size() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let ctx = LookContext::new(&mode, ThemeMode::Light, InteractionState::default());
        let palette = button_palette(
            &ctx,
            embedded_stylesheet(),
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            ControlSize::Sm,
        );

        assert!((palette.typography.size - 12.0).abs() < f32::EPSILON);
    }
}
