use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::controls::selector_panel::{SelectorItemsPanelAppearance, default_selector_items_panel_appearance};
use crate::theme::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemeTokens};

#[derive(Clone, Debug)]
pub struct SelectorAppearance {
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
    pub menu_offset_y: f32,
    pub items_panel: SelectorItemsPanelAppearance,
}

pub trait SelectorTheme: Send + Sync {
    fn resolve(&self, state: InteractionState) -> SelectorAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultSelectorTheme {
    tokens: ThemeTokens,
}

pub fn default_selector_theme() -> Arc<dyn SelectorTheme> {
    static THEME: OnceLock<Arc<dyn SelectorTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultSelectorTheme::default())).clone()
}

impl DefaultSelectorTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl SelectorTheme for DefaultSelectorTheme {
    fn resolve(&self, state: InteractionState) -> SelectorAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let size = ControlSize::Md;

        let trigger_background = match state.layer() {
            InteractionLayer::Disabled => palette.state.disabled.background,
            InteractionLayer::Pressed => palette.action.ghost.pressed_background,
            InteractionLayer::Hovered => palette.action.ghost.hover_background,
            InteractionLayer::Default => palette.action.ghost.background,
        };
        let trigger_foreground = if state.disabled {
            palette.state.disabled.foreground
        } else {
            palette.action.ghost.foreground
        };

        SelectorAppearance {
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
            trigger_icon_size: 12.0,
            menu_offset_y: metrics.gap(size) * 0.5,
            items_panel: default_selector_items_panel_appearance(&self.tokens, size),
        }
    }
}
