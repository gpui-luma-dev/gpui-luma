use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla, SharedString};

use crate::theme::adorner::{AdornerPlacement, AdornerSpec, FocusRingAdornerSpec};
use crate::theme::{InteractionState, LumaTextStyle, MetricTokens, ThemeTokens};

#[derive(Clone, Debug)]
pub struct SwitchPalette {
    pub track_background: Hsla,
    pub track_border: Hsla,
    pub thumb_background: Hsla,
    pub thumb_border: Hsla,
    pub thumb_shadow: Vec<BoxShadow>,
    pub label_color: Hsla,
    pub adorner: Option<AdornerSpec>,
    pub label_typography: LumaTextStyle,
    pub label_font_family: SharedString,
}

pub type SwitchAppearance = SwitchPalette;

pub trait SwitchTheme: Send + Sync {
    fn resolve(&self, on: bool, state: InteractionState) -> SwitchPalette;
    fn metrics(&self) -> &MetricTokens;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultSwitchTheme {
    tokens: ThemeTokens,
}

pub fn default_switch_theme() -> Arc<dyn SwitchTheme> {
    static THEME: OnceLock<Arc<dyn SwitchTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultSwitchTheme::default())).clone()
}

impl DefaultSwitchTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl SwitchTheme for DefaultSwitchTheme {
    fn resolve(&self, on: bool, state: InteractionState) -> SwitchPalette {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let elevation = &self.tokens.elevation;
        let on_action = palette.action.prominent;

        let track_background = if state.disabled {
            palette.state.disabled.background
        } else if on {
            on_action.background
        } else {
            palette.form.input.background
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

        let track_border = if on && !state.disabled {
            track_background
        } else {
            palette.border.default
        };

        let (thumb_background, thumb_border) = if state.disabled {
            (palette.state.disabled.foreground, palette.state.disabled.background)
        } else if on {
            let thumb = on_action.foreground;
            (thumb, thumb)
        } else {
            (palette.app.background, palette.border.default)
        };

        SwitchPalette {
            track_background,
            track_border,
            thumb_background,
            thumb_border,
            thumb_shadow: elevation.thumb.to_box_shadows(),
            label_color: if state.disabled {
                palette.state.disabled.foreground
            } else {
                palette.app.foreground
            },
            adorner,
            label_typography: typography.text.label,
            label_font_family: typography.font.sans.family.clone().into(),
        }
    }

    fn metrics(&self) -> &MetricTokens {
        &self.tokens.metrics
    }
}
