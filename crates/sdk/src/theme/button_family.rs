use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use super::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemeTokens};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonVariant {
    #[default]
    Default,
    Primary,
    Destructive,
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
        let foreground = match (variant, selected, state.disabled) {
            (_, _, true) => palette.state.disabled.foreground,
            (_, true, false) => palette.state.selected.foreground,
            (ButtonVariant::Default, false, false) => palette.action.secondary.foreground,
            (ButtonVariant::Primary, false, false) => palette.action.primary.foreground,
            (ButtonVariant::Destructive, false, false) => palette.action.danger.foreground,
        };

        let background = match (variant, selected, state.layer()) {
            (_, _, InteractionLayer::Disabled) => palette.state.disabled.background,
            (_, true, InteractionLayer::Pressed) => palette.action.primary.pressed_background,
            (_, true, InteractionLayer::Hovered) => palette.action.primary.hover_background,
            (_, true, InteractionLayer::Default) => palette.state.selected.background,
            (ButtonVariant::Primary, _, InteractionLayer::Pressed) => palette.action.primary.pressed_background,
            (ButtonVariant::Primary, _, InteractionLayer::Hovered) => palette.action.primary.hover_background,
            (ButtonVariant::Primary, _, InteractionLayer::Default) => palette.action.primary.background,
            (ButtonVariant::Destructive, _, InteractionLayer::Pressed) => palette.action.danger.pressed_background,
            (ButtonVariant::Destructive, _, InteractionLayer::Hovered) => palette.action.danger.hover_background,
            (ButtonVariant::Destructive, _, InteractionLayer::Default) => palette.action.danger.background,
            (ButtonVariant::Default, _, InteractionLayer::Pressed) => palette.action.secondary.pressed_background,
            (ButtonVariant::Default, _, InteractionLayer::Hovered) => palette.action.secondary.hover_background,
            (ButtonVariant::Default, _, InteractionLayer::Default) => palette.action.secondary.background,
        };

        let height = metrics.control_height(size);
        let border = match variant {
            ButtonVariant::Default => palette.border.default,
            ButtonVariant::Primary => palette.action.primary.background,
            ButtonVariant::Destructive => palette.action.danger.background,
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
