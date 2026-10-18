use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla, SharedString, black};

use crate::theme::{ControlSize, LumaTextStyle, ThemeTokens};

use super::OverlayWindowMode;

#[derive(Clone, Debug)]
pub struct OverlayWindowLook {
    pub background: Hsla,
    pub border: Hsla,
    pub foreground: Hsla,
    pub backdrop_background: Hsla,
    pub shadow: Vec<BoxShadow>,
    pub radius: f32,
    pub padding: f32,
    pub min_width: f32,
    pub max_width: f32,
    pub estimated_height: f32,
    pub body: LumaTextStyle,
    pub font_family: SharedString,
}

pub trait OverlayWindowTheme: Send + Sync {
    fn resolve(&self, size: ControlSize, mode: OverlayWindowMode) -> OverlayWindowLook;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultOverlayWindowTheme {
    tokens: ThemeTokens,
}

pub fn default_overlay_window_theme() -> Arc<dyn OverlayWindowTheme> {
    static THEME: OnceLock<Arc<dyn OverlayWindowTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultOverlayWindowTheme::default())).clone()
}

impl DefaultOverlayWindowTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl OverlayWindowTheme for DefaultOverlayWindowTheme {
    fn resolve(&self, size: ControlSize, mode: OverlayWindowMode) -> OverlayWindowLook {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let elevation = &self.tokens.elevation;
        let geometry = self.tokens.stylesheet.overlay_window.resolve_geometry(
            size.stylesheet_key(),
            crate::theme::stylesheet::OverlayWindowGeometry {
                padding: metrics.padding_x(size),
                font_size: typography.text.body.size,
                line_height: typography.text.body.line_height,
                ..Default::default()
            },
        );

        OverlayWindowLook {
            background: palette.surface.floating.background,
            border: palette.surface.floating.border,
            foreground: palette.surface.floating.foreground,
            backdrop_background: if mode == OverlayWindowMode::Modal {
                Hsla { a: geometry.backdrop_opacity.value_px, ..black() }
            } else {
                Hsla { a: 0.0, ..black() }
            },
            shadow: if mode == OverlayWindowMode::Modal {
                elevation.dialog.to_box_shadows()
            } else {
                elevation.overlay_window.to_box_shadows()
            },
            radius: metrics.radius.xl,
            padding: geometry.padding.value_px,
            min_width: geometry.min_width.value_px,
            max_width: geometry.max_width.value_px,
            estimated_height: geometry.estimated_height.value_px,
            body: LumaTextStyle {
                size: geometry.font_size.value_px,
                line_height: geometry.line_height.value_px,
                ..typography.text.body
            },
            font_family: typography.font.sans.family.clone().into(),
        }
    }
}
