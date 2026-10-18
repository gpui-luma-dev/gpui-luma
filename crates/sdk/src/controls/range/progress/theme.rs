use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::{ControlSize, ThemeTokens};

#[derive(Clone, Debug)]
pub struct ProgressLook {
    /// Resolved sweep and shimmer appearance, independent of phase/timing.
    pub geometry: crate::theme::stylesheet::ResolvedProgressGeometry,
    pub track_color: Hsla,
    pub progress_color: Hsla,
    pub thumb_color: Hsla,
    pub track_height: f32,
    pub thumb_size: f32,
    pub size: f32,
    pub stroke_width: f32,
}

pub trait ProgressTheme: Send + Sync {
    fn resolve(&self, enabled: bool, size: ControlSize) -> ProgressLook;
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
    fn resolve(&self, enabled: bool, size: ControlSize) -> ProgressLook {
        let geometry = self
            .tokens
            .stylesheet
            .progress
            .resolve_geometry(size.stylesheet_key(), crate::theme::stylesheet::ProgressGeometry::default());
        let palette = &self.tokens.palette;

        let track_color = if enabled {
            palette.surface.subtle.background
        } else {
            palette.state.disabled.background
        };
        let progress_color = if enabled {
            palette.state.selected.background
        } else {
            palette.state.disabled.foreground
        };

        ProgressLook {
            track_color,
            progress_color,
            thumb_color: progress_color,
            track_height: geometry.track_height.value_px,
            thumb_size: geometry.thumb_size.value_px,
            size: geometry.size.value_px,
            stroke_width: geometry.stroke_width.value_px,
            geometry,
        }
    }
}
