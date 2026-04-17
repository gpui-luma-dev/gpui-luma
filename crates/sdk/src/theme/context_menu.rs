use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use super::{ControlSize, InteractionLayer, InteractionState, ThemeTokens};

#[derive(Clone, Copy, Debug)]
pub struct ContextMenuAppearance {
    pub target_background: Hsla,
    pub target_foreground: Hsla,
    pub target_border: Hsla,
    pub focus_ring: Option<Hsla>,
    pub target_radius: f32,
    pub target_padding_x: f32,
    pub target_padding_y: f32,
    pub target_min_width: f32,
    pub menu_background: Hsla,
    pub menu_border: Hsla,
    pub menu_radius: f32,
    pub menu_padding: f32,
    pub menu_min_width: f32,
    pub item_foreground: Hsla,
    pub item_disabled_foreground: Hsla,
    pub item_hover_background: Hsla,
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

    THEME
        .get_or_init(|| Arc::new(DefaultContextMenuTheme::default()))
        .clone()
}

impl DefaultContextMenuTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl ContextMenuTheme for DefaultContextMenuTheme {
    fn resolve(&self, state: InteractionState) -> ContextMenuAppearance {
        let colors = &self.tokens.colors;
        let metrics = &self.tokens.metrics;
        let size = ControlSize::Md;

        let target_background = match state.layer() {
            InteractionLayer::Disabled => colors.surface_disabled,
            InteractionLayer::Pressed => colors.surface_pressed,
            InteractionLayer::Hovered => colors.surface_hover,
            InteractionLayer::Default => colors.surface,
        };
        let target_foreground = if state.disabled {
            colors.text_disabled
        } else {
            colors.text
        };

        ContextMenuAppearance {
            target_background,
            target_foreground,
            target_border: colors.border,
            focus_ring: state.focused.then_some(colors.focus_ring),
            target_radius: metrics.radius(size),
            target_padding_x: metrics.padding_x(size),
            target_padding_y: metrics.padding_y(size),
            target_min_width: 200.0,
            menu_background: colors.surface,
            menu_border: colors.border,
            menu_radius: metrics.radius(size),
            menu_padding: metrics.padding_y(size) * 0.5,
            menu_min_width: 180.0,
            item_foreground: colors.text,
            item_disabled_foreground: colors.text_disabled,
            item_hover_background: colors.surface_hover,
            item_height: metrics.control_height(size) * 0.9,
            item_padding_x: metrics.padding_x(size) * 0.75,
            item_gap: metrics.gap(size),
            item_icon_size: metrics.control_height(size) * 0.44,
            item_radius: metrics.radius(size) * 0.75,
            submenu_offset_x: metrics.gap(size) * 0.5,
        }
    }
}
