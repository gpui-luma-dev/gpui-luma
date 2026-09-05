//! Radix-owned button recipes bound to SDK [`ButtonFamilyTheme`].

use std::sync::Arc;

use gpui::{Hsla, SharedString};
use luma::controls::button_family::{
    ButtonFamilyLook, ButtonFamilyPalette, ButtonFamilyRole, ButtonFamilyTheme, compose_button_family_look,
};
use luma::theme::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, MetricTokens, StandardBoxScale};
use gpui::FontWeight;

use crate::look::RadixLook;
use crate::semantic::SemanticRole;

/// Look-owned button treatments. Not Shadcn variant names and not SDK enums.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RadixButtonRecipe {
    #[default]
    Solid,
    Soft,
    Outline,
    Ghost,
}

struct RadixButtonFamilyTheme {
    look: RadixLook,
    recipe: RadixButtonRecipe,
}

impl ButtonFamilyTheme for RadixButtonFamilyTheme {
    fn resolve(&self, role: ButtonFamilyRole, _size: ControlSize, state: InteractionState) -> ButtonFamilyPalette {
        resolve_button_palette(&self.look, self.recipe, role, state)
    }

    fn metrics(&self) -> MetricTokens {
        self.look.metrics()
    }

    fn resolve_look(
        &self,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
        scale: &StandardBoxScale,
        pill_radius: f32,
    ) -> Option<ButtonFamilyLook> {
        let mut palette = self.resolve(role, size, state);
        let metrics = self.look.metrics();
        let control = metrics.for_size(size);
        // Prefer look metrics for typography size when the scale path is used.
        palette.typography = LumaTextStyle {
            size: match size {
                ControlSize::Sm => 12.5,
                ControlSize::Md => 14.0,
                ControlSize::Lg => 16.0,
            },
            line_height: match size {
                ControlSize::Sm => 18.0,
                ControlSize::Md => 20.0,
                ControlSize::Lg => 22.0,
            },
            weight: FontWeight::MEDIUM,
        };
        let _ = control;
        Some(compose_button_family_look(&palette, role, scale, pill_radius))
    }
}

pub fn button_family_theme(look: Arc<RadixLook>, recipe: RadixButtonRecipe) -> Arc<dyn ButtonFamilyTheme> {
    Arc::new(RadixButtonFamilyTheme { look: look.as_ref().clone(), recipe })
}

fn resolve_button_palette(
    look: &RadixLook,
    recipe: RadixButtonRecipe,
    role: ButtonFamilyRole,
    state: InteractionState,
) -> ButtonFamilyPalette {
    let layer = state.layer();
    let selected = matches!(role, ButtonFamilyRole::Toggle { selected: true });

    let (bg_role, fg_role, border_role) = if selected {
        (SemanticRole::Primary, SemanticRole::PrimaryForeground, SemanticRole::Primary)
    } else {
        match recipe {
            RadixButtonRecipe::Solid => (SemanticRole::Primary, SemanticRole::PrimaryForeground, SemanticRole::Primary),
            RadixButtonRecipe::Soft => (SemanticRole::Soft, SemanticRole::SoftForeground, SemanticRole::Soft),
            RadixButtonRecipe::Outline => (SemanticRole::Background, SemanticRole::Foreground, SemanticRole::Border),
            RadixButtonRecipe::Ghost => (SemanticRole::Background, SemanticRole::Foreground, SemanticRole::Background),
        }
    };

    let mut background = look.resolve_role(bg_role).hsla();
    let mut foreground = look.resolve_role(fg_role).hsla();
    let mut border = match recipe {
        RadixButtonRecipe::Ghost if !selected => None,
        RadixButtonRecipe::Outline => Some(look.resolve_role(border_role).hsla()),
        _ => Some(look.resolve_role(border_role).hsla()),
    };

    if matches!(recipe, RadixButtonRecipe::Ghost | RadixButtonRecipe::Outline) && !selected {
        background = transparent();
        if matches!(recipe, RadixButtonRecipe::Ghost) {
            border = None;
        }
    }

    match layer {
        InteractionLayer::Disabled => {
            background = look.resolve_role(SemanticRole::Surface).hsla();
            foreground = look.resolve_role(SemanticRole::MutedForeground).hsla();
            border = Some(look.resolve_role(SemanticRole::Border).hsla());
        }
        InteractionLayer::Pressed => {
            background = shift_for_press(look, recipe, selected, background);
        }
        InteractionLayer::Hovered => {
            background = shift_for_hover(look, recipe, selected, background);
        }
        InteractionLayer::Default => {}
    }

    if state.focused && !state.disabled {
        border = Some(look.resolve_role(SemanticRole::Focus).hsla());
    }

    ButtonFamilyPalette {
        background,
        foreground,
        border,
        typography: LumaTextStyle { size: 14.0, line_height: 20.0, weight: FontWeight::MEDIUM },
        font_family: SharedString::from("System UI"),
    }
}

fn shift_for_hover(look: &RadixLook, recipe: RadixButtonRecipe, selected: bool, base: Hsla) -> Hsla {
    if selected || matches!(recipe, RadixButtonRecipe::Solid) {
        return look.resolve_step(crate::scale::ScaleFamily::Color, 10).hsla();
    }
    if matches!(recipe, RadixButtonRecipe::Soft | RadixButtonRecipe::Ghost | RadixButtonRecipe::Outline) {
        return look.resolve_role(SemanticRole::Soft).hsla();
    }
    base
}

fn shift_for_press(look: &RadixLook, recipe: RadixButtonRecipe, selected: bool, base: Hsla) -> Hsla {
    if selected || matches!(recipe, RadixButtonRecipe::Solid) {
        return look.resolve_step(crate::scale::ScaleFamily::Color, 11).hsla();
    }
    if matches!(recipe, RadixButtonRecipe::Soft | RadixButtonRecipe::Ghost | RadixButtonRecipe::Outline) {
        return look.resolve_step(crate::scale::ScaleFamily::Color, 4).hsla();
    }
    base
}

fn transparent() -> Hsla {
    gpui::hsla(0.0, 0.0, 0.0, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use luma::theme::InteractionState;

    #[test]
    fn solid_default_uses_primary_steps() {
        let look = Arc::new(RadixLook::built_in());
        let theme = button_family_theme(look.clone(), RadixButtonRecipe::Solid);
        let palette = theme.resolve(ButtonFamilyRole::Text, ControlSize::Md, InteractionState::default());
        let expected_bg = look.resolve_role(SemanticRole::Primary).hsla();
        assert_eq!(palette.background.h, expected_bg.h);
        assert_eq!(palette.background.l, expected_bg.l);
    }

    #[test]
    fn ghost_default_is_transparent() {
        let look = Arc::new(RadixLook::built_in());
        let theme = button_family_theme(look, RadixButtonRecipe::Ghost);
        let palette = theme.resolve(ButtonFamilyRole::Text, ControlSize::Md, InteractionState::default());
        assert_eq!(palette.background.a, 0.0);
        assert!(palette.border.is_none());
    }
}
