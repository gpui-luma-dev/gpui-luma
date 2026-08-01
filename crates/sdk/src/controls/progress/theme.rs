use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::{ControlSize, ThemeTokens};

#[derive(Clone, Copy, Debug)]
pub struct ProgressLook {
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
            track_height: progress_track_height(size),
            thumb_size: progress_thumb_size(size),
            size: progress_ring_size(size),
            stroke_width: progress_stroke_width(size),
        }
    }
}

fn progress_ring_size(size: ControlSize) -> f32 {
    match size {
        ControlSize::Sm => 48.0,
        ControlSize::Md => 64.0,
        ControlSize::Lg => 80.0,
    }
}

fn progress_stroke_width(size: ControlSize) -> f32 {
    match size {
        ControlSize::Sm => 5.0,
        ControlSize::Md => 6.0,
        ControlSize::Lg => 8.0,
    }
}

fn progress_track_height(size: ControlSize) -> f32 {
    match size {
        ControlSize::Sm => 4.0,
        ControlSize::Md => 6.0,
        ControlSize::Lg => 8.0,
    }
}

fn progress_thumb_size(size: ControlSize) -> f32 {
    match size {
        ControlSize::Sm => 12.0,
        ControlSize::Md => 16.0,
        ControlSize::Lg => 20.0,
    }
}
