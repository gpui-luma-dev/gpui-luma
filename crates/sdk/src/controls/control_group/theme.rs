use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::{ControlSize, ThemeTokens};

#[derive(Clone, Copy, Debug)]
pub struct ControlGroupListLook {
    pub background: Hsla,
    pub border: Hsla,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
}

pub trait ControlGroupTheme: Send + Sync {
    fn resolve_list(&self, enabled: bool) -> ControlGroupListLook;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultControlGroupTheme {
    tokens: ThemeTokens,
}

pub fn default_control_group_theme() -> Arc<dyn ControlGroupTheme> {
    static THEME: OnceLock<Arc<dyn ControlGroupTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultControlGroupTheme::default())).clone()
}

impl DefaultControlGroupTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl ControlGroupTheme for DefaultControlGroupTheme {
    fn resolve_list(&self, enabled: bool) -> ControlGroupListLook {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;

        ControlGroupListLook {
            background: if enabled {
                palette.surface.subtle.background
            } else {
                palette.state.disabled.background
            },
            border: palette.border.default,
            radius: metrics.radius(ControlSize::Md),
            padding_x: 6.0,
            padding_y: 4.0,
            gap: 6.0,
        }
    }
}
