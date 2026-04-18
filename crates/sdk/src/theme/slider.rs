use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use super::{InteractionLayer, InteractionState, ThemeTokens};

#[derive(Clone, Copy, Debug)]
pub struct SliderAppearance {
    pub track_background: Hsla,
    pub fill_background: Hsla,
    pub thumb_background: Hsla,
    pub focus_ring: Option<Hsla>,
    pub width: f32,
    pub height: f32,
    pub track_height: f32,
    pub thumb_size: f32,
    pub radius: f32,
}

pub trait SliderTheme: Send + Sync {
    fn resolve(&self, state: InteractionState) -> SliderAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultSliderTheme {
    tokens: ThemeTokens,
}

pub fn default_slider_theme() -> Arc<dyn SliderTheme> {
    static THEME: OnceLock<Arc<dyn SliderTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultSliderTheme::default())).clone()
}

impl DefaultSliderTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl SliderTheme for DefaultSliderTheme {
    fn resolve(&self, state: InteractionState) -> SliderAppearance {
        let colors = &self.tokens.colors;
        let fill_background = match state.layer() {
            InteractionLayer::Disabled => colors.surface_disabled,
            InteractionLayer::Pressed => colors.selected_pressed,
            InteractionLayer::Hovered => colors.selected_hover,
            InteractionLayer::Default => colors.selected,
        };

        SliderAppearance {
            track_background: if state.disabled {
                colors.surface_disabled
            } else {
                colors.surface_pressed
            },
            fill_background,
            thumb_background: if state.disabled {
                colors.text_disabled
            } else {
                colors.text_inverse
            },
            focus_ring: state.focused.then_some(colors.focus_ring),
            width: 260.0,
            height: 32.0,
            track_height: 8.0,
            thumb_size: 18.0,
            radius: 999.0,
        }
    }
}
