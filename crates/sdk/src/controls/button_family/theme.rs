use std::sync::{Arc, OnceLock};

use gpui::{Hsla, SharedString};

use crate::theme::adorner::{AdornerPlacement, AdornerSpec, FocusRingAdornerSpec};
use crate::theme::{
    ControlSize, InteractionLayer, InteractionState, LumaTextStyle, MetricTokens, StandardBoxScale, ThemeTokens,
};

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
pub struct ButtonFamilyPalette {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub adorner: Option<AdornerSpec>,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
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
    fn resolve(&self, role: ButtonFamilyRole, size: ControlSize, state: InteractionState) -> ButtonFamilyPalette;
    fn metrics(&self) -> &MetricTokens;
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
    fn resolve(&self, role: ButtonFamilyRole, _size: ControlSize, state: InteractionState) -> ButtonFamilyPalette {
        native_button_palette(&self.tokens, role, state)
    }

    fn metrics(&self) -> &MetricTokens {
        &self.tokens.metrics
    }
}

fn native_button_palette(tokens: &ThemeTokens, role: ButtonFamilyRole, state: InteractionState) -> ButtonFamilyPalette {
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

    ButtonFamilyPalette {
        background,
        foreground,
        border: action.border,
        adorner,
        typography: typography.text.label,
        font_family: typography.font.sans.family.clone().into(),
    }
}

pub(crate) fn compose_button_family_appearance(
    palette: &ButtonFamilyPalette,
    role: ButtonFamilyRole,
    scale: &StandardBoxScale,
    pill_radius: f32,
) -> ButtonFamilyAppearance {
    ButtonFamilyAppearance {
        background: palette.background,
        foreground: palette.foreground,
        border: palette.border,
        adorner: palette.adorner,
        typography: palette.typography,
        font_family: palette.font_family.clone(),
        radius: match role {
            ButtonFamilyRole::Icon => pill_radius,
            _ => scale.radius,
        },
        padding_x: match role {
            ButtonFamilyRole::Icon => 0.0,
            _ => scale.padding_x,
        },
        padding_y: match role {
            ButtonFamilyRole::Icon => 0.0,
            _ => scale.padding_y,
        },
        gap: scale.gap,
        height: scale.height,
    }
}

fn native_action_role(palette: &crate::theme::LumaPalette, role: ButtonFamilyRole) -> crate::theme::ActionRolePalette {
    match role {
        ButtonFamilyRole::Toggle { selected: false } => palette.action.subtle,
        _ => palette.action.standard,
    }
}
