use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::adorner::{AdornerPlacement, AdornerSpec, FocusRingAdornerSpec};
use crate::theme::{
    ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemePartUsage, ThemeTokens, ThemeUsage,
};

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
            part: "unselected standard background",
            token: "action.standard.background",
            states: &["default"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "unselected standard hover background",
            token: "action.standard.hover_background",
            states: &["hovered"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "unselected standard pressed background",
            token: "action.standard.pressed_background",
            states: &["pressed"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "unselected standard foreground",
            token: "action.standard.foreground",
            states: &["default", "hovered", "pressed", "focused"],
            appearance_fields: &["ButtonFamilyAppearance.foreground"],
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

impl ButtonFamilyTheme for DefaultButtonFamilyTheme {
    fn resolve(
        &self,
        variant: ButtonVariant,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
    ) -> ButtonFamilyAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let selected = matches!(role, ButtonFamilyRole::Toggle { selected: true });
        let transparent = Hsla { h: 0.0, s: 0.0, l: 0.0, a: 0.0 };

        let foreground = match (variant, selected, state.disabled) {
            (_, _, true) => palette.state.disabled.foreground,
            (_, true, false) => palette.state.selected.foreground,
            (ButtonVariant::Standard, false, false) => palette.action.standard.foreground,
            (ButtonVariant::Subtle, false, false) => palette.action.subtle.foreground,
            (ButtonVariant::Ghost, false, false) => palette.action.ghost.foreground,
            (ButtonVariant::Prominent, false, false) => palette.action.prominent.foreground,
        };

        let background = match (variant, selected, state.layer()) {
            (_, _, InteractionLayer::Disabled) => palette.state.disabled.background,
            (_, true, InteractionLayer::Pressed) => palette.action.prominent.pressed_background,
            (_, true, InteractionLayer::Hovered) => palette.action.prominent.hover_background,
            (_, true, InteractionLayer::Default) => palette.state.selected.background,
            (ButtonVariant::Prominent, _, InteractionLayer::Pressed) => palette.action.prominent.pressed_background,
            (ButtonVariant::Prominent, _, InteractionLayer::Hovered) => palette.action.prominent.hover_background,
            (ButtonVariant::Prominent, _, InteractionLayer::Default) => palette.action.prominent.background,
            (ButtonVariant::Subtle, _, InteractionLayer::Pressed) => palette.action.subtle.pressed_background,
            (ButtonVariant::Subtle, _, InteractionLayer::Hovered) => palette.action.subtle.hover_background,
            (ButtonVariant::Subtle, _, InteractionLayer::Default) => palette.action.subtle.background,
            (ButtonVariant::Standard, _, InteractionLayer::Pressed) => palette.action.standard.pressed_background,
            (ButtonVariant::Standard, _, InteractionLayer::Hovered) => palette.action.standard.hover_background,
            (ButtonVariant::Standard, _, InteractionLayer::Default) => palette.action.standard.background,
            (ButtonVariant::Ghost, _, InteractionLayer::Pressed) => palette.state.pressed.background,
            (ButtonVariant::Ghost, _, InteractionLayer::Hovered) => palette.state.hover.background,
            (ButtonVariant::Ghost, _, InteractionLayer::Default) => transparent,
        };

        let height = metrics.control_height(size);
        let border = match variant {
            ButtonVariant::Standard => palette.action.standard.border,
            ButtonVariant::Subtle => palette.action.subtle.border,
            ButtonVariant::Ghost => palette.action.ghost.border,
            ButtonVariant::Prominent => palette.action.prominent.border,
        };

        let adorner = if state.focused {
            let (placement, distance) = match variant {
                ButtonVariant::Ghost => (AdornerPlacement::Inset, metrics.border_width.default),
                _ => (AdornerPlacement::Oversize, metrics.border_width.default + metrics.focus.width),
            };

            Some(AdornerSpec::FocusRing(FocusRingAdornerSpec {
                color: palette.focus.ring,
                placement,
                distance,
                width: metrics.focus.width,
            }))
        } else {
            None
        };

        ButtonFamilyAppearance {
            background,
            foreground,
            border,
            adorner,
            typography: typography.text.label,
            radius: match role {
                ButtonFamilyRole::Icon => metrics.radius.pill,
                _ => metrics.radius(size),
            },
            padding_x: match role {
                ButtonFamilyRole::Icon => 0.0,
                _ => metrics.padding_x(size),
            },
            padding_y: match role {
                ButtonFamilyRole::Icon => 0.0,
                _ => metrics.padding_y(size),
            },
            gap: metrics.gap(size),
            height,
        }
    }
}
