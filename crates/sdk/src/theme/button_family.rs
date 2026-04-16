use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use super::{ControlSize, InteractionLayer, InteractionState, ThemeTokens};

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

    THEME
        .get_or_init(|| Arc::new(DefaultButtonFamilyTheme::default()))
        .clone()
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
        let colors = &self.tokens.colors;
        let metrics = &self.tokens.metrics;
        let selected = matches!(role, ButtonFamilyRole::Toggle { selected: true });
        let variant = if selected {
            ButtonVariant::Primary
        } else {
            variant
        };
        let foreground = match (variant, state.disabled) {
            (_, true) => colors.text_disabled,
            (ButtonVariant::Default, false) => colors.text,
            _ => colors.text_inverse,
        };

        let background = match (variant, selected, state.layer()) {
            (_, _, InteractionLayer::Disabled) => colors.surface_disabled,
            (_, true, InteractionLayer::Pressed) => colors.selected_pressed,
            (_, true, InteractionLayer::Hovered) => colors.selected_hover,
            (_, true, InteractionLayer::Default) => colors.selected,
            (ButtonVariant::Primary, _, InteractionLayer::Pressed) => colors.primary_pressed,
            (ButtonVariant::Primary, _, InteractionLayer::Hovered) => colors.primary_hover,
            (ButtonVariant::Primary, _, InteractionLayer::Default) => colors.primary,
            (ButtonVariant::Destructive, _, InteractionLayer::Pressed) => {
                colors.destructive_pressed
            }
            (ButtonVariant::Destructive, _, InteractionLayer::Hovered) => colors.destructive_hover,
            (ButtonVariant::Destructive, _, InteractionLayer::Default) => colors.destructive,
            (ButtonVariant::Default, _, InteractionLayer::Pressed) => colors.surface_pressed,
            (ButtonVariant::Default, _, InteractionLayer::Hovered) => colors.surface_hover,
            (ButtonVariant::Default, _, InteractionLayer::Default) => colors.surface,
        };

        let height = metrics.control_height(size);

        ButtonFamilyAppearance {
            background,
            foreground,
            border: colors.border,
            focus_ring: state.focused.then_some(colors.focus_ring),
            radius: match role {
                ButtonFamilyRole::Icon => height / 2.0,
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
