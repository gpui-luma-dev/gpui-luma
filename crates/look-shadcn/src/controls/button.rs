use gpui::Hsla;

use gpui_luma::controls::button_family::{
    ButtonFamilyAppearance, ButtonFamilyPalette, ButtonFamilyRole, compose_button_family_appearance,
};
use gpui_luma::theme::adorner::{AdornerPlacement, AdornerSpec, FocusRingAdornerSpec};
use gpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, StandardBoxScale, ThemeMode};

use crate::appearance_context::AppearanceContext;
use crate::mode::ShadcnModeTokens;
use crate::palette::ShadcnActionRole;

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
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    size: ControlSize,
    state: InteractionState,
) -> ButtonFamilyAppearance {
    let ctx = AppearanceContext::new(mode, ThemeMode::Light, state);
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

    let foreground = match (style, selected, state.disabled) {
        (_, _, true) => palette.disabled_foreground,
        (ShadcnButtonStyle::Secondary, true, false) => action.foreground,
        (_, true, false) => palette.selected_foreground,
        (ShadcnButtonStyle::Secondary, false, false) => action.foreground,
        (ShadcnButtonStyle::Outline, false, false) => action.foreground,
        (ShadcnButtonStyle::Ghost, false, false) => action.foreground,
        (ShadcnButtonStyle::Primary, false, false) => action.foreground,
    };

    let background = button_background(style, action, palette, selected, state.layer());
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

fn button_background(
    style: ShadcnButtonStyle,
    action: ShadcnActionRole,
    palette: &crate::palette::ShadcnPalette,
    selected: bool,
    layer: InteractionLayer,
) -> Hsla {
    match (style, selected, layer) {
        (_, _, InteractionLayer::Disabled) => palette.disabled_background,
        (ShadcnButtonStyle::Secondary, true, InteractionLayer::Pressed) => action.pressed_background,
        (ShadcnButtonStyle::Secondary, true, InteractionLayer::Hovered) => action.hover_background,
        (ShadcnButtonStyle::Secondary, true, InteractionLayer::Default) => action.background,
        (_, true, InteractionLayer::Pressed) => palette.primary.pressed_background,
        (_, true, InteractionLayer::Hovered) => palette.primary.hover_background,
        (_, true, InteractionLayer::Default) => palette.selected_background,
        (ShadcnButtonStyle::Primary, _, InteractionLayer::Pressed) => action.pressed_background,
        (ShadcnButtonStyle::Primary, _, InteractionLayer::Hovered) => action.hover_background,
        (ShadcnButtonStyle::Primary, _, InteractionLayer::Default) => action.background,
        (ShadcnButtonStyle::Outline, _, InteractionLayer::Pressed) => action.pressed_background,
        (ShadcnButtonStyle::Outline, _, InteractionLayer::Hovered) => action.hover_background,
        (ShadcnButtonStyle::Outline, _, InteractionLayer::Default) => action.background,
        (ShadcnButtonStyle::Secondary, _, InteractionLayer::Pressed) => action.pressed_background,
        (ShadcnButtonStyle::Secondary, _, InteractionLayer::Hovered) => action.hover_background,
        (ShadcnButtonStyle::Secondary, _, InteractionLayer::Default) => action.background,
        (ShadcnButtonStyle::Ghost, _, InteractionLayer::Pressed) => action.pressed_background,
        (ShadcnButtonStyle::Ghost, _, InteractionLayer::Hovered) => action.hover_background,
        (ShadcnButtonStyle::Ghost, _, InteractionLayer::Default) => action.background,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_luma::controls::button_family::ButtonFamilyRole;
    use crate::mode::ShadcnModeTokens;
    use gpui_luma::theme::{LumaTheme, ThemeMode};

    #[test]
    fn primary_hover_background_differs_from_default() {
        let theme = LumaTheme::native();
        let mode = ShadcnModeTokens::from_luma_tokens(theme.mode(ThemeMode::Light));
        let default = button_appearance(
            &mode,
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            InteractionState::default(),
        );
        let hovered = button_appearance(
            &mode,
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            InteractionState { hovered: true, ..InteractionState::default() },
        );

        assert_ne!(default.background, hovered.background);
    }
}
