use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::controls::selector_panel::{SelectorItemsPanelAppearance, default_selector_items_panel_appearance};
use crate::theme::{
    ControlSize, InteractionLayer, InteractionState, LumaTextStyle, MetricTokens, StandardBoxScale, ThemeTokens,
};

#[derive(Clone, Debug)]
pub struct SelectorPalette {
    pub trigger_background: Hsla,
    pub trigger_foreground: Hsla,
    pub trigger_border: Hsla,
    pub focus_ring: Option<Hsla>,
    pub trigger_typography: LumaTextStyle,
    pub items_panel: SelectorItemsPanelAppearance,
}

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
    fn resolve(&self, state: InteractionState) -> SelectorPalette;

    fn metrics(&self) -> MetricTokens;

    fn resolve_appearance(&self, state: InteractionState, scale: &StandardBoxScale) -> SelectorAppearance {
        compose_selector_appearance(&self.resolve(state), scale)
    }
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
    fn resolve(&self, state: InteractionState) -> SelectorPalette {
        let palette = &self.tokens.palette;
        let typography = &self.tokens.typography;
        let size = ControlSize::Md;

        let trigger_background = match state.layer() {
            InteractionLayer::Disabled => palette.state.disabled.background,
            InteractionLayer::Pressed => palette.state.pressed.background,
            InteractionLayer::Hovered => palette.state.hover.background,
            InteractionLayer::Default => palette.app.background,
        };
        let trigger_foreground = if state.disabled {
            palette.state.disabled.foreground
        } else {
            palette.app.foreground
        };

        SelectorPalette {
            trigger_background,
            trigger_foreground,
            trigger_border: palette.border.default,
            focus_ring: state.focused.then_some(palette.focus.ring),
            trigger_typography: typography.text.label,
            items_panel: default_selector_items_panel_appearance(&self.tokens, size),
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }
}

pub(crate) fn compose_selector_appearance(palette: &SelectorPalette, scale: &StandardBoxScale) -> SelectorAppearance {
    SelectorAppearance {
        trigger_background: palette.trigger_background,
        trigger_foreground: palette.trigger_foreground,
        trigger_border: palette.trigger_border,
        focus_ring: palette.focus_ring,
        trigger_typography: palette.trigger_typography,
        trigger_radius: scale.radius,
        trigger_padding_x: scale.padding_x,
        trigger_padding_y: scale.padding_y,
        trigger_gap: scale.gap,
        trigger_height: scale.height,
        trigger_icon_size: scale.height / 3.0,
        menu_offset_y: scale.gap * 0.5,
        items_panel: palette.items_panel.clone(),
    }
}
