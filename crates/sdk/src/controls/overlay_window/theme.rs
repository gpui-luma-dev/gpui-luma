use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla, SharedString, black, point, px};

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
        let min_width = match size {
            ControlSize::Sm => 320.0,
            ControlSize::Md => 400.0,
            ControlSize::Lg => 480.0,
        };

        OverlayWindowLook {
            background: palette.surface.floating.background,
            border: palette.surface.floating.border,
            foreground: palette.surface.floating.foreground,
            backdrop_background: if mode == OverlayWindowMode::Modal {
                Hsla { a: 0.44, ..black() }
            } else {
                Hsla { a: 0.0, ..black() }
            },
            shadow: if mode == OverlayWindowMode::Modal {
                elevation.dialog.to_box_shadows()
            } else {
                vec![BoxShadow {
                    offset: point(px(0.0), px(18.0)),
                    blur_radius: px(38.0),
                    spread_radius: px(-16.0),
                    color: Hsla { a: 0.20, ..black() },
                    inset: false,
                }]
            },
            radius: metrics.radius.xl,
            padding: metrics.padding_x(size),
            min_width,
            max_width: min_width + 120.0,
            estimated_height: match size {
                ControlSize::Sm => 196.0,
                ControlSize::Md => 228.0,
                ControlSize::Lg => 264.0,
            },
            body: typography.text.body,
            font_family: typography.font.sans.family.clone().into(),
        }
    }
}
