use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::ThemeTokens;

#[derive(Clone, Copy, Debug)]
pub struct ProgressAppearance {
    pub track_color: Hsla,
    pub progress_color: Hsla,
    pub size: f32,
    pub stroke_width: f32,
}

pub trait ProgressTheme: Send + Sync {
    fn resolve(&self, enabled: bool) -> ProgressAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultProgressTheme {
    tokens: ThemeTokens,
}

pub fn default_progress_theme() -> Arc<dyn ProgressTheme> {
    static THEME: OnceLock<Arc<dyn ProgressTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultProgressTheme::default())).clone()
}

impl DefaultProgressTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl ProgressTheme for DefaultProgressTheme {
    fn resolve(&self, enabled: bool) -> ProgressAppearance {
        let palette = &self.tokens.palette;

        ProgressAppearance {
            track_color: if enabled {
                palette.surface.subtle.background
            } else {
                palette.state.disabled.background
            },
            progress_color: if enabled {
                palette.action.prominent.background
            } else {
                palette.state.disabled.foreground
            },
            size: 64.0,
            stroke_width: 6.0,
        }
    }
}
