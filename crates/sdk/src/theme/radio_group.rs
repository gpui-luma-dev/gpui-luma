use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use super::{ControlSize, InteractionLayer, InteractionState, ThemeTokens};

#[derive(Clone, Copy, Debug)]
pub struct RadioGroupItemAppearance {
    pub control_background: Option<Hsla>,
    pub control_border: Option<Hsla>,
    pub indicator_background: Hsla,
    pub indicator_border: Hsla,
    pub dot_color: Hsla,
    pub label_color: Hsla,
    pub focus_ring: Option<Hsla>,
    pub control_radius: f32,
    pub control_padding_x: f32,
    pub control_padding_y: f32,
    pub indicator_size: f32,
    pub dot_size: f32,
    pub gap: f32,
    pub height: f32,
}

pub trait RadioGroupTheme: Send + Sync {
    fn resolve_item(&self, selected: bool, state: InteractionState) -> RadioGroupItemAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultRadioGroupTheme {
    tokens: ThemeTokens,
}

pub fn default_radio_group_theme() -> Arc<dyn RadioGroupTheme> {
    static THEME: OnceLock<Arc<dyn RadioGroupTheme>> = OnceLock::new();

    THEME
        .get_or_init(|| Arc::new(DefaultRadioGroupTheme::default()))
        .clone()
}

impl DefaultRadioGroupTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl RadioGroupTheme for DefaultRadioGroupTheme {
    fn resolve_item(&self, selected: bool, state: InteractionState) -> RadioGroupItemAppearance {
        let colors = &self.tokens.colors;
        let metrics = &self.tokens.metrics;
        let size = ControlSize::Md;
        let layer = state.layer();

        let indicator_background = match layer {
            InteractionLayer::Disabled => colors.surface_disabled,
            InteractionLayer::Pressed => colors.surface_pressed,
            InteractionLayer::Hovered => colors.surface_hover,
            InteractionLayer::Default => colors.surface,
        };

        let selected_color = match layer {
            InteractionLayer::Disabled => colors.text_disabled,
            InteractionLayer::Pressed => colors.selected_pressed,
            InteractionLayer::Hovered => colors.selected_hover,
            InteractionLayer::Default => colors.selected,
        };

        RadioGroupItemAppearance {
            control_background: None,
            control_border: None,
            indicator_background,
            indicator_border: if selected {
                selected_color
            } else {
                colors.border
            },
            dot_color: selected_color,
            label_color: if state.disabled {
                colors.text_disabled
            } else {
                colors.text
            },
            focus_ring: state.focused.then_some(colors.focus_ring),
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
