use std::sync::Arc;
use gpui::{BoxShadow, Hsla, SharedString};
use crate::theme::ThemeTokens;

/// Resolved tooltip style. Looks own all ordinary presentation defaults.
#[derive(Clone)]
pub struct TooltipLook {
    pub background: Hsla,
    pub foreground: Hsla,
    pub padding: f32,
    pub padding_y: f32,
    pub radius: f32,
    pub max_width: f32,
    pub text_size: f32,
    pub line_height: f32,
    pub shadow: Vec<BoxShadow>,
}

impl TooltipLook {
    /// Shared compact geometry for look adapters. Colors and corner radius remain
    /// look-owned; adapters may override any resolved field before returning it.
    pub fn compact(background: Hsla, foreground: Hsla, radius: f32) -> Self {
        Self {
            background,
            foreground,
            radius,
            padding: 8.0,
            padding_y: 4.0,
            max_width: 260.0,
            text_size: 12.0,
            line_height: 16.0,
            shadow: Vec::new(),
        }
    }
}

pub trait TooltipTheme: Send + Sync {
    fn resolve(&self) -> TooltipLook;
}

impl TooltipTheme for TooltipLook {
    fn resolve(&self) -> TooltipLook {
        self.clone()
    }
}

pub fn default_tooltip_theme() -> Arc<dyn TooltipTheme> {
    let tokens = ThemeTokens::default();
    Arc::new(TooltipLook {
        background: tokens.palette.surface.floating.background,
        foreground: tokens.palette.surface.floating.foreground,
        padding: 12.0,
        padding_y: 8.0,
        radius: 6.0,
        max_width: 260.0,
        text_size: 14.0,
        line_height: 20.0,
        shadow: tokens.elevation.menu.to_box_shadows(),
    })
}

/// Presentation-only input, resolved from the explicitly bound draft look on every frame.
#[derive(Clone)]
pub struct TooltipRenderModel {
    pub text: SharedString,
    pub shortcut: Option<SharedString>,
    pub background: Hsla,
    pub foreground: Hsla,
    pub padding: f32,
    pub padding_y: f32,
    pub radius: f32,
    pub max_width: f32,
    pub text_size: f32,
    pub line_height: f32,
    pub shadow: Vec<BoxShadow>,
}
