use gpui_luma::controls::button_family::{
    ButtonFamilyLook, ButtonFamilyPalette, ButtonFamilyRole, compose_button_family_look,
};
use gpui_luma::theme::{ControlSize, InteractionState, StandardBoxScale, ThemeMode, snap_to_pixel};

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::LookResolver;
use crate::stylesheet::{StylesheetConfig, embedded_stylesheet, resolve_button_metrics_rule};

use super::button::{
    ButtonColorPalette, ButtonRadiusPreset, ShadcnButtonStyle, button_box_scale, button_elevation_shadow,
    effective_button_style_for_role, resolve_button_colors_with_stylesheet, resolve_button_radius_preset,
    toggle_elevation_shadow,
};

/// Inline padded toggle (shadcn default). Square geometry requires explicit `.round(true)` on the builder.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToggleLayout {
    #[default]
    Inline,
    Square,
}

impl ToggleLayout {
    pub fn from_round(round: bool) -> Self {
        if round { Self::Square } else { Self::Inline }
    }
}

pub fn toggle_look(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    selected: bool,
    size: ControlSize,
    state: InteractionState,
) -> ButtonFamilyLook {
    toggle_look_semantic(mode, theme_mode, style, selected, size, None, state)
}

pub fn toggle_look_semantic(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    selected: bool,
    size: ControlSize,
    radius: Option<ButtonRadiusPreset>,
    state: InteractionState,
) -> ButtonFamilyLook {
    let role = ButtonFamilyRole::Toggle { selected };
    let ctx = LookContext::new(mode, theme_mode, state);
    let stylesheet = embedded_stylesheet();
    let palette = toggle_palette(&ctx, stylesheet, style, selected, size);
    let scale = toggle_box_scale(&ctx, stylesheet, size, 1.0);
    let mut look = compose_button_family_look(&palette, role, &scale, ctx.metrics().radius.pill);
    if let Some(rule) = stylesheet.toggle.metrics_for_size(size) {
        look.icon_size = resolve_button_metrics_rule(rule, ctx.metrics(), size).icon_size;
    }
    if let Some(radius) = radius {
        look.radius = resolve_button_radius_preset(radius, ctx.metrics(), look.height);
    }
    look.shadow = toggle_shadow(&ctx, stylesheet, style);
    look
}

/// Content-only toggles have no selected color chrome; styled toggles use their
/// selected stylesheet state, including outline and ghost variants.
fn toggle_color_selected(style: ShadcnButtonStyle, selected: bool) -> bool {
    selected && !matches!(style, ShadcnButtonStyle::ContentOnly)
}

fn toggle_shadow(
    ctx: &LookContext,
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
) -> Option<Vec<gpui::BoxShadow>> {
    match style {
        ShadcnButtonStyle::Outline | ShadcnButtonStyle::Ghost => button_elevation_shadow(ctx, stylesheet, style),
        ShadcnButtonStyle::ContentOnly => None,
        _ => toggle_elevation_shadow(ctx.catalog(), stylesheet, ctx.state.layer()),
    }
}

/// Square icon toggle sizing (matches icon-button metrics); radius preset controls corners.
pub fn toggle_icon_look_semantic(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    selected: bool,
    size: ControlSize,
    radius: Option<ButtonRadiusPreset>,
    state: InteractionState,
) -> ButtonFamilyLook {
    let role = ButtonFamilyRole::Toggle { selected };
    let ctx = LookContext::new(mode, theme_mode, state);
    let stylesheet = embedded_stylesheet();
    let palette = toggle_palette(&ctx, stylesheet, style, selected, size);
    let mut scale = button_box_scale(&ctx, stylesheet, size, 1.0);
    scale.padding_x = 0.0;
    scale.padding_y = 0.0;
    let mut look = compose_button_family_look(&palette, role, &scale, ctx.metrics().radius.pill);
    if let Some(rule) = stylesheet.button.metrics_for_size(size) {
        look.icon_size = resolve_button_metrics_rule(rule, ctx.metrics(), size).icon_size;
    }
    if let Some(radius) = radius {
        look.radius = resolve_button_radius_preset(radius, ctx.metrics(), look.height);
    }
    look.shadow = toggle_shadow(&ctx, stylesheet, style);
    look
}

pub fn toggle_box_scale(
    ctx: &LookContext,
    stylesheet: &StylesheetConfig,
    size: ControlSize,
    scale_factor: f32,
) -> StandardBoxScale {
    let fallback = StandardBoxScale::compute(size, ctx.metrics(), scale_factor);
    let Some(rule) = stylesheet.toggle.metrics_for_size(size) else {
        return fallback;
    };

    let metrics = resolve_button_metrics_rule(rule, ctx.metrics(), size);
    StandardBoxScale {
        height: snap_to_pixel(metrics.height, scale_factor),
        padding_x: snap_to_pixel(metrics.padding_horizontal, scale_factor),
        padding_y: fallback.padding_y,
        gap: fallback.gap,
        radius: metrics.corner_radius,
        icon_size: snap_to_pixel(metrics.icon_size, scale_factor),
    }
}

pub fn toggle_palette(
    ctx: &LookContext,
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
    selected: bool,
    size: ControlSize,
) -> ButtonFamilyPalette {
    let role = ButtonFamilyRole::Toggle { selected };
    let requested_style = style;
    let effective_style = effective_button_style_for_role(requested_style, role);
    let layer = ctx.state.layer();
    let theme_mode = ctx.theme_mode;

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "toggle_resolver");
    let color_selected = toggle_color_selected(requested_style, selected);
    let mut colors = resolve_button_colors_with_stylesheet(
        &resolver,
        stylesheet,
        effective_style,
        layer,
        theme_mode,
        color_selected,
    )
    .unwrap_or_else(|_| ButtonColorPalette::fallback());
    if ctx.state.focused && !ctx.state.disabled {
        if let Ok(focus_border) = resolver.resolve_decl("ring") {
            colors.border = Some(focus_border);
        }
    }

    let size_metrics = stylesheet
        .toggle
        .metrics_for_size(size)
        .map(|rule| resolve_button_metrics_rule(rule, ctx.metrics(), size));

    let background = colors.background.hsla();
    let foreground = colors.foreground.hsla();
    let border = colors.border.map(|color| color.hsla());

    let mut typography = ctx.typography().text.label;
    if let Some(metrics) = size_metrics {
        let base_size = typography.size;
        typography.size = metrics.font_size;
        if base_size > 0.0 {
            typography.line_height = metrics.font_size * (typography.line_height / base_size);
        }
    }

    ButtonFamilyPalette {
        background,
        foreground,
        border,
        typography,
        font_family: ctx.typography().font.sans.family.clone().into(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::controls::button::button_look;
    use crate::catalog::CssTokenMap;
    use crate::controls::button::ButtonRadiusPreset;
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
    fn toggle_look_uses_toggle_metrics_not_button_metrics() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let sm = toggle_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Secondary,
            true,
            ControlSize::Sm,
            InteractionState::default(),
        );
        let lg = toggle_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Secondary,
            true,
            ControlSize::Lg,
            InteractionState::default(),
        );

        assert!((sm.height - 36.0).abs() < f32::EPSILON);
        assert!((sm.padding_x - 10.0).abs() < f32::EPSILON);
        assert!((lg.height - 44.0).abs() < f32::EPSILON);
        assert!((lg.padding_x - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn toggle_inline_look_has_horizontal_padding() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let look = toggle_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Secondary,
            false,
            ControlSize::Md,
            InteractionState::default(),
        );

        assert!(look.padding_x > 0.0);
    }

    #[test]
    fn focused_toggle_uses_ring_border() {
        let catalog = retro_arcade_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let focused = toggle_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::ContentOnly,
            false,
            ControlSize::Md,
            InteractionState { focused: true, ..InteractionState::default() },
        );

        assert_eq!(focused.border, Some(catalog.color("ring").expect("ring")));
    }

    #[test]
    fn outline_toggle_selected_uses_selected_button_colors() {
        let catalog = retro_arcade_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let icon = button_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Outline,
            ButtonFamilyRole::Icon,
            ControlSize::Md,
            InteractionState::default(),
        );
        let toggle = toggle_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Outline,
            true,
            ControlSize::Md,
            InteractionState::default(),
        );

        assert_ne!(toggle.background, icon.background);
        assert_ne!(toggle.foreground, icon.foreground);
        assert_eq!(toggle.shadow, icon.shadow);
    }

    #[test]
    fn ghost_toggle_selected_uses_selected_button_colors() {
        let catalog = retro_arcade_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let icon = button_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Ghost,
            ButtonFamilyRole::Icon,
            ControlSize::Md,
            InteractionState::default(),
        );
        let toggle = toggle_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Ghost,
            true,
            ControlSize::Md,
            InteractionState::default(),
        );

        assert_ne!(toggle.background, icon.background);
        assert_ne!(toggle.foreground, icon.foreground);
        assert_eq!(toggle.border, icon.border);
        assert!(toggle.shadow.is_none());
        assert!(icon.shadow.is_none());
    }

    #[test]
    fn toggle_icon_look_uses_button_metrics_not_toggle_inline_metrics() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let inline = toggle_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Secondary,
            true,
            ControlSize::Md,
            InteractionState::default(),
        );
        let icon = toggle_icon_look_semantic(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            true,
            ControlSize::Md,
            None,
            InteractionState::default(),
        );

        assert!(inline.padding_x > 0.0);
        assert_eq!(icon.padding_x, 0.0);
        assert_eq!(icon.padding_y, 0.0);
        assert_ne!(inline.height, icon.height);
    }

    #[test]
    fn toggle_look_resolves_radius_preset() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let full = toggle_look_semantic(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Secondary,
            true,
            ControlSize::Md,
            Some(ButtonRadiusPreset::Full),
            InteractionState::default(),
        );

        assert!((full.radius - full.height / 2.0).abs() < f32::EPSILON);
    }
}
