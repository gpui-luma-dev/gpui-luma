use std::sync::{Arc, OnceLock};

use gpui::{Hsla, SharedString};

use crate::theme::adorner::AdornerSpec;

use crate::theme::radix::{RadixModeTokens, button_appearance};
use crate::theme::{ControlSize, InteractionState, LumaTextStyle, ThemePartUsage, ThemeTokens, ThemeUsage};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonVariant {
    #[default]
    Standard,
    Subtle,
    Ghost,
    Prominent,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonFamilyRole {
    #[default]
    Text,
    Icon,
    Toggle {
        selected: bool,
    },
}

#[derive(Clone, Debug)]
pub struct ButtonFamilyAppearance {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub adorner: Option<AdornerSpec>,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub height: f32,
}

pub trait ButtonFamilyTheme: Send + Sync {
    fn resolve(
        &self,
        variant: ButtonVariant,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
    ) -> ButtonFamilyAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultButtonFamilyTheme {
    tokens: ThemeTokens,
}

pub fn default_button_family_theme() -> Arc<dyn ButtonFamilyTheme> {
    if let Some(radix) = crate::theme::radix::active_radix_theme() {
        return radix.button_family_theme();
    }

    if let Some(live) = crate::theme::pack::active_live_theme() {
        return live;
    }
    static THEME: OnceLock<Arc<dyn ButtonFamilyTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultButtonFamilyTheme::default())).clone()
}

pub const BUTTON_THEME_USAGE: ThemeUsage = ThemeUsage {
    label: "Button",
    parts: &[
        ThemePartUsage {
            part: "standard background",
            token: "action.standard.background",
            states: &["default"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "standard hover background",
            token: "action.standard.hover_background",
            states: &["hovered"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "standard pressed background",
            token: "action.standard.pressed_background",
            states: &["pressed"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "standard foreground",
            token: "action.standard.foreground",
            states: &["default", "hovered", "pressed", "focused"],
            appearance_fields: &["ButtonFamilyAppearance.foreground"],
        },
        ThemePartUsage {
            part: "standard border",
            token: "action.standard.border",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            appearance_fields: &["ButtonFamilyAppearance.border"],
        },
        ThemePartUsage {
            part: "prominent background",
            token: "action.prominent.background",
            states: &["default"],
            appearance_fields: &["ButtonFamilyAppearance.background", "ButtonFamilyAppearance.border"],
        },
        ThemePartUsage {
            part: "prominent hover background",
            token: "action.prominent.hover_background",
            states: &["hovered"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "prominent pressed background",
            token: "action.prominent.pressed_background",
            states: &["pressed"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "prominent foreground",
            token: "action.prominent.foreground",
            states: &["default", "hovered", "pressed", "focused"],
            appearance_fields: &["ButtonFamilyAppearance.foreground"],
        },
        ThemePartUsage {
            part: "subtle background",
            token: "action.subtle.background",
            states: &["default"],
            appearance_fields: &["ButtonFamilyAppearance.background", "ButtonFamilyAppearance.border"],
        },
        ThemePartUsage {
            part: "subtle hover background",
            token: "action.subtle.hover_background",
            states: &["hovered"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "subtle pressed background",
            token: "action.subtle.pressed_background",
            states: &["pressed"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "subtle foreground",
            token: "action.subtle.foreground",
            states: &["default", "hovered", "pressed", "focused"],
            appearance_fields: &["ButtonFamilyAppearance.foreground"],
        },
        ThemePartUsage {
            part: "disabled background",
            token: "state.disabled.background",
            states: &["disabled"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "disabled foreground",
            token: "state.disabled.foreground",
            states: &["disabled"],
            appearance_fields: &["ButtonFamilyAppearance.foreground"],
        },
        ThemePartUsage {
            part: "focus ring",
            token: "focus.ring",
            states: &["focused"],
            appearance_fields: &["ButtonFamilyAppearance.adorner"],
        },
    ],
};

pub const ICON_BUTTON_THEME_USAGE: ThemeUsage = ThemeUsage { label: "Icon Button", parts: BUTTON_THEME_USAGE.parts };

pub const TOGGLE_THEME_USAGE: ThemeUsage = ThemeUsage {
    label: "Toggle",
    parts: &[
        ThemePartUsage {
            part: "unselected outline background",
            token: "action.subtle.background",
            states: &["default"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "unselected outline hover background",
            token: "action.subtle.hover_background",
            states: &["hovered"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "unselected outline pressed background",
            token: "action.subtle.pressed_background",
            states: &["pressed"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "unselected outline foreground",
            token: "action.subtle.foreground",
            states: &["default", "hovered", "pressed", "focused"],
            appearance_fields: &["ButtonFamilyAppearance.foreground"],
        },
        ThemePartUsage {
            part: "unselected outline border",
            token: "action.subtle.border",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            appearance_fields: &["ButtonFamilyAppearance.border"],
        },
        ThemePartUsage {
            part: "selected background",
            token: "state.selected.background",
            states: &["selected"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "selected hover background",
            token: "action.prominent.hover_background",
            states: &["selected hovered"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "selected pressed background",
            token: "action.prominent.pressed_background",
            states: &["selected pressed"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "selected foreground",
            token: "state.selected.foreground",
            states: &["selected", "selected hovered", "selected pressed", "selected focused"],
            appearance_fields: &["ButtonFamilyAppearance.foreground"],
        },
        ThemePartUsage {
            part: "prominent background",
            token: "action.prominent.background",
            states: &["default"],
            appearance_fields: &["ButtonFamilyAppearance.background", "ButtonFamilyAppearance.border"],
        },
        ThemePartUsage {
            part: "prominent foreground",
            token: "action.prominent.foreground",
            states: &["default", "hovered", "pressed", "focused"],
            appearance_fields: &["ButtonFamilyAppearance.foreground"],
        },
        ThemePartUsage {
            part: "standard border",
            token: "action.standard.border",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            appearance_fields: &["ButtonFamilyAppearance.border"],
        },
        ThemePartUsage {
            part: "disabled background",
            token: "state.disabled.background",
            states: &["disabled"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "disabled foreground",
            token: "state.disabled.foreground",
            states: &["disabled"],
            appearance_fields: &["ButtonFamilyAppearance.foreground"],
        },
        ThemePartUsage {
            part: "focus ring",
            token: "focus.ring",
            states: &["focused", "selected focused"],
            appearance_fields: &["ButtonFamilyAppearance.adorner"],
        },
    ],
};

pub const TOGGLE_BUTTON_THEME_USAGE: ThemeUsage = TOGGLE_THEME_USAGE;

impl DefaultButtonFamilyTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

fn radix_style_from_variant(variant: ButtonVariant) -> crate::theme::radix::RadixButtonStyle {
    use crate::theme::radix::RadixButtonStyle;

    match variant {
        ButtonVariant::Prominent => RadixButtonStyle::Primary,
        ButtonVariant::Standard => RadixButtonStyle::Secondary,
        ButtonVariant::Subtle => RadixButtonStyle::Outline,
        ButtonVariant::Ghost => RadixButtonStyle::Ghost,
    }
}

impl ButtonFamilyTheme for DefaultButtonFamilyTheme {
    fn resolve(
        &self,
        variant: ButtonVariant,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
    ) -> ButtonFamilyAppearance {
        let mode = RadixModeTokens::from_luma_tokens(&self.tokens);
        button_appearance(&mode, radix_style_from_variant(variant), role, size, state)
    }
}
