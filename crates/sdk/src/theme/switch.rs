use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use super::{ControlSize, InteractionLayer, InteractionState, ThemeTokens};

#[derive(Clone, Copy, Debug)]
pub struct SwitchAppearance {
    pub track_background: Hsla,
    pub track_border: Hsla,
    pub thumb_background: Hsla,
    pub thumb_border: Hsla,
    pub label_color: Hsla,
    pub focus_ring: Option<Hsla>,
    pub width: f32,
    pub height: f32,
    pub thumb_size: f32,
    pub padding: f32,
    pub gap: f32,
    pub radius: f32,
}

pub trait SwitchTheme: Send + Sync {
    fn resolve(&self, on: bool, state: InteractionState) -> SwitchAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultSwitchTheme {
    tokens: ThemeTokens,
}

pub fn default_switch_theme() -> Arc<dyn SwitchTheme> {
    static THEME: OnceLock<Arc<dyn SwitchTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultSwitchTheme::default())).clone()
}

impl DefaultSwitchTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl SwitchTheme for DefaultSwitchTheme {
    fn resolve(&self, on: bool, state: InteractionState) -> SwitchAppearance {
        let colors = &self.tokens.colors;
        let metrics = &self.tokens.metrics;
        let size = ControlSize::Md;
        let layer = state.layer();

        let track_background = match (on, layer) {
            (_, InteractionLayer::Disabled) => colors.surface_disabled,
            (true, _) => colors.text,
            (false, InteractionLayer::Pressed) => colors.surface_pressed,
            (false, InteractionLayer::Hovered) => colors.surface_hover,
            (false, InteractionLayer::Default) => colors.surface,
        };

        SwitchAppearance {
            track_background,
            track_border: if on && !state.disabled {
                track_background
            } else {
                colors.border
            },
            thumb_background: if state.disabled {
                colors.text_disabled
            } else {
                colors.text_inverse
            },
            thumb_border: if state.disabled {
                colors.surface_disabled
            } else {
                colors.border
            },
            label_color: if state.disabled {
                colors.text_disabled
            } else {
                colors.text
            },
            focus_ring: state.focused.then_some(colors.focus_ring),
            width: 42.0,
            height: 22.0,
            thumb_size: metrics.control_height(size) * 0.5,
            padding: 2.0,
            gap: metrics.gap(size),
            radius: 999.0,
        }
    }
}
