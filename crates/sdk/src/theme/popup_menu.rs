use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla};

use super::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemeTokens};

#[derive(Clone, Debug)]
pub struct PopupMenuAppearance {
    pub trigger_background: Hsla,
    pub trigger_foreground: Hsla,
    pub trigger_border: Hsla,
    pub focus_ring: Option<Hsla>,
    pub trigger_typography: LumaTextStyle,
    pub trigger_radius: f32,
    pub trigger_padding_x: f32,
    pub trigger_padding_y: f32,
    pub trigger_gap: f32,
    pub trigger_height: f32,
    pub trigger_icon_size: f32,
    pub menu_background: Hsla,
    pub menu_border: Hsla,
    pub menu_shadow: Vec<BoxShadow>,
    pub menu_radius: f32,
    pub menu_padding: f32,
    pub menu_offset_y: f32,
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

pub trait PopupMenuTheme: Send + Sync {
    fn resolve(&self, state: InteractionState) -> PopupMenuAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultPopupMenuTheme {
    tokens: ThemeTokens,
}

pub fn default_popup_menu_theme() -> Arc<dyn PopupMenuTheme> {
    static THEME: OnceLock<Arc<dyn PopupMenuTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultPopupMenuTheme::default())).clone()
}

impl DefaultPopupMenuTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl PopupMenuTheme for DefaultPopupMenuTheme {
    fn resolve(&self, state: InteractionState) -> PopupMenuAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let elevation = &self.tokens.elevation;
        let size = ControlSize::Md;

        let trigger_background = match state.layer() {
            InteractionLayer::Disabled => palette.state.disabled.background,
            InteractionLayer::Pressed => palette.action.secondary.pressed_background,
            InteractionLayer::Hovered => palette.action.secondary.hover_background,
            InteractionLayer::Default => palette.action.secondary.background,
        };
        let trigger_foreground = if state.disabled {
            palette.state.disabled.foreground
        } else {
            palette.action.secondary.foreground
        };

        PopupMenuAppearance {
            trigger_background,
            trigger_foreground,
            trigger_border: palette.border.default,
            focus_ring: state.focused.then_some(palette.focus.ring),
            trigger_typography: typography.text.label,
            trigger_radius: metrics.radius(size),
            trigger_padding_x: metrics.padding_x(size),
            trigger_padding_y: metrics.padding_y(size),
            trigger_gap: metrics.gap(size),
            trigger_height: metrics.control_height(size),
            trigger_icon_size: metrics.control_height(size) * 0.44,
            menu_background: palette.surface.floating.background,
            menu_border: palette.surface.floating.border,
            menu_shadow: elevation.menu.to_box_shadows(),
            menu_radius: metrics.radius.lg,
            menu_padding: metrics.padding_y(size) * 0.5,
            menu_offset_y: metrics.gap(size) * 0.5,
            menu_min_width: 160.0,
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
