use gpui_luma::controls::button_family::{
    ButtonFamilyLook, ButtonFamilyPalette, ButtonFamilyRole, compose_button_family_look,
};
use gpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, StandardBoxScale, ThemeMode, snap_to_pixel};

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::shadow::parse_shadow_token;
use crate::stylesheet::{
    StylesheetConfig, embedded_stylesheet, find_button_color_rule, find_button_elevation_rule,
    resolve_button_color_rule, resolve_button_metrics_rule, resolve_layered_elevation_shadow,
    resolve_stylesheet_shadow_token,
};

/// Button-local corner radius presets (shadcn/Shadcn `radius` prop).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ButtonRadiusPreset {
    None,
    Small,
    Medium,
    Large,
    Full,
}

impl ButtonRadiusPreset {
    pub const ALL: [Self; 5] = [Self::None, Self::Small, Self::Medium, Self::Large, Self::Full];

    pub fn label(self) -> &'static str {
        match self {
            Self::None => "No radius",
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
            Self::Full => "Full",
        }
    }
}

pub fn resolve_button_radius_preset(
    preset: ButtonRadiusPreset,
    metrics: &gpui_luma::theme::MetricTokens,
    height: f32,
) -> f32 {
    match preset {
        ButtonRadiusPreset::None => metrics.radius.none,
        ButtonRadiusPreset::Small => metrics.radius.sm,
        ButtonRadiusPreset::Medium => metrics.radius.md,
        ButtonRadiusPreset::Large => metrics.radius.lg,
        ButtonRadiusPreset::Full => height / 2.0,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShadcnButtonStyle {
    Primary,
    Secondary,
    Outline,
    Ghost,
    ContentOnly,
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
    if let ButtonFamilyRole::Toggle { selected } = role {
        return super::toggle::toggle_look(mode, theme_mode, style, selected, size, state);
    }
    button_look_semantic(mode, theme_mode, style, role, size, None, state)
}

pub fn button_look_semantic(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    size: ControlSize,
    radius: Option<ButtonRadiusPreset>,
    state: InteractionState,
) -> ButtonFamilyLook {
    if let ButtonFamilyRole::Toggle { selected } = role {
        return super::toggle::toggle_look_semantic(mode, theme_mode, style, selected, size, radius, state);
    }

    let ctx = LookContext::new(mode, theme_mode, state);
    let stylesheet = embedded_stylesheet();
    let palette = button_palette(&ctx, stylesheet, style, role, size);
    let scale = button_box_scale(&ctx, stylesheet, size, 1.0);
    let effective_style = effective_button_style(style, role);
    let mut look = compose_button_family_look(&palette, role, &scale, ctx.metrics().radius.pill);
    if let Some(rule) = stylesheet.button.metrics_for_size(size) {
        look.icon_size = resolve_button_metrics_rule(rule, ctx.metrics(), size).icon_size;
    }
    if let Some(radius) = radius {
        look.radius = resolve_button_radius_preset(radius, ctx.metrics(), look.height);
    }
    look.shadow = button_elevation_shadow(&ctx, stylesheet, effective_style);
    look
}

pub(crate) fn toggle_elevation_shadow(
    catalog: &crate::catalog::CssTokenMap,
    stylesheet: &StylesheetConfig,
    layer: InteractionLayer,
) -> Option<Vec<gpui::BoxShadow>> {
    let token = resolve_layered_elevation_shadow(&stylesheet.toggle.elevation_rules, layer)?;
    let shadows = parse_shadow_token(catalog, &token).ok()?;
    if shadows.is_empty() { None } else { Some(shadows) }
}

pub(crate) fn button_elevation_shadow(
    ctx: &LookContext,
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
) -> Option<Vec<gpui::BoxShadow>> {
    let rule = find_button_elevation_rule(stylesheet, style)?;
    let token = resolve_stylesheet_shadow_token(&rule.shadow)?;
    let shadows = parse_shadow_token(ctx.catalog(), &token).ok()?;
    if shadows.is_empty() { None } else { Some(shadows) }
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
        icon_size: snap_to_pixel(metrics.icon_size, scale_factor),
    }
}

/// Final button/toggle colors, including role normalization and focus border.
pub fn resolve_button_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    state: InteractionState,
) -> ButtonColorPalette {
    resolve_button_palette_with_stylesheet(
        &LookContext::new(mode, theme_mode, state),
        embedded_stylesheet(),
        style,
        role,
    )
}

pub(crate) fn resolve_button_palette_with_stylesheet(
    ctx: &LookContext,
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
) -> ButtonColorPalette {
    let style = effective_button_style(style, role);
    let layer = ctx.state.layer();
    let theme_mode = ctx.theme_mode;
    let selected =
        matches!(role, ButtonFamilyRole::Toggle { selected: true }) && style != ShadcnButtonStyle::ContentOnly;

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "button_resolver");
    let mut colors = resolve_button_colors_with_stylesheet(&resolver, stylesheet, style, layer, theme_mode, selected)
        .unwrap_or_else(|_| ButtonColorPalette::fallback());
    if (matches!(role, ButtonFamilyRole::Toggle { .. }) || style != ShadcnButtonStyle::ContentOnly)
        && ctx.state.focused
        && !ctx.state.disabled
    {
        if let Ok(focus_border) = resolver.resolve_decl("ring") {
            colors.border = Some(focus_border);
        }
    }

    colors
}

pub fn button_palette(
    ctx: &LookContext,
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    size: ControlSize,
) -> ButtonFamilyPalette {
    let colors = resolve_button_palette_with_stylesheet(ctx, stylesheet, style, role);

    let background = colors.background.hsla();
    let foreground = colors.foreground.hsla();
    let border = colors.border.map(|color| color.hsla());

    let typography =
        crate::tables::typography::resolve_control_typography_with_stylesheet(ctx.tokens, stylesheet, size, false);

    ButtonFamilyPalette {
        background,
        foreground,
        muted_foreground: ctx.palette().app_muted_foreground,
        border,
        typography: typography.style,
        font_family: typography.font_family,
    }
}

fn effective_button_style(style: ShadcnButtonStyle, role: ButtonFamilyRole) -> ShadcnButtonStyle {
    effective_button_style_for_role(style, role)
}

pub(crate) fn effective_button_style_for_role(style: ShadcnButtonStyle, role: ButtonFamilyRole) -> ShadcnButtonStyle {
    if matches!(role, ButtonFamilyRole::Toggle { selected: false }) {
        match style {
            ShadcnButtonStyle::Ghost | ShadcnButtonStyle::Outline | ShadcnButtonStyle::ContentOnly => style,
            _ => ShadcnButtonStyle::Outline,
        }
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
    use crate::controls::toggle::toggle_look;
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
    fn focused_button_uses_ring_border() {
        let catalog = retro_arcade_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let focused = button_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Ghost,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            InteractionState { focused: true, ..InteractionState::default() },
        );

        assert_eq!(focused.border, Some(catalog.color("ring").expect("ring")));
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
    fn button_look_resolves_radius_preset() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let none = button_look_semantic(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            Some(ButtonRadiusPreset::None),
            InteractionState::default(),
        );
        let full = button_look_semantic(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            Some(ButtonRadiusPreset::Full),
            InteractionState::default(),
        );

        assert!((none.radius - 0.0).abs() < f32::EPSILON);
        assert!((full.radius - full.height / 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn button_look_resolves_stylesheet_icon_size() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let sm = button_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            ControlSize::Sm,
            InteractionState::default(),
        );
        let lg = button_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            ControlSize::Lg,
            InteractionState::default(),
        );

        assert!((sm.icon_size - 14.0).abs() < f32::EPSILON);
        assert!((lg.icon_size - 18.0).abs() < f32::EPSILON);
    }

    #[test]
    fn lg_button_look_is_larger_than_sm() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let sm = button_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            ControlSize::Sm,
            InteractionState::default(),
        );
        let lg = button_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            ControlSize::Lg,
            InteractionState::default(),
        );

        assert!(lg.height > sm.height);
        assert!(lg.icon_size > sm.icon_size);
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

    #[test]
    fn outline_button_look_resolves_stylesheet_shadow() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let look = button_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Outline,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            InteractionState::default(),
        );

        assert!(look.shadow.as_ref().is_some_and(|shadows| !shadows.is_empty()));
    }

    #[test]
    fn primary_button_look_has_no_shadow() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let look = button_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            InteractionState::default(),
        );

        assert!(look.shadow.is_none());
    }

    #[test]
    fn ghost_button_look_has_no_shadow() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let look = button_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Ghost,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            InteractionState::default(),
        );

        assert!(look.shadow.is_none());
    }

    #[test]
    fn unselected_ghost_toggle_has_no_border_like_ghost_icon_button() {
        let catalog = retro_arcade_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog, ThemeMode::Light).expect("catalog");
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
            false,
            ControlSize::Md,
            InteractionState::default(),
        );

        assert_eq!(toggle.border, icon.border);
        assert!(
            gpui_luma::controls::button_family::button_family_effective_border(toggle.border).a <= 0.0,
            "ghost toggle off-state should be borderless"
        );
    }

    #[test]
    fn unselected_primary_toggle_still_uses_outline_border() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let toggle = toggle_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            false,
            ControlSize::Md,
            InteractionState::default(),
        );

        assert!(
            gpui_luma::controls::button_family::button_family_effective_border(toggle.border).a > 0.0,
            "primary toggle off-state should keep outline border"
        );
    }

    #[test]
    fn primary_toggle_still_uses_toggle_elevation_shadow() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let look = toggle_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            true,
            ControlSize::Md,
            InteractionState::default(),
        );
        assert!(
            look.shadow.as_ref().is_some_and(|shadows| !shadows.is_empty()),
            "primary toggle should keep toggle elevation"
        );
    }

    #[test]
    fn toggle_look_resolves_stylesheet_shadow_for_selected_and_unselected() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        for selected in [false, true] {
            let look = button_look(
                &mode,
                ThemeMode::Light,
                ShadcnButtonStyle::Primary,
                ButtonFamilyRole::Toggle { selected },
                ControlSize::Md,
                InteractionState::default(),
            );
            assert!(
                look.shadow.as_ref().is_some_and(|shadows| !shadows.is_empty()),
                "toggle selected={selected} should resolve shadow-sm"
            );
        }
    }

    #[test]
    fn disabled_toggle_look_has_no_shadow() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let look = button_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Toggle { selected: true },
            ControlSize::Md,
            InteractionState { disabled: true, ..InteractionState::default() },
        );

        assert!(look.shadow.is_none());
    }

    #[test]
    fn content_only_button_has_no_state_chrome() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");

        for state in [
            InteractionState::default(),
            InteractionState { hovered: true, ..InteractionState::default() },
            InteractionState { pressed: true, ..InteractionState::default() },
            InteractionState { focused: true, ..InteractionState::default() },
            InteractionState { disabled: true, ..InteractionState::default() },
        ] {
            let look = button_look(
                &mode,
                ThemeMode::Light,
                ShadcnButtonStyle::ContentOnly,
                ButtonFamilyRole::Text,
                ControlSize::Md,
                state,
            );

            assert_eq!(look.background.a, 0.0);
            assert!(look.border.is_none_or(|border| border.a == 0.0));
            assert!(look.shadow.is_none());
        }
    }
}
