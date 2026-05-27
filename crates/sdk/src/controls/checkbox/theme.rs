use std::sync::{Arc, OnceLock};

use gpui::{Hsla, SharedString};

use crate::controls::button_family::{ButtonKind, ButtonVariant, button_variant};
use crate::theme::adorner::{AdornerPlacement, AdornerSpec, FocusRingAdornerSpec};
use crate::theme::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemeTokens};

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
    pub label_font_family: SharedString,
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
    /// `kind` selects the checked-state accent (`Standard` = filled secondary, `Prominent` = primary accent).
    fn resolve(&self, kind: ButtonKind, checked: bool, state: InteractionState) -> CheckboxAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultCheckboxTheme {
    tokens: ThemeTokens,
}

pub fn default_checkbox_theme() -> Arc<dyn CheckboxTheme> {
    if let Some(radix) = crate::theme::radix::active_radix_theme() {
        return radix.checkbox_theme(crate::theme::RadixButtonStyle::Primary);
    }
    static THEME: OnceLock<Arc<dyn CheckboxTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultCheckboxTheme::default())).clone()
}

impl DefaultCheckboxTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl CheckboxTheme for DefaultCheckboxTheme {
    fn resolve(&self, kind: ButtonKind, checked: bool, state: InteractionState) -> CheckboxAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let size = ControlSize::Md;
        let layer = state.layer();
        let checked_action = match button_variant(kind) {
            ButtonVariant::Standard => &palette.action.standard,
            _ => &palette.action.prominent,
        };

        let indicator_background = match (checked, layer) {
            (_, InteractionLayer::Disabled) => palette.state.disabled.background,
            (true, InteractionLayer::Pressed) => checked_action.pressed_background,
            (true, InteractionLayer::Hovered) => checked_action.hover_background,
            (true, InteractionLayer::Default) => checked_action.background,
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
            } else if checked {
                checked_action.foreground
            } else {
                palette.app.foreground
            },
            label_color,
            adorner,
            label_typography: typography.text.label,
            label_font_family: typography.font.sans.family.clone().into(),
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
