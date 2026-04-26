use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use super::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemePartUsage, ThemeTokens, ThemeUsage};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonVariant {
    #[default]
    Default,
    Ghost,
    Primary,
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

#[derive(Clone, Copy, Debug)]
pub struct ButtonFamilyAppearance {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub focus_ring: Option<Hsla>,
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
    static THEME: OnceLock<Arc<dyn ButtonFamilyTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultButtonFamilyTheme::default())).clone()
}

pub const BUTTON_THEME_USAGE: ThemeUsage = ThemeUsage {
    component: "Button",
    parts: &[
        ThemePartUsage {
            part: "default background",
            token: "action.ghost.background",
            states: &["default"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "default hover background",
            token: "action.ghost.hover_background",
            states: &["hovered"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "default pressed background",
            token: "action.ghost.pressed_background",
            states: &["pressed"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "default foreground",
            token: "action.ghost.foreground",
            states: &["default", "hovered", "pressed", "focused"],
            appearance_fields: &["ButtonFamilyAppearance.foreground"],
        },
        ThemePartUsage {
            part: "default border",
            token: "border.default",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            appearance_fields: &["ButtonFamilyAppearance.border"],
        },
        ThemePartUsage {
            part: "primary background",
            token: "action.primary.background",
            states: &["default"],
            appearance_fields: &["ButtonFamilyAppearance.background", "ButtonFamilyAppearance.border"],
        },
        ThemePartUsage {
            part: "primary hover background",
            token: "action.primary.hover_background",
            states: &["hovered"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "primary pressed background",
            token: "action.primary.pressed_background",
            states: &["pressed"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "primary foreground",
            token: "action.primary.foreground",
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
            appearance_fields: &["ButtonFamilyAppearance.focus_ring"],
        },
    ],
};

pub const ICON_BUTTON_THEME_USAGE: ThemeUsage =
    ThemeUsage { component: "Icon Button", parts: BUTTON_THEME_USAGE.parts };

pub const TOGGLE_THEME_USAGE: ThemeUsage = ThemeUsage {
    component: "Toggle",
    parts: &[
        ThemePartUsage {
            part: "unselected default background",
            token: "action.ghost.background",
            states: &["default"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "unselected default hover background",
            token: "action.ghost.hover_background",
            states: &["hovered"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "unselected default pressed background",
            token: "action.ghost.pressed_background",
            states: &["pressed"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "unselected default foreground",
            token: "action.ghost.foreground",
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
            token: "action.primary.hover_background",
            states: &["selected hovered"],
            appearance_fields: &["ButtonFamilyAppearance.background"],
        },
        ThemePartUsage {
            part: "selected pressed background",
            token: "action.primary.pressed_background",
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
            part: "primary background",
            token: "action.primary.background",
            states: &["default"],
            appearance_fields: &["ButtonFamilyAppearance.background", "ButtonFamilyAppearance.border"],
        },
        ThemePartUsage {
            part: "primary foreground",
            token: "action.primary.foreground",
            states: &["default", "hovered", "pressed", "focused"],
            appearance_fields: &["ButtonFamilyAppearance.foreground"],
        },
        ThemePartUsage {
            part: "default border",
            token: "border.default",
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
            appearance_fields: &["ButtonFamilyAppearance.focus_ring"],
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
            (ButtonVariant::Default, false, false) => palette.action.ghost.foreground,
            (ButtonVariant::Ghost, false, false) => palette.action.ghost.foreground,
            (ButtonVariant::Primary, false, false) => palette.action.primary.foreground,
        };

        let background = match (variant, selected, state.layer()) {
            (_, _, InteractionLayer::Disabled) => palette.state.disabled.background,
            (_, true, InteractionLayer::Pressed) => palette.action.primary.pressed_background,
            (_, true, InteractionLayer::Hovered) => palette.action.primary.hover_background,
            (_, true, InteractionLayer::Default) => palette.state.selected.background,
            (ButtonVariant::Primary, _, InteractionLayer::Pressed) => palette.action.primary.pressed_background,
            (ButtonVariant::Primary, _, InteractionLayer::Hovered) => palette.action.primary.hover_background,
            (ButtonVariant::Primary, _, InteractionLayer::Default) => palette.action.primary.background,
            (ButtonVariant::Default, _, InteractionLayer::Pressed) => palette.action.ghost.pressed_background,
            (ButtonVariant::Default, _, InteractionLayer::Hovered) => palette.action.ghost.hover_background,
            (ButtonVariant::Default, _, InteractionLayer::Default) => palette.action.ghost.background,
            (ButtonVariant::Ghost, _, InteractionLayer::Pressed) => palette.state.pressed.background,
            (ButtonVariant::Ghost, _, InteractionLayer::Hovered) => palette.state.hover.background,
            (ButtonVariant::Ghost, _, InteractionLayer::Default) => transparent,
        };

        let height = metrics.control_height(size);
        let border = match variant {
            ButtonVariant::Default => palette.border.default,
            ButtonVariant::Ghost => transparent,
            ButtonVariant::Primary => palette.action.primary.background,
        };

        ButtonFamilyAppearance {
            background,
            foreground,
            border,
            focus_ring: state.focused.then_some(palette.focus.ring),
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
