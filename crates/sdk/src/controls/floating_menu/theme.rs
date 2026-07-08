use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla};

use crate::theme::{ControlSize, LumaTextStyle, ThemeTokens};

#[derive(Clone, Debug)]
pub struct FloatingMenuLook {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub shadow: Vec<BoxShadow>,
    pub radius: f32,
    pub padding: f32,
    pub min_width: f32,
    pub item_disabled_foreground: Hsla,
    pub item_hover_background: Hsla,
    pub item_hover_foreground: Hsla,
    pub item_typography: LumaTextStyle,
    pub item_height: f32,
    pub item_padding_x: f32,
    pub item_gap: f32,
    pub item_icon_size: f32,
    pub item_radius: f32,
    pub submenu_offset_x: f32,
}

pub trait FloatingMenuTheme: Send + Sync {
    fn resolve(&self) -> FloatingMenuLook;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultFloatingMenuTheme {
    tokens: ThemeTokens,
}

pub fn default_floating_menu_theme() -> Arc<dyn FloatingMenuTheme> {
    static THEME: OnceLock<Arc<dyn FloatingMenuTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultFloatingMenuTheme::default())).clone()
}

impl DefaultFloatingMenuTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl FloatingMenuTheme for DefaultFloatingMenuTheme {
    fn resolve(&self) -> FloatingMenuLook {
        default_floating_menu_look(&self.tokens, ControlSize::Md)
    }
}

pub(crate) fn default_floating_menu_look(tokens: &ThemeTokens, size: ControlSize) -> FloatingMenuLook {
    let palette = &tokens.palette;
    let metrics = &tokens.metrics;
    let typography = &tokens.typography;
    let elevation = &tokens.elevation;
    let item_typography = scaled_menu_item_typography(typography, size);

    FloatingMenuLook {
        background: palette.surface.floating.background,
        foreground: palette.surface.floating.foreground,
        border: palette.surface.floating.border,
        shadow: elevation.menu.to_box_shadows(),
        radius: metrics.radius.lg,
        padding: metrics.padding_y(size) * 0.5,
        min_width: 180.0,
        item_disabled_foreground: palette.state.disabled.foreground,
        item_hover_background: palette.state.hover.background,
        item_hover_foreground: palette.state.hover.foreground,
        item_typography,
        item_height: metrics.control_height(size) * 0.9,
        item_padding_x: metrics.padding_x(size) * 0.75,
        item_gap: metrics.gap(size),
        item_icon_size: default_menu_item_icon_size(metrics, size),
        item_radius: metrics.radius.sm,
        submenu_offset_x: metrics.gap(size) * 0.5,
    }
}

fn scaled_menu_item_typography(typography: &crate::theme::LumaTypography, size: ControlSize) -> LumaTextStyle {
    let mut item_typography = typography.text.label;
    let font_size = match size {
        ControlSize::Sm => 12.0,
        ControlSize::Md => 14.0,
        ControlSize::Lg => 16.0,
    };
    let base_size = item_typography.size;
    item_typography.size = font_size;
    if base_size > 0.0 {
        item_typography.line_height = font_size * (item_typography.line_height / base_size);
    }
    item_typography
}

fn default_menu_item_icon_size(_metrics: &crate::theme::MetricTokens, size: ControlSize) -> f32 {
    match size {
        ControlSize::Sm => 14.0,
        ControlSize::Md => 16.0,
        ControlSize::Lg => 18.0,
    }
}
