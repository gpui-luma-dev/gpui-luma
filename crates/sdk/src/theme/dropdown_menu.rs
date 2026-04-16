use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use super::{ControlSize, InteractionLayer, InteractionState, ThemeTokens};

#[derive(Clone, Copy, Debug)]
pub struct DropdownMenuAppearance {
    pub trigger_background: Hsla,
    pub trigger_foreground: Hsla,
    pub trigger_border: Hsla,
    pub focus_ring: Option<Hsla>,
    pub trigger_radius: f32,
    pub trigger_padding_x: f32,
    pub trigger_padding_y: f32,
    pub trigger_gap: f32,
    pub trigger_height: f32,
    pub trigger_icon_size: f32,
    pub menu_background: Hsla,
    pub menu_border: Hsla,
    pub menu_radius: f32,
    pub menu_padding: f32,
    pub menu_offset_y: f32,
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

pub trait DropdownMenuTheme: Send + Sync {
    fn resolve(&self, state: InteractionState) -> DropdownMenuAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultDropdownMenuTheme {
    tokens: ThemeTokens,
}

pub fn default_dropdown_menu_theme() -> Arc<dyn DropdownMenuTheme> {
    static THEME: OnceLock<Arc<dyn DropdownMenuTheme>> = OnceLock::new();

    THEME
        .get_or_init(|| Arc::new(DefaultDropdownMenuTheme::default()))
        .clone()
}

impl DefaultDropdownMenuTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl DropdownMenuTheme for DefaultDropdownMenuTheme {
    fn resolve(&self, state: InteractionState) -> DropdownMenuAppearance {
        let colors = &self.tokens.colors;
        let metrics = &self.tokens.metrics;
        let size = ControlSize::Md;

        let trigger_background = match state.layer() {
            InteractionLayer::Disabled => colors.surface_disabled,
            InteractionLayer::Pressed => colors.surface_pressed,
            InteractionLayer::Hovered => colors.surface_hover,
            InteractionLayer::Default => colors.surface,
        };
        let trigger_foreground = if state.disabled {
            colors.text_disabled
        } else {
            colors.text
        };

        DropdownMenuAppearance {
            trigger_background,
            trigger_foreground,
            trigger_border: colors.border,
            focus_ring: state.focused.then_some(colors.focus_ring),
            trigger_radius: metrics.radius(size),
            trigger_padding_x: metrics.padding_x(size),
            trigger_padding_y: metrics.padding_y(size),
            trigger_gap: metrics.gap(size),
            trigger_height: metrics.control_height(size),
            trigger_icon_size: metrics.control_height(size) * 0.44,
            menu_background: colors.surface,
            menu_border: colors.border,
            menu_radius: metrics.radius(size),
            menu_padding: metrics.padding_y(size) * 0.5,
            menu_offset_y: metrics.gap(size) * 0.5,
            menu_min_width: 160.0,
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
