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
        let palette = self.tokens.stylesheet.recipes.palette("progress", &self.tokens.palette);

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

crate::theme::stylesheet::geometry::geometry_family!(
    ProgressStylesheet,
    ProgressGeometryStylesheet,
    ProgressGeometry,
    ResolvedProgressGeometry,
    "progress",
    [track_height, size, stroke_width, thumb_size],
    unit[indeterminate_fraction, shimmer_soft_opacity, shimmer_mid_opacity, shimmer_mid_offset, shimmer_mid_width, shimmer_core_offset, shimmer_core_width]
);

impl ProgressStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: ProgressGeometryStylesheet {
                indeterminate_fraction: Some(0.28),
                shimmer_soft_opacity: Some(0.35),
                shimmer_mid_opacity: Some(0.85),
                shimmer_mid_offset: Some(0.2),
                shimmer_mid_width: Some(0.6),
                shimmer_core_offset: Some(0.35),
                shimmer_core_width: Some(0.3),
                ..Default::default()
            },
            sizes: [("sm", 4.0, 48.0, 5.0, 12.0), ("md", 6.0, 64.0, 6.0, 16.0), ("lg", 8.0, 80.0, 8.0, 20.0)]
                .into_iter()
                .map(|(key, track, size, stroke, thumb)| {
                    (
                        key.into(),
                        ProgressGeometryStylesheet {
                            track_height: Some(track),
                            size: Some(size),
                            stroke_width: Some(stroke),
                            thumb_size: Some(thumb),
                            ..Default::default()
                        },
                    )
                })
                .collect(),
        }
    }
}
