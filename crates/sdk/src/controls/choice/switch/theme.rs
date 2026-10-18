use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla, SharedString};

use crate::theme::scales::{label_baseline_shift, snap_to_pixel};
use crate::theme::{ControlSize, InteractionState, LumaTextStyle, MetricTokens, ThemeTokens};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwitchScale {
    /// Whole-control opacity when disabled.
    pub disabled_opacity: f32,
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
        Self::compute_with_stylesheet(
            size,
            metrics,
            scale_factor,
            crate::theme::stylesheet::CommonStylesheet::sdk_default(),
        )
    }

    /// Resolve a snapshot's shared dimensions without changing control behavior.
    pub fn compute_with_stylesheet(
        size: ControlSize,
        metrics: &MetricTokens,
        scale_factor: f32,
        stylesheet: &crate::theme::stylesheet::CommonStylesheet,
    ) -> Self {
        let key = size.stylesheet_key();
        let baseline = crate::theme::stylesheet::CommonStylesheet::sdk_default().switch.authored_geometry(key);
        let recipe = stylesheet.switch.authored_geometry(key);
        // Authored scales are dimension / control height, preserving proportional
        // sizing when metric tokens change. Explicit dimensions take precedence below.
        let height = metrics.control_height(size);
        let fallback = Self {
            disabled_opacity: recipe.disabled_opacity.or(baseline.disabled_opacity).unwrap_or_default(),
            track_width: snap_to_pixel(
                height * recipe.track_width_scale.or(baseline.track_width_scale).unwrap_or_default(),
                scale_factor,
            ),
            track_height: snap_to_pixel(
                height * recipe.track_height_scale.or(baseline.track_height_scale).unwrap_or_default(),
                scale_factor,
            ),
            track_padding: snap_to_pixel(baseline.track_padding.unwrap_or_default(), scale_factor),
            thumb_size: snap_to_pixel(
                height * recipe.thumb_scale.or(baseline.thumb_scale).unwrap_or_default(),
                scale_factor,
            ),
            track_radius: metrics.radius.pill,
            gap: snap_to_pixel(metrics.gap(size), scale_factor),
            label_baseline_shift: label_baseline_shift(size, stylesheet),
        };
        let geometry = stylesheet.switch.resolve_geometry(key, fallback, scale_factor);
        Self {
            track_width: geometry.track_width.value_px,
            track_height: geometry.track_height.value_px,
            track_padding: geometry.track_padding.value_px,
            thumb_size: geometry.thumb_size.value_px,
            gap: geometry.gap.value_px,
            ..fallback
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
        let palette = self.tokens.stylesheet.recipes.palette("switch", &self.tokens.palette);
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

        let track_border = if state.focused && !state.disabled {
            palette.focus.ring
        } else if on && !state.disabled {
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

    fn scale(&self, size: ControlSize, scale_factor: f32) -> SwitchScale {
        SwitchScale::compute_with_stylesheet(size, &self.tokens.metrics, scale_factor, &self.tokens.stylesheet)
    }
}
