use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use super::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemePartUsage, ThemeTokens, ThemeUsage};

#[derive(Clone, Copy, Debug)]
pub struct RadioGroupItemAppearance {
    pub control_background: Option<Hsla>,
    pub control_border: Option<Hsla>,
    pub indicator_background: Hsla,
    pub indicator_border: Hsla,
    pub dot_color: Hsla,
    pub label_color: Hsla,
    pub focus_ring: Option<Hsla>,
    pub label_typography: LumaTextStyle,
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

    THEME.get_or_init(|| Arc::new(DefaultRadioGroupTheme::default())).clone()
}

pub const RADIO_GROUP_THEME_USAGE: ThemeUsage = ThemeUsage {
    component: "Radio Group",
    parts: &[
        ThemePartUsage {
            part: "unselected indicator background",
            token: "form.input.background",
            states: &["unselected"],
            appearance_fields: &["RadioGroupItemAppearance.indicator_background"],
        },
        ThemePartUsage {
            part: "unselected indicator hover background",
            token: "state.hover.background",
            states: &["unselected hovered"],
            appearance_fields: &["RadioGroupItemAppearance.indicator_background"],
        },
        ThemePartUsage {
            part: "unselected indicator pressed background",
            token: "state.pressed.background",
            states: &["unselected pressed"],
            appearance_fields: &["RadioGroupItemAppearance.indicator_background"],
        },
        ThemePartUsage {
            part: "unselected indicator border",
            token: "form.input.border",
            states: &["unselected", "unselected hovered", "unselected pressed"],
            appearance_fields: &["RadioGroupItemAppearance.indicator_border"],
        },
        ThemePartUsage {
            part: "selected indicator background",
            token: "action.prominent.background",
            states: &["selected"],
            appearance_fields: &[
                "RadioGroupItemAppearance.indicator_background",
                "RadioGroupItemAppearance.indicator_border",
            ],
        },
        ThemePartUsage {
            part: "selected indicator hover background",
            token: "action.prominent.hover_background",
            states: &["selected hovered"],
            appearance_fields: &[
                "RadioGroupItemAppearance.indicator_background",
                "RadioGroupItemAppearance.indicator_border",
            ],
        },
        ThemePartUsage {
            part: "selected indicator pressed background",
            token: "action.prominent.pressed_background",
            states: &["selected pressed"],
            appearance_fields: &[
                "RadioGroupItemAppearance.indicator_background",
                "RadioGroupItemAppearance.indicator_border",
            ],
        },
        ThemePartUsage {
            part: "selected dot",
            token: "action.prominent.foreground",
            states: &["selected"],
            appearance_fields: &["RadioGroupItemAppearance.dot_color"],
        },
        ThemePartUsage {
            part: "label",
            token: "app.foreground",
            states: &["default", "selected"],
            appearance_fields: &["RadioGroupItemAppearance.label_color"],
        },
        ThemePartUsage {
            part: "disabled fill",
            token: "state.disabled.background",
            states: &["disabled"],
            appearance_fields: &["RadioGroupItemAppearance.indicator_background"],
        },
        ThemePartUsage {
            part: "disabled foreground",
            token: "state.disabled.foreground",
            states: &["disabled"],
            appearance_fields: &[
                "RadioGroupItemAppearance.indicator_border",
                "RadioGroupItemAppearance.dot_color",
                "RadioGroupItemAppearance.label_color",
            ],
        },
        ThemePartUsage {
            part: "focus ring",
            token: "focus.ring",
            states: &["focused"],
            appearance_fields: &["RadioGroupItemAppearance.focus_ring"],
        },
    ],
};

impl DefaultRadioGroupTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl RadioGroupTheme for DefaultRadioGroupTheme {
    fn resolve_item(&self, selected: bool, state: InteractionState) -> RadioGroupItemAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let size = ControlSize::Md;
        let layer = state.layer();

        let indicator_background = match (selected, layer) {
            (_, InteractionLayer::Disabled) => palette.state.disabled.background,
            (true, InteractionLayer::Pressed) => palette.action.prominent.pressed_background,
            (true, InteractionLayer::Hovered) => palette.action.prominent.hover_background,
            (true, InteractionLayer::Default) => palette.action.prominent.background,
            (false, InteractionLayer::Pressed) => palette.state.pressed.background,
            (false, InteractionLayer::Hovered) => palette.state.hover.background,
            (false, InteractionLayer::Default) => palette.form.input.background,
        };

        let selected_color = match layer {
            InteractionLayer::Disabled => palette.state.disabled.foreground,
            InteractionLayer::Pressed => palette.action.prominent.pressed_background,
            InteractionLayer::Hovered => palette.action.prominent.hover_background,
            InteractionLayer::Default => palette.action.prominent.background,
        };

        RadioGroupItemAppearance {
            control_background: None,
            control_border: None,
            indicator_background,
            indicator_border: if selected {
                selected_color
            } else {
                palette.form.input.border
            },
            dot_color: if state.disabled {
                palette.state.disabled.foreground
            } else {
                palette.action.prominent.foreground
            },
            label_color: if state.disabled {
                palette.state.disabled.foreground
            } else {
                palette.app.foreground
            },
            focus_ring: state.focused.then_some(palette.focus.ring),
            label_typography: typography.text.label,
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
