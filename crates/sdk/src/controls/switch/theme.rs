use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla, SharedString};

use crate::theme::layout::{label_baseline_shift, snap_to_pixel};
use crate::theme::{ControlSize, InteractionState, LumaTextStyle, MetricTokens, ThemeTokens};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwitchScale {
    pub track_width: f32,
    pub track_height: f32,
    pub track_padding: f32,
    pub thumb_size: f32,
    pub track_radius: f32,
    pub gap: f32,
    pub label_baseline_shift: f32,
}

impl SwitchScale {
    pub fn compute(size: ControlSize, metrics: &MetricTokens, scale_factor: f32) -> Self {
        let control_height = metrics.control_height(size);

        Self {
            track_width: snap_to_pixel(control_height * (42.0 / 36.0), scale_factor),
            track_height: snap_to_pixel(control_height * (22.0 / 36.0), scale_factor),
            track_padding: snap_to_pixel((control_height * (2.0 / 36.0)).max(1.0), scale_factor),
            thumb_size: snap_to_pixel(control_height * 0.5, scale_factor),
            track_radius: metrics.radius.pill,
            gap: snap_to_pixel(metrics.gap(size), scale_factor),
            label_baseline_shift: label_baseline_shift(size),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SwitchPalette {
    pub track_background: Hsla,
    pub track_border: Hsla,
    pub thumb_background: Hsla,
    pub thumb_border: Hsla,
    pub thumb_shadow: Vec<BoxShadow>,
    pub label_color: Hsla,
    pub label_typography: LumaTextStyle,
    pub label_font_family: SharedString,
}

pub type SwitchLook = SwitchPalette;

pub trait SwitchTheme: Send + Sync {
    fn resolve(&self, on: bool, state: InteractionState, size: ControlSize) -> SwitchPalette;
    fn metrics(&self) -> MetricTokens;
    /// Look-owned geometry for `size`. Defaults to [`SwitchScale::compute`].
    fn scale(&self, size: ControlSize, scale_factor: f32) -> SwitchScale {
        SwitchScale::compute(size, &self.metrics(), scale_factor)
    }
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
    fn resolve(&self, on: bool, state: InteractionState, size: ControlSize) -> SwitchPalette {
        let palette = &self.tokens.palette;
        let typography = &self.tokens.typography;
        let elevation = &self.tokens.elevation;
        let selected = palette.state.selected;

        let track_background = if state.disabled {
            palette.state.disabled.background
        } else if on {
            selected.background
        } else {
            palette.form.input.background
        };

        let track_border = if on && !state.disabled {
            track_background
        } else {
            palette.border.default
        };

        let (thumb_background, thumb_border) = if state.disabled {
            (palette.state.disabled.foreground, palette.state.disabled.background)
        } else if on {
            let thumb = selected.foreground;
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
            label_typography: {
                let mut label_typography = typography.text.label;
                crate::controls::textfield::apply_control_size_typography(&mut label_typography, typography, size);
                label_typography
            },
            label_font_family: typography.font.sans.family.clone().into(),
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }
}
