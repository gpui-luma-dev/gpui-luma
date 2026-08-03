use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::controls::floating_menu::{FloatingMenuLook, default_floating_menu_look};
use crate::theme::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, MetricTokens, ThemeTokens};

#[derive(Clone, Debug)]
pub struct ContextMenuLook {
    pub target_background: Hsla,
    pub target_foreground: Hsla,
    pub target_border: Hsla,
    pub target_typography: LumaTextStyle,
    pub target_radius: f32,
    pub target_padding_x: f32,
    pub target_padding_y: f32,
    pub target_min_width: f32,
    pub floating_menu: FloatingMenuLook,
}

pub trait ContextMenuTheme: Send + Sync {
    fn resolve(&self, state: InteractionState) -> ContextMenuLook;

    fn metrics(&self) -> MetricTokens {
        MetricTokens::default()
    }
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
    fn resolve(&self, state: InteractionState) -> ContextMenuLook {
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

        ContextMenuLook {
            target_background,
            target_foreground,
            target_border: palette.border.default,
            target_typography: typography.text.label,
            target_radius: metrics.radius(size),
            target_padding_x: metrics.padding_x(size),
            target_padding_y: metrics.padding_y(size),
            target_min_width: 200.0,
            floating_menu: default_floating_menu_look(&self.tokens, size),
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }
}
