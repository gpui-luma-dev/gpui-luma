use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla};

use super::SliderThumbSize;
use crate::theme::{ControlSize, InteractionLayer, InteractionState, ThemeTokens};

#[derive(Clone, Debug)]
pub struct SliderLook {
    pub track_background: Hsla,
    pub fill_background: Hsla,
    pub thumb_background: Hsla,
    pub thumb_border: Hsla,
    pub thumb_shadow: Vec<BoxShadow>,
    pub focus_ring: Option<Hsla>,
    pub width: f32,
    pub height: f32,
    pub track_height: f32,
    pub thumb_size: f32,
    pub radius: f32,
}

pub trait SliderTheme: Send + Sync {
    fn resolve(&self, size: ControlSize, thumb_size: Option<SliderThumbSize>, state: InteractionState) -> SliderLook;
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
    fn resolve(&self, size: ControlSize, thumb_size: Option<SliderThumbSize>, state: InteractionState) -> SliderLook {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let elevation = &self.tokens.elevation;
        let selected = palette.state.selected;
        let fill_background = match state.layer() {
            InteractionLayer::Disabled => palette.state.disabled.foreground,
            InteractionLayer::Pressed => palette.state.pressed.background,
            InteractionLayer::Hovered => selected.background,
            InteractionLayer::Default => selected.background,
        };

        SliderLook {
            track_background: if state.disabled {
                palette.state.disabled.background
            } else {
                palette.surface.subtle.background
            },
            fill_background,
            thumb_background: if state.disabled {
                palette.state.disabled.foreground
            } else {
                palette.surface.panel.background
            },
            thumb_border: if state.disabled {
                palette.state.disabled.background
            } else {
                selected.background
            },
            thumb_shadow: elevation.thumb.to_box_shadows(),
            focus_ring: state.focused.then_some(palette.focus.ring),
            width: 260.0,
            height: slider_height(size),
            track_height: slider_track_height(size),
            thumb_size: slider_thumb_size(thumb_size.unwrap_or(size.into())),
            radius: metrics.radius.pill,
        }
    }
}

fn slider_height(size: ControlSize) -> f32 {
    match size {
        ControlSize::Sm => 24.0,
        ControlSize::Md => 32.0,
        ControlSize::Lg => 40.0,
    }
}

fn slider_track_height(size: ControlSize) -> f32 {
    match size {
        ControlSize::Sm => 4.0,
        ControlSize::Md => 6.0,
        ControlSize::Lg => 8.0,
    }
}

pub fn slider_thumb_size(size: SliderThumbSize) -> f32 {
    match size {
        SliderThumbSize::Sm => 12.0,
        SliderThumbSize::Md => 16.0,
        SliderThumbSize::Lg => 20.0,
    }
}
