use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::controls::floating_menu::{FloatingMenuAppearance, default_floating_menu_appearance};
use crate::theme::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemeTokens};

#[derive(Clone, Debug)]
pub struct ContextMenuAppearance {
    pub target_background: Hsla,
    pub target_foreground: Hsla,
    pub target_border: Hsla,
    pub focus_ring: Option<Hsla>,
    pub target_typography: LumaTextStyle,
    pub target_radius: f32,
    pub target_padding_x: f32,
    pub target_padding_y: f32,
    pub target_min_width: f32,
    pub floating_menu: FloatingMenuAppearance,
}

pub trait ContextMenuTheme: Send + Sync {
    fn resolve(&self, state: InteractionState) -> ContextMenuAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultContextMenuTheme {
    tokens: ThemeTokens,
}

pub fn default_context_menu_theme() -> Arc<dyn ContextMenuTheme> {
    static THEME: OnceLock<Arc<dyn ContextMenuTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultContextMenuTheme::default())).clone()
}

impl DefaultContextMenuTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl ContextMenuTheme for DefaultContextMenuTheme {
    fn resolve(&self, state: InteractionState) -> ContextMenuAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let size = ControlSize::Md;

        let target_background = match state.layer() {
            InteractionLayer::Disabled => palette.state.disabled.background,
            InteractionLayer::Pressed => palette.state.pressed.background,
            InteractionLayer::Hovered => palette.state.hover.background,
            InteractionLayer::Default => palette.app.background,
        };
        let target_foreground = if state.disabled {
            palette.state.disabled.foreground
        } else {
            palette.app.foreground
        };

        ContextMenuAppearance {
            target_background,
            target_foreground,
            target_border: palette.border.default,
            focus_ring: state.focused.then_some(palette.focus.ring),
            target_typography: typography.text.label,
            target_radius: metrics.radius(size),
            target_padding_x: metrics.padding_x(size),
            target_padding_y: metrics.padding_y(size),
            target_min_width: 200.0,
            floating_menu: default_floating_menu_appearance(&self.tokens, size),
        }
    }
}
