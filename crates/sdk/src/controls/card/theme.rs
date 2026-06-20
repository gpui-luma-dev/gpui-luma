use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla, SharedString};

use crate::theme::{ControlSize, LumaTextStyle, ThemeTokens};

#[derive(Clone, Debug)]
pub struct CardLook {
    pub background: Hsla,
    pub border: Hsla,
    pub title_color: Hsla,
    pub description_color: Hsla,
    pub body_color: Hsla,
    pub shadow: Vec<BoxShadow>,
    pub radius: f32,
    pub padding: f32,
    pub section_gap: f32,
    pub header_gap: f32,
    pub body_gap: f32,
    pub title: LumaTextStyle,
    pub description: LumaTextStyle,
    pub body: LumaTextStyle,
    pub font_family: SharedString,
}

pub trait CardTheme: Send + Sync {
    fn resolve(&self, size: ControlSize) -> CardLook;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultCardTheme {
    tokens: ThemeTokens,
}

pub fn default_card_theme() -> Arc<dyn CardTheme> {
    static THEME: OnceLock<Arc<dyn CardTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultCardTheme::default())).clone()
}

impl DefaultCardTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl CardTheme for DefaultCardTheme {
    fn resolve(&self, size: ControlSize) -> CardLook {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let elevation = &self.tokens.elevation;

        CardLook {
            background: palette.surface.panel.background,
            border: palette.surface.panel.border,
            title_color: palette.surface.panel.foreground,
            description_color: palette.app.muted_foreground,
            body_color: palette.surface.panel.foreground,
            shadow: elevation.panel.to_box_shadows(),
            radius: metrics.radius.lg,
            padding: metrics.padding_x(size),
            section_gap: metrics.gap(size),
            header_gap: (metrics.gap(size) * 0.5).max(2.0),
            body_gap: metrics.gap(size),
            title: typography.text.title,
            description: typography.text.caption,
            body: typography.text.body,
            font_family: typography.font.sans.family.clone().into(),
        }
    }
}
