use std::sync::{Arc, OnceLock};

use gpui::{Hsla, hsla};

use crate::controls::scrollbar::ScrollbarOrientation;

use super::{InteractionLayer, InteractionState, ThemeTokens};

#[derive(Clone, Copy, Debug)]
pub struct ScrollbarAppearance {
    pub track_background: Hsla,
    pub thumb_background: Hsla,
    pub focus_ring: Option<Hsla>,
    pub length: f32,
    pub thickness: f32,
    pub track_thickness: f32,
    pub thumb_thickness: f32,
    pub min_thumb_length: f32,
    pub radius: f32,
}

pub trait ScrollbarTheme: Send + Sync {
    fn resolve(&self, state: InteractionState, orientation: ScrollbarOrientation) -> ScrollbarAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultScrollbarTheme {
    tokens: ThemeTokens,
}

pub fn default_scrollbar_theme() -> Arc<dyn ScrollbarTheme> {
    static THEME: OnceLock<Arc<dyn ScrollbarTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultScrollbarTheme::default())).clone()
}

impl DefaultScrollbarTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl ScrollbarTheme for DefaultScrollbarTheme {
    fn resolve(&self, state: InteractionState, orientation: ScrollbarOrientation) -> ScrollbarAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let thumb_background = match state.layer() {
            InteractionLayer::Disabled => palette.state.disabled.foreground,
            InteractionLayer::Pressed => palette.state.pressed.background,
            InteractionLayer::Hovered => palette.state.hover.background,
            InteractionLayer::Default => palette.border.default,
        };

        let length = match orientation {
            ScrollbarOrientation::Horizontal => 260.0,
            ScrollbarOrientation::Vertical => 180.0,
        };

        ScrollbarAppearance {
            track_background: if state.disabled {
                palette.state.disabled.background
            } else {
                hsla(0.0, 0.0, 0.0, 0.0)
            },
            thumb_background,
            focus_ring: state.focused.then_some(palette.focus.ring),
            length,
            thickness: 12.0,
            track_thickness: 4.0,
            thumb_thickness: 8.0,
            min_thumb_length: 28.0,
            radius: metrics.radius.pill,
        }
    }
}
