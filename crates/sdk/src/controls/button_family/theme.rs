use std::sync::{Arc, OnceLock};

use gpui::{Hsla, SharedString};

use crate::theme::adorner::{AdornerPlacement, AdornerSpec, FocusRingAdornerSpec};
use crate::theme::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemeTokens};

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
    fn resolve(&self, role: ButtonFamilyRole, size: ControlSize, state: InteractionState) -> ButtonFamilyAppearance;
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
    fn resolve(&self, role: ButtonFamilyRole, size: ControlSize, state: InteractionState) -> ButtonFamilyAppearance {
        native_button_appearance(&self.tokens, role, size, state)
    }
}

fn native_button_appearance(
    tokens: &ThemeTokens,
    role: ButtonFamilyRole,
    size: ControlSize,
    state: InteractionState,
) -> ButtonFamilyAppearance {
    let palette = &tokens.palette;
    let metrics = &tokens.metrics;
    let typography = &tokens.typography;
    let layer = state.layer();
    let action = native_action_role(palette, role);

    let foreground = if state.disabled {
        palette.state.disabled.foreground
    } else {
        action.foreground
    };

    let background = match layer {
        InteractionLayer::Disabled => palette.state.disabled.background,
        InteractionLayer::Pressed => action.pressed_background,
        InteractionLayer::Hovered => action.hover_background,
        InteractionLayer::Default => action.background,
    };

    let adorner = if state.focused {
        Some(AdornerSpec::FocusRing(FocusRingAdornerSpec {
            color: palette.focus.ring,
            placement: AdornerPlacement::Oversize,
            distance: metrics.border_width.default + metrics.focus.width,
            width: metrics.focus.width,
        }))
    } else {
        None
    };

    ButtonFamilyAppearance {
        background,
        foreground,
        border: action.border,
        adorner,
        typography: typography.text.label,
        font_family: typography.font.sans.family.clone().into(),
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
        height: metrics.control_height(size),
    }
}

fn native_action_role(palette: &crate::theme::LumaPalette, role: ButtonFamilyRole) -> crate::theme::ActionRolePalette {
    match role {
        ButtonFamilyRole::Toggle { selected: false } => palette.action.subtle,
        _ => palette.action.standard,
    }
}
