use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla};

use super::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemeTokens};

#[derive(Clone, Debug)]
pub struct ContextMenuAppearance {
    pub target_background: Hsla,
    pub target_foreground: Hsla,
    pub target_border: Hsla,
    pub focus_ring: Option<Hsla>,
    pub target_typography: LumaTextStyle,
    pub target_radius: f32,
    pub target_padding_x: f32,
    pub target_padding_y: f32,
    pub target_min_width: f32,
    pub menu_background: Hsla,
    pub menu_border: Hsla,
    pub menu_shadow: Vec<BoxShadow>,
    pub menu_radius: f32,
    pub menu_padding: f32,
    pub menu_min_width: f32,
    pub item_foreground: Hsla,
    pub item_disabled_foreground: Hsla,
    pub item_hover_background: Hsla,
    pub item_typography: LumaTextStyle,
    pub item_height: f32,
    pub item_padding_x: f32,
    pub item_gap: f32,
    pub item_icon_size: f32,
    pub item_radius: f32,
    pub submenu_offset_x: f32,
}

pub trait ContextMenuTheme: Send + Sync {
    fn resolve(&self, state: InteractionState) -> ContextMenuAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultContextMenuTheme {
    tokens: ThemeTokens,
}

pub fn default_context_menu_theme() -> Arc<dyn ContextMenuTheme> {
    static THEME: OnceLock<Arc<dyn ContextMenuTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultContextMenuTheme::default())).clone()
}

impl DefaultContextMenuTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl ContextMenuTheme for DefaultContextMenuTheme {
    fn resolve(&self, state: InteractionState) -> ContextMenuAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let elevation = &self.tokens.elevation;
        let size = ControlSize::Md;

        let target_background = match state.layer() {
            InteractionLayer::Disabled => palette.state.disabled.background,
            InteractionLayer::Pressed => palette.action.secondary.pressed_background,
            InteractionLayer::Hovered => palette.action.secondary.hover_background,
            InteractionLayer::Default => palette.action.secondary.background,
        };
        let target_foreground = if state.disabled {
            palette.state.disabled.foreground
        } else {
            palette.action.secondary.foreground
        };

        ContextMenuAppearance {
            target_background,
            target_foreground,
            target_border: palette.border.default,
            focus_ring: state.focused.then_some(palette.focus.ring),
            target_typography: typography.text.label,
            target_radius: metrics.radius(size),
            target_padding_x: metrics.padding_x(size),
            target_padding_y: metrics.padding_y(size),
            target_min_width: 200.0,
            menu_background: palette.surface.floating.background,
            menu_border: palette.surface.floating.border,
            menu_shadow: elevation.menu.to_box_shadows(),
            menu_radius: metrics.radius.lg,
            menu_padding: metrics.padding_y(size) * 0.5,
            menu_min_width: 180.0,
            item_foreground: palette.surface.floating.foreground,
            item_disabled_foreground: palette.state.disabled.foreground,
            item_hover_background: palette.state.hover.background,
            item_typography: typography.text.label,
            item_height: metrics.control_height(size) * 0.9,
            item_padding_x: metrics.padding_x(size) * 0.75,
            item_gap: metrics.gap(size),
            item_icon_size: metrics.control_height(size) * 0.44,
            item_radius: metrics.radius.sm,
            submenu_offset_x: metrics.gap(size) * 0.5,
        }
    }
}
