use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::ThemeTokens;

#[derive(Clone, Debug)]
pub struct DockSplitterLook {
    /// Captured, resolved thumb geometry and resting opacity.
    pub geometry: crate::theme::stylesheet::ResolvedDockSplitterGeometry,
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
        let geometry = self.tokens.stylesheet.dock_splitter.resolve_geometry("md", Default::default());

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
            hit_target_px: geometry.hit_target.value_px,
            visible_line_px: geometry.visible_line.value_px,
            geometry,
        }
    }
}
