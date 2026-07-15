use std::sync::{Arc, OnceLock};

use gpui::{Hsla, hsla};

use crate::controls::scrollbar::{ScrollbarOrientation, ScrollbarStyle};

use crate::theme::{ControlSize, InteractionLayer, InteractionState, ThemeTokens};

#[derive(Clone, Copy, Debug)]
pub struct ScrollbarLook {
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
    fn resolve(
        &self,
        state: InteractionState,
        orientation: ScrollbarOrientation,
        size: ControlSize,
        style: ScrollbarStyle,
    ) -> ScrollbarLook;
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
    fn resolve(
        &self,
        state: InteractionState,
        orientation: ScrollbarOrientation,
        size: ControlSize,
        style: ScrollbarStyle,
    ) -> ScrollbarLook {
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

        ScrollbarLook {
            track_background: if state.disabled {
                palette.state.disabled.background
            } else if style == ScrollbarStyle::Soft {
                palette.surface.subtle.background
            } else {
                hsla(0.0, 0.0, 0.0, 0.0)
            },
            thumb_background,
            focus_ring: state.focused.then_some(palette.focus.ring),
            length,
            thickness: scrollbar_thickness(size),
            track_thickness: scrollbar_track_thickness(size),
            thumb_thickness: scrollbar_thumb_thickness(size),
            min_thumb_length: scrollbar_min_thumb_length(size),
            radius: metrics.radius.pill,
        }
    }
}

fn scrollbar_thickness(size: ControlSize) -> f32 {
    match size {
        ControlSize::Sm => 10.0,
        ControlSize::Md => 12.0,
        ControlSize::Lg => 14.0,
    }
}

fn scrollbar_track_thickness(size: ControlSize) -> f32 {
    match size {
        ControlSize::Sm => 3.0,
        ControlSize::Md => 4.0,
        ControlSize::Lg => 5.0,
    }
}

fn scrollbar_thumb_thickness(size: ControlSize) -> f32 {
    match size {
        ControlSize::Sm => 6.0,
        ControlSize::Md => 8.0,
        ControlSize::Lg => 10.0,
    }
}

fn scrollbar_min_thumb_length(size: ControlSize) -> f32 {
    match size {
        ControlSize::Sm => 24.0,
        ControlSize::Md => 28.0,
        ControlSize::Lg => 32.0,
    }
}
