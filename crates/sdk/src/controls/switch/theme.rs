use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla, SharedString};

use crate::controls::button_family::{ButtonKind, ButtonVariant, button_variant};
use crate::theme::adorner::{AdornerPlacement, AdornerSpec, FocusRingAdornerSpec};
use crate::theme::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemeTokens};

#[derive(Clone, Debug)]
pub struct SwitchAppearance {
    pub track_background: Hsla,
    pub track_border: Hsla,
    pub thumb_background: Hsla,
    pub thumb_border: Hsla,
    pub thumb_shadow: Vec<BoxShadow>,
    pub label_color: Hsla,
    pub adorner: Option<AdornerSpec>,
    pub label_typography: LumaTextStyle,
    pub label_font_family: SharedString,
    pub width: f32,
    pub height: f32,
    pub thumb_size: f32,
    pub padding: f32,
    pub gap: f32,
    pub radius: f32,
}

pub trait SwitchTheme: Send + Sync {
    /// `kind` selects the on-state accent (`Standard` = filled secondary, `Prominent` = primary accent).
    fn resolve(&self, kind: ButtonKind, on: bool, state: InteractionState) -> SwitchAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultSwitchTheme {
    tokens: ThemeTokens,
}

pub fn default_switch_theme() -> Arc<dyn SwitchTheme> {
    if let Some(radix) = crate::theme::radix::active_radix_theme() {
        return radix.switch_theme(crate::theme::RadixButtonStyle::Primary);
    }
    static THEME: OnceLock<Arc<dyn SwitchTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultSwitchTheme::default())).clone()
}

impl DefaultSwitchTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl SwitchTheme for DefaultSwitchTheme {
    fn resolve(&self, kind: ButtonKind, on: bool, state: InteractionState) -> SwitchAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let elevation = &self.tokens.elevation;
        let size = ControlSize::Md;
        let layer = state.layer();
        let on_action = match button_variant(kind) {
            ButtonVariant::Standard => &palette.action.standard,
            _ => &palette.action.prominent,
        };

        let track_background = match (on, layer) {
            (_, InteractionLayer::Disabled) => palette.state.disabled.background,
            (true, InteractionLayer::Pressed) => on_action.pressed_background,
            (true, InteractionLayer::Hovered) => on_action.hover_background,
            (true, InteractionLayer::Default) => on_action.background,
            (false, InteractionLayer::Pressed) => palette.state.pressed.background,
            (false, InteractionLayer::Hovered) => palette.state.hover.background,
            (false, InteractionLayer::Default) => palette.form.input.background,
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

        SwitchAppearance {
            track_background,
            track_border: if on && !state.disabled {
                track_background
            } else {
                palette.form.input.border
            },
            thumb_background: if state.disabled {
                palette.state.disabled.foreground
            } else if on {
                on_action.foreground
            } else {
                palette.surface.panel.background
            },
            thumb_border: if state.disabled {
                palette.state.disabled.background
            } else if on {
                on_action.foreground
            } else {
                palette.border.default
            },
            thumb_shadow: elevation.thumb.to_box_shadows(),
            label_color: if state.disabled {
                palette.state.disabled.foreground
            } else {
                palette.app.foreground
            },
            adorner,
            label_typography: typography.text.label,
            label_font_family: typography.font.sans.family.clone().into(),
            width: 42.0,
            height: 22.0,
            thumb_size: metrics.control_height(size) * 0.5,
            padding: 2.0,
            gap: metrics.gap(size),
            radius: metrics.radius.pill,
        }
    }
}
