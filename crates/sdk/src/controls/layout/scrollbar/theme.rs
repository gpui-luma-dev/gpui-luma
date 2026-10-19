use std::sync::{Arc, OnceLock};

use gpui::{Hsla, hsla};

use crate::controls::scrollbar::{ScrollbarOrientation, ScrollbarStyle};

use crate::theme::{ControlSize, InteractionLayer, InteractionState, MetricTokens, ThemeTokens};

#[derive(Clone, Copy, Debug)]
pub struct ScrollbarLook {
    /// Whole-scrollbar opacity when disabled.
    pub disabled_opacity: f32,
    pub track_background: Hsla,
    pub thumb_background: Hsla,
    pub length: f32,
    pub thickness: f32,
    pub track_thickness: f32,
    pub thumb_thickness: f32,
    pub min_thumb_length: f32,
    pub radius: f32,
}

pub trait ScrollbarTheme: Send + Sync {
    fn resolve(
        &self,
        state: InteractionState,
        orientation: ScrollbarOrientation,
        size: ControlSize,
        style: ScrollbarStyle,
    ) -> ScrollbarLook;

    fn metrics(&self) -> MetricTokens {
        MetricTokens::default()
    }
}

#[derive(Clone, Debug, Default)]
pub struct DefaultScrollbarTheme {
    tokens: ThemeTokens,
}

pub fn default_scrollbar_theme() -> Arc<dyn ScrollbarTheme> {
    static THEME: OnceLock<Arc<dyn ScrollbarTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultScrollbarTheme::default())).clone()
}

impl DefaultScrollbarTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl ScrollbarTheme for DefaultScrollbarTheme {
    fn resolve(
        &self,
        state: InteractionState,
        orientation: ScrollbarOrientation,
        size: ControlSize,
        style: ScrollbarStyle,
    ) -> ScrollbarLook {
        let geometry = self
            .tokens
            .stylesheet
            .scrollbar
            .resolve_geometry(size.stylesheet_key(), crate::theme::stylesheet::ScrollbarGeometry::default());
        let palette = self.tokens.stylesheet.recipes.palette("scrollbar", &self.tokens.palette);
        let metrics = &self.tokens.metrics;
        let thumb_background = match state.layer() {
            InteractionLayer::Disabled => palette.state.disabled.foreground,
            InteractionLayer::Pressed => palette.state.pressed.background,
            InteractionLayer::Hovered => palette.state.hover.background,
            InteractionLayer::Default => palette.border.default,
        };

        let length = match orientation {
            ScrollbarOrientation::Horizontal => geometry.length_h.value_px,
            ScrollbarOrientation::Vertical => geometry.length_v.value_px,
        };

        ScrollbarLook {
            disabled_opacity: geometry.disabled_opacity.value_px,
            track_background: if state.disabled {
                palette.state.disabled.background
            } else if style == ScrollbarStyle::Soft {
                palette.surface.subtle.background
            } else {
                hsla(0.0, 0.0, 0.0, 0.0)
            },
            thumb_background,
            length,
            thickness: geometry.thickness.value_px,
            track_thickness: geometry.track_thickness.value_px,
            thumb_thickness: geometry.thumb_thickness.value_px,
            min_thumb_length: geometry.min_thumb_length.value_px,
            radius: metrics.radius.pill,
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }
}

crate::theme::stylesheet::geometry::geometry_family!(
    ScrollbarStylesheet,
    ScrollbarGeometryStylesheet,
    ScrollbarGeometry,
    ResolvedScrollbarGeometry,
    "scrollbar",
    [thickness, track_thickness, thumb_thickness, min_thumb_length, length_h, length_v],
    unit[disabled_opacity, thumb_fraction]
);

impl ScrollbarStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: ScrollbarGeometryStylesheet {
                thumb_fraction: Some(0.25),
                disabled_opacity: Some(0.56),
                ..Default::default()
            },
            sizes: [("sm", 10.0, 3.0, 6.0, 24.0), ("md", 12.0, 4.0, 8.0, 28.0), ("lg", 14.0, 5.0, 10.0, 32.0)]
                .into_iter()
                .map(|(key, thickness, track, thumb, min)| {
                    (
                        key.into(),
                        ScrollbarGeometryStylesheet {
                            thickness: Some(thickness),
                            track_thickness: Some(track),
                            thumb_thickness: Some(thumb),
                            min_thumb_length: Some(min),
                            length_h: Some(260.0),
                            length_v: Some(180.0),
                            ..Default::default()
                        },
                    )
                })
                .collect(),
        }
    }
}
