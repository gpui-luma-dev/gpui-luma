use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla, SharedString, black, point, px};

use crate::theme::{ControlSize, LumaTextStyle, ThemeTokens};

use super::DialogMode;

#[derive(Clone, Debug)]
pub struct DialogLook {
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

pub trait DialogTheme: Send + Sync {
    fn resolve(&self, size: ControlSize, mode: DialogMode) -> DialogLook;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultDialogTheme {
    tokens: ThemeTokens,
}

pub fn default_dialog_theme() -> Arc<dyn DialogTheme> {
    static THEME: OnceLock<Arc<dyn DialogTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultDialogTheme::default())).clone()
}

impl DefaultDialogTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl DialogTheme for DefaultDialogTheme {
    fn resolve(&self, size: ControlSize, mode: DialogMode) -> DialogLook {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let elevation = &self.tokens.elevation;
        let min_width = match size {
            ControlSize::Sm => 320.0,
            ControlSize::Md => 400.0,
            ControlSize::Lg => 480.0,
        };

        DialogLook {
            background: palette.surface.floating.background,
            border: palette.surface.floating.border,
            foreground: palette.surface.floating.foreground,
            backdrop_background: if mode == DialogMode::Modal {
                Hsla { a: 0.44, ..black() }
            } else {
                Hsla { a: 0.0, ..black() }
            },
            shadow: if mode == DialogMode::Modal {
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
