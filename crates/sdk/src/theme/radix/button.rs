use gpui::Hsla;

use crate::controls::button_family::{
    ButtonFamilyAppearance, ButtonFamilyPalette, ButtonFamilyRole, compose_button_family_appearance,
};
use crate::theme::adorner::{AdornerPlacement, AdornerSpec, FocusRingAdornerSpec};
use crate::theme::{ControlSize, InteractionLayer, InteractionState, StandardBoxScale};

use super::mode::RadixModeTokens;
use super::palette::RadixActionRole;

/// Radix-style button appearance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RadixButtonStyle {
    Primary,
    Secondary,
    Outline,
    Ghost,
}

pub(crate) fn button_appearance(
    mode: &RadixModeTokens,
    style: RadixButtonStyle,
    role: ButtonFamilyRole,
    size: ControlSize,
    state: InteractionState,
) -> ButtonFamilyAppearance {
    let palette = button_palette(mode, style, role, size, state);
    let scale = StandardBoxScale::compute(size, &mode.metrics, 1.0);
    compose_button_family_appearance(&palette, role, &scale)
}

pub(crate) fn button_palette(
    mode: &RadixModeTokens,
    style: RadixButtonStyle,
    role: ButtonFamilyRole,
    _size: ControlSize,
    state: InteractionState,
) -> ButtonFamilyPalette {
    let style = if matches!(role, ButtonFamilyRole::Toggle { selected: false }) {
        RadixButtonStyle::Outline
    } else {
        style
    };

    let palette = &mode.palette;
    let metrics = &mode.metrics;
    let typography = &mode.typography;
    let selected = matches!(role, ButtonFamilyRole::Toggle { selected: true });
    let action = palette.action(style);

    let foreground = match (style, selected, state.disabled) {
        (_, _, true) => palette.disabled_foreground,
        (RadixButtonStyle::Secondary, true, false) => action.foreground,
        (_, true, false) => palette.selected_foreground,
        (RadixButtonStyle::Secondary, false, false) => action.foreground,
        (RadixButtonStyle::Outline, false, false) => action.foreground,
        (RadixButtonStyle::Ghost, false, false) => action.foreground,
        (RadixButtonStyle::Primary, false, false) => action.foreground,
    };

    let background = button_background(style, action, palette, selected, state.layer());
    let border = action.border;

    let adorner = if state.focused {
        let (placement, distance) = match style {
            RadixButtonStyle::Ghost => (AdornerPlacement::Inset, metrics.border_width.default),
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
    style: RadixButtonStyle,
    action: RadixActionRole,
    palette: &super::palette::RadixPalette,
    selected: bool,
    layer: InteractionLayer,
) -> Hsla {
    match (style, selected, layer) {
        (_, _, InteractionLayer::Disabled) => palette.disabled_background,
        (RadixButtonStyle::Secondary, true, InteractionLayer::Pressed) => action.pressed_background,
        (RadixButtonStyle::Secondary, true, InteractionLayer::Hovered) => action.hover_background,
        (RadixButtonStyle::Secondary, true, InteractionLayer::Default) => action.background,
        (_, true, InteractionLayer::Pressed) => palette.primary.pressed_background,
        (_, true, InteractionLayer::Hovered) => palette.primary.hover_background,
        (_, true, InteractionLayer::Default) => palette.selected_background,
        (RadixButtonStyle::Primary, _, InteractionLayer::Pressed) => action.pressed_background,
        (RadixButtonStyle::Primary, _, InteractionLayer::Hovered) => action.hover_background,
        (RadixButtonStyle::Primary, _, InteractionLayer::Default) => action.background,
        (RadixButtonStyle::Outline, _, InteractionLayer::Pressed) => action.pressed_background,
        (RadixButtonStyle::Outline, _, InteractionLayer::Hovered) => action.hover_background,
        (RadixButtonStyle::Outline, _, InteractionLayer::Default) => action.background,
        (RadixButtonStyle::Secondary, _, InteractionLayer::Pressed) => action.pressed_background,
        (RadixButtonStyle::Secondary, _, InteractionLayer::Hovered) => action.hover_background,
        (RadixButtonStyle::Secondary, _, InteractionLayer::Default) => action.background,
        (RadixButtonStyle::Ghost, _, InteractionLayer::Pressed) => action.pressed_background,
        (RadixButtonStyle::Ghost, _, InteractionLayer::Hovered) => action.hover_background,
        (RadixButtonStyle::Ghost, _, InteractionLayer::Default) => action.background,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::button_family::ButtonFamilyRole;
    use crate::theme::radix::mode::RadixModeTokens;
    use crate::theme::{LumaTheme, ThemeMode};

    #[test]
    fn primary_hover_background_differs_from_default() {
        let theme = LumaTheme::native();
        let mode = RadixModeTokens::from_luma_tokens(theme.mode(ThemeMode::Light));
        let default = button_appearance(
            &mode,
            RadixButtonStyle::Primary,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            InteractionState::default(),
        );
        let hovered = button_appearance(
            &mode,
            RadixButtonStyle::Primary,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            InteractionState { hovered: true, ..InteractionState::default() },
        );

        assert_ne!(default.background, hovered.background);
    }
}
