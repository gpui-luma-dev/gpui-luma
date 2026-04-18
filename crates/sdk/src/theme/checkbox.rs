use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use super::{ControlSize, InteractionLayer, InteractionState, ThemeTokens};

#[derive(Clone, Copy, Debug)]
pub struct CheckboxAppearance {
    pub control_background: Option<Hsla>,
    pub control_border: Option<Hsla>,
    pub indicator_background: Hsla,
    pub indicator_border: Hsla,
    pub checkmark_color: Hsla,
    pub label_color: Hsla,
    pub focus_ring: Option<Hsla>,
    pub control_radius: f32,
    pub control_padding_x: f32,
    pub control_padding_y: f32,
    pub indicator_radius: f32,
    pub indicator_size: f32,
    pub checkmark_size: f32,
    pub gap: f32,
    pub height: f32,
}

pub trait CheckboxTheme: Send + Sync {
    fn resolve(&self, checked: bool, state: InteractionState) -> CheckboxAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultCheckboxTheme {
    tokens: ThemeTokens,
}

pub fn default_checkbox_theme() -> Arc<dyn CheckboxTheme> {
    static THEME: OnceLock<Arc<dyn CheckboxTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultCheckboxTheme::default())).clone()
}

impl DefaultCheckboxTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl CheckboxTheme for DefaultCheckboxTheme {
    fn resolve(&self, checked: bool, state: InteractionState) -> CheckboxAppearance {
        let colors = &self.tokens.colors;
        let metrics = &self.tokens.metrics;
        let size = ControlSize::Md;
        let layer = state.layer();

        let indicator_background = match (checked, layer) {
            (_, InteractionLayer::Disabled) => colors.surface_disabled,
            (true, InteractionLayer::Pressed) => colors.selected_pressed,
            (true, InteractionLayer::Hovered) => colors.selected_hover,
            (true, InteractionLayer::Default) => colors.selected,
            (false, InteractionLayer::Pressed) => colors.surface_pressed,
            (false, InteractionLayer::Hovered) => colors.surface_hover,
            (false, InteractionLayer::Default) => colors.surface,
        };

        let label_color = if state.disabled {
            colors.text_disabled
        } else {
            colors.text
        };

        CheckboxAppearance {
            control_background: None,
            control_border: None,
            indicator_background,
            indicator_border: if checked && !state.disabled {
                indicator_background
            } else {
                colors.border
            },
            checkmark_color: if state.disabled {
                colors.text_disabled
            } else {
                colors.text_inverse
            },
            label_color,
            focus_ring: state.focused.then_some(colors.focus_ring),
            control_radius: metrics.radius(size),
            control_padding_x: 0.0,
            control_padding_y: 0.0,
            indicator_radius: metrics.radius(size) * 0.5,
            indicator_size: metrics.control_height(size) * 0.5,
            checkmark_size: metrics.control_height(size) * 0.42,
            gap: metrics.gap(size),
            height: metrics.control_height(size),
        }
    }
}
