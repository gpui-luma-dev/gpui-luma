use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla};

use super::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemeTokens};

#[derive(Clone, Debug)]
pub struct SwitchAppearance {
    pub track_background: Hsla,
    pub track_border: Hsla,
    pub thumb_background: Hsla,
    pub thumb_border: Hsla,
    pub thumb_shadow: Vec<BoxShadow>,
    pub label_color: Hsla,
    pub focus_ring: Option<Hsla>,
    pub label_typography: LumaTextStyle,
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
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let elevation = &self.tokens.elevation;
        let size = ControlSize::Md;
        let layer = state.layer();

        let track_background = match (on, layer) {
            (_, InteractionLayer::Disabled) => palette.state.disabled.background,
            (true, InteractionLayer::Pressed) => palette.action.primary.pressed_background,
            (true, InteractionLayer::Hovered) => palette.action.primary.hover_background,
            (true, InteractionLayer::Default) => palette.action.primary.background,
            (false, InteractionLayer::Pressed) => palette.state.pressed.background,
            (false, InteractionLayer::Hovered) => palette.state.hover.background,
            (false, InteractionLayer::Default) => palette.form.input.background,
        };

        SwitchAppearance {
            track_background,
            track_border: if on && !state.disabled {
                track_background
            } else {
                palette.form.input.border
            },
            thumb_background: if state.disabled {
                palette.state.disabled.foreground
            } else if on {
                palette.action.primary.foreground
            } else {
                palette.surface.panel.background
            },
            thumb_border: if state.disabled {
                palette.state.disabled.background
            } else if on {
                palette.action.primary.foreground
            } else {
                palette.border.default
            },
            thumb_shadow: elevation.thumb.to_box_shadows(),
            label_color: if state.disabled {
                palette.state.disabled.foreground
            } else {
                palette.app.foreground
            },
            focus_ring: state.focused.then_some(palette.focus.ring),
            label_typography: typography.text.label,
            width: 42.0,
            height: 22.0,
            thumb_size: metrics.control_height(size) * 0.5,
            padding: 2.0,
            gap: metrics.gap(size),
            radius: metrics.radius.pill,
        }
    }
}
