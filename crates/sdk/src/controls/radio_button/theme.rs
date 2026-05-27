use std::sync::{Arc, OnceLock};

use gpui::{Hsla, SharedString};

use crate::theme::adorner::{AdornerPlacement, AdornerSpec, FocusRingAdornerSpec};
use crate::theme::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemeTokens};

#[derive(Clone, Debug)]
pub struct RadioButtonAppearance {
    pub control_background: Option<Hsla>,
    pub control_border: Option<Hsla>,
    pub indicator_background: Hsla,
    pub indicator_border: Hsla,
    pub dot_color: Hsla,
    pub label_color: Hsla,
    pub adorner: Option<AdornerSpec>,
    pub label_typography: LumaTextStyle,
    pub label_font_family: SharedString,
    pub control_radius: f32,
    pub control_padding_x: f32,
    pub control_padding_y: f32,
    pub indicator_size: f32,
    pub dot_size: f32,
    pub gap: f32,
    pub height: f32,
}

pub trait RadioButtonTheme: Send + Sync {
    fn resolve(&self, checked: bool, state: InteractionState) -> RadioButtonAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultRadioButtonTheme {
    tokens: ThemeTokens,
}

pub fn default_radio_button_theme() -> Arc<dyn RadioButtonTheme> {
    static THEME: OnceLock<Arc<dyn RadioButtonTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultRadioButtonTheme::default())).clone()
}

impl DefaultRadioButtonTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl RadioButtonTheme for DefaultRadioButtonTheme {
    fn resolve(&self, checked: bool, state: InteractionState) -> RadioButtonAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let size = ControlSize::Md;
        let layer = state.layer();
        let checked_action = palette.action.prominent;

        let indicator_background = match layer {
            InteractionLayer::Disabled => palette.state.disabled.background,
            InteractionLayer::Pressed => palette.state.pressed.background,
            InteractionLayer::Hovered => palette.state.hover.background,
            InteractionLayer::Default => palette.form.input.background,
        };

        let selected_color = match layer {
            InteractionLayer::Disabled => palette.state.disabled.foreground,
            InteractionLayer::Pressed => checked_action.pressed_background,
            InteractionLayer::Hovered => checked_action.hover_background,
            InteractionLayer::Default => checked_action.background,
        };

        let adorner = if state.focused {
            Some(AdornerSpec::FocusRing(FocusRingAdornerSpec {
                color: palette.focus.ring,
                placement: AdornerPlacement::Oversize,
                distance: metrics.border_width.default + metrics.focus.width,
                width: metrics.focus.width,
            }))
        } else {
            None
        };

        RadioButtonAppearance {
            control_background: None,
            control_border: None,
            indicator_background,
            indicator_border: if checked && !state.disabled {
                selected_color
            } else {
                palette.form.input.border
            },
            dot_color: if state.disabled {
                palette.state.disabled.foreground
            } else if checked {
                checked_action.foreground
            } else {
                palette.app.foreground
            },
            label_color: if state.disabled {
                palette.state.disabled.foreground
            } else {
                palette.app.foreground
            },
            adorner,
            label_typography: typography.text.label,
            label_font_family: typography.font.sans.family.clone().into(),
            control_radius: metrics.radius(size),
            control_padding_x: 0.0,
            control_padding_y: 0.0,
            indicator_size: metrics.control_height(size) * 0.5,
            dot_size: metrics.control_height(size) * 0.24,
            gap: metrics.gap(size),
            height: metrics.control_height(size),
        }
    }
}
