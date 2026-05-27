use std::sync::{Arc, OnceLock};

use gpui::{Hsla, SharedString};

use crate::controls::button_family::{ButtonKind, ButtonVariant, button_variant};
use crate::theme::adorner::{AdornerPlacement, AdornerSpec, FocusRingAdornerSpec};
use crate::theme::{
    ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemePartUsage, ThemeTokens, ThemeUsage,
};

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
    /// `kind` selects the selected-state accent (`Standard` = secondary, `Prominent` = primary accent).
    fn resolve(&self, kind: ButtonKind, checked: bool, state: InteractionState) -> RadioButtonAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultRadioButtonTheme {
    tokens: ThemeTokens,
}

pub fn default_radio_button_theme() -> Arc<dyn RadioButtonTheme> {
    if let Some(radix) = crate::theme::radix::active_radix_theme() {
        return radix.radio_button_theme(crate::theme::RadixButtonStyle::Primary);
    }

    if let Some(live) = crate::theme::pack::active_live_theme() {
        return live;
    }
    static THEME: OnceLock<Arc<dyn RadioButtonTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultRadioButtonTheme::default())).clone()
}

pub const RADIO_BUTTON_THEME_USAGE: ThemeUsage = ThemeUsage {
    label: "Radio Button",
    parts: &[
        ThemePartUsage {
            part: "unchecked indicator background",
            token: "form.input.background",
            states: &["unchecked"],
            appearance_fields: &["RadioButtonAppearance.indicator_background"],
        },
        ThemePartUsage {
            part: "unchecked indicator hover background",
            token: "state.hover.background",
            states: &["unchecked hovered"],
            appearance_fields: &["RadioButtonAppearance.indicator_background"],
        },
        ThemePartUsage {
            part: "unchecked indicator pressed background",
            token: "state.pressed.background",
            states: &["unchecked pressed"],
            appearance_fields: &["RadioButtonAppearance.indicator_background"],
        },
        ThemePartUsage {
            part: "unchecked indicator border",
            token: "form.input.border",
            states: &["unchecked", "unchecked hovered", "unchecked pressed"],
            appearance_fields: &["RadioButtonAppearance.indicator_border"],
        },
        ThemePartUsage {
            part: "checked standard indicator border",
            token: "action.standard.background",
            states: &["checked standard"],
            appearance_fields: &["RadioButtonAppearance.indicator_border", "RadioButtonAppearance.dot_color"],
        },
        ThemePartUsage {
            part: "checked standard indicator hover border",
            token: "action.standard.hover_background",
            states: &["checked standard hovered"],
            appearance_fields: &["RadioButtonAppearance.indicator_border", "RadioButtonAppearance.dot_color"],
        },
        ThemePartUsage {
            part: "checked standard indicator pressed border",
            token: "action.standard.pressed_background",
            states: &["checked standard pressed"],
            appearance_fields: &["RadioButtonAppearance.indicator_border", "RadioButtonAppearance.dot_color"],
        },
        ThemePartUsage {
            part: "checked standard dot",
            token: "action.standard.foreground",
            states: &["checked standard"],
            appearance_fields: &["RadioButtonAppearance.dot_color"],
        },
        ThemePartUsage {
            part: "checked prominent indicator border",
            token: "action.prominent.background",
            states: &["checked prominent"],
            appearance_fields: &["RadioButtonAppearance.indicator_border", "RadioButtonAppearance.dot_color"],
        },
        ThemePartUsage {
            part: "checked prominent indicator hover border",
            token: "action.prominent.hover_background",
            states: &["checked prominent hovered"],
            appearance_fields: &["RadioButtonAppearance.indicator_border", "RadioButtonAppearance.dot_color"],
        },
        ThemePartUsage {
            part: "checked prominent indicator pressed border",
            token: "action.prominent.pressed_background",
            states: &["checked prominent pressed"],
            appearance_fields: &["RadioButtonAppearance.indicator_border", "RadioButtonAppearance.dot_color"],
        },
        ThemePartUsage {
            part: "checked prominent dot",
            token: "action.prominent.foreground",
            states: &["checked prominent"],
            appearance_fields: &["RadioButtonAppearance.dot_color"],
        },
        ThemePartUsage {
            part: "label",
            token: "app.foreground",
            states: &["default", "checked"],
            appearance_fields: &["RadioButtonAppearance.label_color"],
        },
        ThemePartUsage {
            part: "disabled fill",
            token: "state.disabled.background",
            states: &["disabled"],
            appearance_fields: &["RadioButtonAppearance.indicator_background"],
        },
        ThemePartUsage {
            part: "disabled foreground",
            token: "state.disabled.foreground",
            states: &["disabled"],
            appearance_fields: &[
                "RadioButtonAppearance.indicator_border",
                "RadioButtonAppearance.dot_color",
                "RadioButtonAppearance.label_color",
            ],
        },
        ThemePartUsage {
            part: "focus ring",
            token: "focus.ring",
            states: &["focused"],
            appearance_fields: &["RadioButtonAppearance.adorner"],
        },
    ],
};

impl DefaultRadioButtonTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl RadioButtonTheme for DefaultRadioButtonTheme {
    fn resolve(&self, kind: ButtonKind, checked: bool, state: InteractionState) -> RadioButtonAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let size = ControlSize::Md;
        let layer = state.layer();
        let checked_action = match button_variant(kind) {
            ButtonVariant::Standard => &palette.action.standard,
            _ => &palette.action.prominent,
        };

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
