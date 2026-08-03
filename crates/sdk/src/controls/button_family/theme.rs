use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla, SharedString};

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
    /// Explicit border color from the theme. `None` means borderless.
    pub border: Option<Hsla>,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
}

#[derive(Clone, Debug)]
pub struct ButtonFamilyLook {
    pub background: Hsla,
    pub foreground: Hsla,
    /// Explicit border color from the theme. `None` means borderless.
    pub border: Option<Hsla>,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub height: f32,
    pub icon_size: f32,
    pub shadow: Option<Vec<BoxShadow>>,
}

pub trait ButtonFamilyTheme: Send + Sync {
    fn resolve(&self, role: ButtonFamilyRole, size: ControlSize, state: InteractionState) -> ButtonFamilyPalette;
    fn metrics(&self) -> MetricTokens;

    /// Optional full look resolution for themes that resolve geometry and elevation together.
    fn resolve_look(
        &self,
        _role: ButtonFamilyRole,
        _size: ControlSize,
        _state: InteractionState,
        _scale: &StandardBoxScale,
        _pill_radius: f32,
    ) -> Option<ButtonFamilyLook> {
        None
    }
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

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }
}

fn native_button_palette(tokens: &ThemeTokens, role: ButtonFamilyRole, state: InteractionState) -> ButtonFamilyPalette {
    let palette = &tokens.palette;
    let typography = &tokens.typography;
    let layer = state.layer();

    let (base_background, base_foreground, border) = match role {
        ButtonFamilyRole::Toggle { selected: true } => {
            (palette.state.selected.background, palette.state.selected.foreground, palette.border.default)
        }
        _ => (palette.surface.subtle.background, palette.app.foreground, palette.border.default),
    };

    let foreground = if state.disabled {
        palette.state.disabled.foreground
    } else {
        base_foreground
    };

    let background = match layer {
        InteractionLayer::Disabled => palette.state.disabled.background,
        InteractionLayer::Pressed => palette.state.pressed.background,
        InteractionLayer::Hovered => palette.state.hover.background,
        InteractionLayer::Default => base_background,
    };

    ButtonFamilyPalette {
        background,
        foreground,
        border: Some(border),
        typography: typography.text.label,
        font_family: typography.font.sans.family.clone().into(),
    }
}

/// Resolves the border color used for layout and focus-ring placement.
///
/// Absent borders are treated as transparent (borderless).
pub fn button_family_effective_border(border: Option<Hsla>) -> Hsla {
    border.unwrap_or_else(|| gpui::hsla(0.0, 0.0, 0.0, 0.0))
}

pub fn compose_button_family_look(
    palette: &ButtonFamilyPalette,
    role: ButtonFamilyRole,
    scale: &StandardBoxScale,
    pill_radius: f32,
) -> ButtonFamilyLook {
    ButtonFamilyLook {
        background: palette.background,
        foreground: palette.foreground,
        border: palette.border,
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
        icon_size: default_button_icon_size(role, scale, &palette.typography),
        shadow: None,
    }
}

fn default_button_icon_size(role: ButtonFamilyRole, scale: &StandardBoxScale, typography: &LumaTextStyle) -> f32 {
    if matches!(role, ButtonFamilyRole::Icon) {
        scale.height * 0.44
    } else {
        typography.size
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn effective_border_is_transparent_when_absent() {
        let effective = button_family_effective_border(None);
        assert_eq!(effective.a, 0.0);
    }
}
