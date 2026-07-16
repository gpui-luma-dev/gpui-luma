use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::{ControlSize, ThemeTokens};

#[derive(Clone, Debug)]
pub struct ToolbarLook {
    pub background: Hsla,
    pub border: Hsla,
    pub separator: Hsla,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub separator_height: f32,
}

pub trait ToolbarTheme: Send + Sync {
    fn resolve(&self, enabled: bool, size: ControlSize) -> ToolbarLook;
}

pub struct DefaultToolbarTheme {
    tokens: ThemeTokens,
}

impl Default for DefaultToolbarTheme {
    fn default() -> Self {
        Self::new(ThemeTokens::default())
    }
}

impl DefaultToolbarTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

pub fn default_toolbar_theme() -> Arc<dyn ToolbarTheme> {
    static THEME: OnceLock<Arc<dyn ToolbarTheme>> = OnceLock::new();
    THEME.get_or_init(|| Arc::new(DefaultToolbarTheme::default())).clone()
}

impl ToolbarTheme for DefaultToolbarTheme {
    fn resolve(&self, enabled: bool, size: ControlSize) -> ToolbarLook {
        let metrics = &self.tokens.metrics;
        let control = metrics.for_size(size);
        let background = if enabled {
            self.tokens.palette.surface.subtle.background
        } else {
            self.tokens.palette.state.disabled.background
        };

        ToolbarLook {
            background,
            border: self.tokens.palette.border.default,
            separator: self.tokens.palette.border.default,
            radius: metrics.radius.md,
            padding_x: metrics.spacing.s2,
            padding_y: metrics.spacing.s1,
            gap: control.gap,
            separator_height: control.height,
        }
    }
}
