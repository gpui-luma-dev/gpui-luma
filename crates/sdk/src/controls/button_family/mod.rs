mod theme;

pub use theme::{
    BUTTON_THEME_USAGE, ICON_BUTTON_THEME_USAGE, TOGGLE_BUTTON_THEME_USAGE, TOGGLE_THEME_USAGE, ButtonFamilyAppearance,
    ButtonFamilyRole, ButtonFamilyTheme, ButtonVariant, DefaultButtonFamilyTheme, default_button_family_theme,
};

use crate::theme::{ControlSize, InteractionState};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonKind {
    #[default]
    Standard,
    Subtle,
    Ghost,
    Prominent,
}

pub fn button_variant(kind: ButtonKind) -> ButtonVariant {
    match kind {
        ButtonKind::Standard => ButtonVariant::Standard,
        ButtonKind::Subtle => ButtonVariant::Subtle,
        ButtonKind::Ghost => ButtonVariant::Ghost,
        ButtonKind::Prominent => ButtonVariant::Prominent,
    }
}

pub type ButtonSize = ControlSize;
pub type ButtonInteractionState = InteractionState;
