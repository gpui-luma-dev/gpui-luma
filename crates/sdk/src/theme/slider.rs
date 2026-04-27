use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla};

use super::{InteractionLayer, InteractionState, ThemePartUsage, ThemeTokens, ThemeUsage};

#[derive(Clone, Debug)]
pub struct SliderAppearance {
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

pub const SLIDER_THEME_USAGE: ThemeUsage = ThemeUsage {
    component: "Slider",
    parts: &[
        ThemePartUsage {
            part: "track background",
            token: "surface.subtle.background",
            states: &["default", "hovered", "pressed", "focused"],
            appearance_fields: &["SliderAppearance.track_background"],
        },
        ThemePartUsage {
            part: "fill background",
            token: "action.prominent.background",
            states: &["default"],
            appearance_fields: &["SliderAppearance.fill_background", "SliderAppearance.thumb_border"],
        },
        ThemePartUsage {
            part: "fill hover background",
            token: "action.prominent.hover_background",
            states: &["hovered"],
            appearance_fields: &["SliderAppearance.fill_background"],
        },
        ThemePartUsage {
            part: "fill pressed background",
            token: "action.prominent.pressed_background",
            states: &["pressed"],
            appearance_fields: &["SliderAppearance.fill_background"],
        },
        ThemePartUsage {
            part: "thumb background",
            token: "surface.panel.background",
            states: &["default", "hovered", "pressed", "focused"],
            appearance_fields: &["SliderAppearance.thumb_background"],
        },
        ThemePartUsage {
            part: "disabled track",
            token: "state.disabled.background",
            states: &["disabled"],
            appearance_fields: &["SliderAppearance.track_background", "SliderAppearance.thumb_border"],
        },
        ThemePartUsage {
            part: "disabled fill and thumb",
            token: "state.disabled.foreground",
            states: &["disabled"],
            appearance_fields: &["SliderAppearance.fill_background", "SliderAppearance.thumb_background"],
        },
        ThemePartUsage {
            part: "focus ring",
            token: "focus.ring",
            states: &["focused"],
            appearance_fields: &["SliderAppearance.focus_ring"],
        },
    ],
};

impl DefaultSliderTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl SliderTheme for DefaultSliderTheme {
    fn resolve(&self, state: InteractionState) -> SliderAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let elevation = &self.tokens.elevation;
        let fill_background = match state.layer() {
            InteractionLayer::Disabled => palette.state.disabled.foreground,
            InteractionLayer::Pressed => palette.action.prominent.pressed_background,
            InteractionLayer::Hovered => palette.action.prominent.hover_background,
            InteractionLayer::Default => palette.action.prominent.background,
        };

        SliderAppearance {
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
                palette.action.prominent.background
            },
            thumb_shadow: elevation.thumb.to_box_shadows(),
            focus_ring: state.focused.then_some(palette.focus.ring),
            width: 260.0,
            height: 32.0,
            track_height: 8.0,
            thumb_size: 18.0,
            radius: metrics.radius.pill,
        }
    }
}
