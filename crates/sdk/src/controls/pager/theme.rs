use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::{LumaTextStyle, ThemeTokens};

use super::PagerStyle;

#[derive(Clone, Debug)]
pub struct PagerAppearance {
    pub panel_background: Hsla,
    pub border: Hsla,
    pub body_text: Hsla,
    pub muted_text: Hsla,
    pub selected_background: Hsla,
    pub selected_foreground: Hsla,
    pub hover_background: Hsla,
    pub shadow: Vec<gpui::BoxShadow>,
    pub typography: LumaTextStyle,
    pub button_size: f32,
    pub button_min_width: f32,
    pub control_height: f32,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub group_gap: f32,
    pub disabled_opacity: f32,
}

pub trait PagerTheme: Send + Sync {
    fn resolve(&self, enabled: bool, style: PagerStyle) -> PagerAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultPagerTheme {
    tokens: ThemeTokens,
}

pub fn default_pager_theme() -> Arc<dyn PagerTheme> {
    static THEME: OnceLock<Arc<dyn PagerTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultPagerTheme::default())).clone()
}

impl DefaultPagerTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl PagerTheme for DefaultPagerTheme {
    fn resolve(&self, enabled: bool, style: PagerStyle) -> PagerAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = self.tokens.typography.text.caption;
        let compact = matches!(style, PagerStyle::Minimal);

        PagerAppearance {
            panel_background: if enabled {
                palette.surface.panel.background
            } else {
                palette.state.disabled.background
            },
            border: palette.form.input.border,
            body_text: if enabled {
                palette.app.foreground
            } else {
                palette.state.disabled.foreground
            },
            muted_text: if enabled {
                palette.app.muted_foreground
            } else {
                palette.state.disabled.foreground
            },
            selected_background: palette.state.selected.background,
            selected_foreground: palette.state.selected.foreground,
            hover_background: palette.state.hover.background,
            shadow: vec![],
            typography,
            button_size: if compact {
                28.0
            } else {
                metrics.control_height(crate::theme::ControlSize::Sm)
            },
            button_min_width: if compact { 28.0 } else { 32.0 },
            control_height: metrics.control_height(crate::theme::ControlSize::Sm),
            radius: metrics.radius(crate::theme::ControlSize::Sm),
            padding_x: 8.0,
            padding_y: 8.0,
            gap: 4.0,
            group_gap: if compact { 8.0 } else { 16.0 },
            disabled_opacity: 0.56,
        }
    }
}
