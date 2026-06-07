use gpui::Hsla;

use gpui_luma::controls::button_family::{
    ButtonFamilyAppearance, ButtonFamilyPalette, ButtonFamilyRole, compose_button_family_appearance,
};
use gpui_luma::theme::adorner::{AdornerPlacement, AdornerSpec, FocusRingAdornerSpec};
use gpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, StandardBoxScale, ThemeMode};

use crate::appearance_context::AppearanceContext;
use crate::mode::ShadcnModeTokens;
use crate::tokens::ShadcnToken;

/// Radix-style button appearance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShadcnButtonStyle {
    Primary,
    Secondary,
    Outline,
    Ghost,
}

pub(crate) fn button_appearance(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    size: ControlSize,
    state: InteractionState,
) -> ButtonFamilyAppearance {
    let ctx = AppearanceContext::new(mode, theme_mode, state);
    let palette = button_palette(&ctx, style, role, size);
    let scale = StandardBoxScale::compute(size, ctx.metrics(), 1.0);
    compose_button_family_appearance(&palette, role, &scale, ctx.metrics().radius.pill)
}

pub(crate) fn button_palette(
    ctx: &AppearanceContext,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    _size: ControlSize,
) -> ButtonFamilyPalette {
    let state = ctx.state;
    let style = if matches!(role, ButtonFamilyRole::Toggle { selected: false }) {
        ShadcnButtonStyle::Outline
    } else {
        style
    };

    let palette = ctx.palette();
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let selected = matches!(role, ButtonFamilyRole::Toggle { selected: true });
    let action = palette.action(style);
    let layer = state.layer();

    let foreground = button_foreground(ctx, style, action.foreground, selected, layer);
    let background = button_background(ctx, style, selected, layer);
    let border = action.border;

    let adorner = if state.focused {
        let (placement, distance) = match style {
            ShadcnButtonStyle::Ghost => (AdornerPlacement::Inset, metrics.border_width.default),
            _ => (AdornerPlacement::Oversize, metrics.border_width.default + metrics.focus.width),
        };

        Some(AdornerSpec::FocusRing(FocusRingAdornerSpec {
            color: palette.focus_ring,
            placement,
            distance,
            width: metrics.focus.width,
        }))
    } else {
        None
    };

    ButtonFamilyPalette {
        background,
        foreground,
        border,
        adorner,
        typography: typography.text.label,
        font_family: typography.font.sans.family.clone().into(),
    }
}

fn button_foreground(
    ctx: &AppearanceContext,
    style: ShadcnButtonStyle,
    action_foreground: Hsla,
    selected: bool,
    layer: InteractionLayer,
) -> Hsla {
    let palette = ctx.palette();
    let state = ctx.state;

    if state.disabled {
        return palette.disabled_foreground;
    }

    if uses_accent_foreground_on_hover(ctx, style, layer) {
        return ctx.palette().accent_foreground;
    }

    match (style, selected) {
        (ShadcnButtonStyle::Secondary, true) => action_foreground,
        (_, true) => palette.selected_foreground,
        _ => action_foreground,
    }
}

/// Outline and ghost use accent foreground on hover/press (shadcn `hover:text-accent-foreground`).
fn uses_accent_foreground_on_hover(
    _ctx: &AppearanceContext,
    style: ShadcnButtonStyle,
    layer: InteractionLayer,
) -> bool {
    matches!(style, ShadcnButtonStyle::Outline | ShadcnButtonStyle::Ghost)
        && matches!(layer, InteractionLayer::Hovered | InteractionLayer::Pressed)
}

fn button_background(
    ctx: &AppearanceContext,
    style: ShadcnButtonStyle,
    selected: bool,
    layer: InteractionLayer,
) -> Hsla {
    let palette = ctx.palette();
    let action = palette.action(style);
    match (style, selected, layer) {
        (_, _, InteractionLayer::Disabled) => palette.disabled_background,
        (ShadcnButtonStyle::Secondary, true, InteractionLayer::Pressed) => action.pressed_background,
        (ShadcnButtonStyle::Secondary, true, InteractionLayer::Hovered) => action.hover_background,
        (ShadcnButtonStyle::Secondary, true, InteractionLayer::Default) => action.background,
        (_, true, InteractionLayer::Pressed) => {
            ctx.resolve_color_state(ShadcnToken::Primary, InteractionLayer::Pressed)
        }
        (_, true, InteractionLayer::Hovered) => {
            ctx.resolve_color_state(ShadcnToken::Primary, InteractionLayer::Hovered)
        }
        (_, true, InteractionLayer::Default) => palette.selected_background,
        (ShadcnButtonStyle::Primary, _, layer) => ctx.resolve_color_state(ShadcnToken::Primary, layer),
        (ShadcnButtonStyle::Secondary, _, layer) => ctx.resolve_color_state(ShadcnToken::Secondary, layer),
        (ShadcnButtonStyle::Outline | ShadcnButtonStyle::Ghost, _, InteractionLayer::Default) => action.background,
        (ShadcnButtonStyle::Outline | ShadcnButtonStyle::Ghost, _, InteractionLayer::Hovered) => {
            action.hover_background
        }
        (ShadcnButtonStyle::Outline | ShadcnButtonStyle::Ghost, _, InteractionLayer::Pressed) => {
            action.pressed_background
        }
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
        ]))
    }

    #[test]
    fn primary_hover_background_differs_from_default() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let default = button_appearance(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            InteractionState::default(),
        );
        let hovered = button_appearance(
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
        let hovered = button_appearance(
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
        let hovered = button_appearance(
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
        let hovered = button_appearance(
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
        let hovered = button_appearance(
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
        assert_eq!(hovered.border, input);
    }

    #[test]
    fn retro_arcade_primary_hover_is_subtle() {
        let catalog = retro_arcade_catalog();
        let light = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("light");
        let dark = ShadcnModeTokens::from_catalog(catalog, ThemeMode::Dark).expect("dark");

        for (mode, theme_mode) in [(&light, ThemeMode::Light), (&dark, ThemeMode::Dark)] {
            let default = button_appearance(
                mode,
                theme_mode,
                ShadcnButtonStyle::Primary,
                ButtonFamilyRole::Text,
                ControlSize::Md,
                InteractionState::default(),
            );
            let hovered = button_appearance(
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
}
