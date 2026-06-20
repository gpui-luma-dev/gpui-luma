use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::ThemeTokens;

#[derive(Clone, Copy, Debug)]
pub struct DockSplitterLook {
    pub line_color: Hsla,
    pub hover_color: Hsla,
    pub thumb_color: Hsla,
    pub hit_target_px: f32,
    pub visible_line_px: f32,
}

pub trait DockSplitterTheme: Send + Sync {
    fn resolve(&self, enabled: bool) -> DockSplitterLook;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultDockSplitterTheme {
    tokens: ThemeTokens,
}

pub fn default_dock_splitter_theme() -> Arc<dyn DockSplitterTheme> {
    static THEME: OnceLock<Arc<dyn DockSplitterTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultDockSplitterTheme::default())).clone()
}

impl DefaultDockSplitterTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl DockSplitterTheme for DefaultDockSplitterTheme {
    fn resolve(&self, enabled: bool) -> DockSplitterLook {
        let palette = &self.tokens.palette;

        DockSplitterLook {
            line_color: if enabled {
                palette.border.default
            } else {
                palette.state.disabled.foreground
            },
            hover_color: if enabled {
                palette.state.selected.background
            } else {
                palette.state.disabled.foreground
            },
            thumb_color: if enabled {
                palette.state.selected.background
            } else {
                palette.state.disabled.foreground
            },
            hit_target_px: 8.0,
            visible_line_px: 1.0,
        }
    }
}
