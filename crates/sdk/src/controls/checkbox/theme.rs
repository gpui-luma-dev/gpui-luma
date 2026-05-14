use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::adorner::{AdornerPlacement, AdornerSpec, FocusRingAdornerSpec};
use crate::theme::{
    ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemePartUsage, ThemeTokens, ThemeUsage,
};

#[derive(Clone, Debug)]
pub struct CheckboxAppearance {
    pub control_background: Option<Hsla>,
    pub control_border: Option<Hsla>,
    pub indicator_background: Hsla,
    pub indicator_border: Hsla,
    pub checkmark_color: Hsla,
    pub label_color: Hsla,
    pub adorner: Option<AdornerSpec>,
    pub label_typography: LumaTextStyle,
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

pub const CHECKBOX_THEME_USAGE: ThemeUsage = ThemeUsage {
    label: "Checkbox",
    parts: &[
        ThemePartUsage {
            part: "unchecked indicator background",
            token: "form.input.background",
            states: &["unchecked"],
            appearance_fields: &["CheckboxAppearance.indicator_background"],
        },
        ThemePartUsage {
            part: "unchecked indicator hover background",
            token: "state.hover.background",
            states: &["unchecked hovered"],
            appearance_fields: &["CheckboxAppearance.indicator_background"],
        },
        ThemePartUsage {
            part: "unchecked indicator pressed background",
            token: "state.pressed.background",
            states: &["unchecked pressed"],
            appearance_fields: &["CheckboxAppearance.indicator_background"],
        },
        ThemePartUsage {
            part: "unchecked indicator border",
            token: "form.input.border",
            states: &["unchecked", "unchecked hovered", "unchecked pressed", "disabled"],
            appearance_fields: &["CheckboxAppearance.indicator_border"],
        },
        ThemePartUsage {
            part: "checked indicator background",
            token: "action.prominent.background",
            states: &["checked"],
            appearance_fields: &["CheckboxAppearance.indicator_background", "CheckboxAppearance.indicator_border"],
        },
        ThemePartUsage {
            part: "checked indicator hover background",
            token: "action.prominent.hover_background",
            states: &["checked hovered"],
            appearance_fields: &["CheckboxAppearance.indicator_background", "CheckboxAppearance.indicator_border"],
        },
        ThemePartUsage {
            part: "checked indicator pressed background",
            token: "action.prominent.pressed_background",
            states: &["checked pressed"],
            appearance_fields: &["CheckboxAppearance.indicator_background", "CheckboxAppearance.indicator_border"],
        },
        ThemePartUsage {
            part: "checked checkmark",
            token: "action.prominent.foreground",
            states: &["checked"],
            appearance_fields: &["CheckboxAppearance.checkmark_color"],
        },
        ThemePartUsage {
            part: "label",
            token: "app.foreground",
            states: &["default", "checked"],
            appearance_fields: &["CheckboxAppearance.label_color"],
        },
        ThemePartUsage {
            part: "disabled fill",
            token: "state.disabled.background",
            states: &["disabled"],
            appearance_fields: &["CheckboxAppearance.indicator_background"],
        },
        ThemePartUsage {
            part: "disabled foreground",
            token: "state.disabled.foreground",
            states: &["disabled"],
            appearance_fields: &["CheckboxAppearance.label_color", "CheckboxAppearance.checkmark_color"],
        },
        ThemePartUsage {
            part: "focus ring",
            token: "focus.ring",
            states: &["focused"],
            appearance_fields: &["CheckboxAppearance.adorner"],
        },
    ],
};

impl DefaultCheckboxTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl CheckboxTheme for DefaultCheckboxTheme {
    fn resolve(&self, checked: bool, state: InteractionState) -> CheckboxAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let size = ControlSize::Md;
        let layer = state.layer();

        let indicator_background = match (checked, layer) {
            (_, InteractionLayer::Disabled) => palette.state.disabled.background,
            (true, InteractionLayer::Pressed) => palette.action.prominent.pressed_background,
            (true, InteractionLayer::Hovered) => palette.action.prominent.hover_background,
            (true, InteractionLayer::Default) => palette.action.prominent.background,
            (false, InteractionLayer::Pressed) => palette.state.pressed.background,
            (false, InteractionLayer::Hovered) => palette.state.hover.background,
            (false, InteractionLayer::Default) => palette.form.input.background,
        };

        let label_color = if state.disabled {
            palette.state.disabled.foreground
        } else {
            palette.app.foreground
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

        CheckboxAppearance {
            control_background: None,
            control_border: None,
            indicator_background,
            indicator_border: if checked && !state.disabled {
                indicator_background
            } else {
                palette.form.input.border
            },
            checkmark_color: if state.disabled {
                palette.state.disabled.foreground
            } else {
                palette.action.prominent.foreground
            },
            label_color,
            adorner,
            label_typography: typography.text.label,
            control_radius: metrics.radius(size),
            control_padding_x: 0.0,
            control_padding_y: 0.0,
            indicator_radius: metrics.radius.sm,
            indicator_size: metrics.control_height(size) * 0.5,
            checkmark_size: metrics.control_height(size) * 0.42,
            gap: metrics.gap(size),
            height: metrics.control_height(size),
        }
    }
}
