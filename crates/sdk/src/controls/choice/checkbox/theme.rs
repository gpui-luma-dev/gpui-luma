use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla, SharedString};

use crate::theme::scales::{label_baseline_shift, snap_to_pixel};
use crate::theme::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, MetricTokens, ThemeTokens};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CheckboxScale {
    pub control_radius: f32,
    pub control_padding_x: f32,
    pub control_padding_y: f32,
    pub indicator_size: f32,
    pub indicator_radius: f32,
    pub height: f32,
    pub gap: f32,
    pub label_baseline_shift: f32,
    pub glyph_size: f32,
}

impl CheckboxScale {
    pub fn compute(size: ControlSize, metrics: &MetricTokens, scale_factor: f32) -> Self {
        Self::compute_with_stylesheet(
            size,
            metrics,
            scale_factor,
            crate::theme::stylesheet::CommonStylesheet::sdk_default(),
        )
    }

    /// Resolve shared appearance over the vanilla SDK baseline, then snap once.
    pub fn compute_with_stylesheet(
        size: ControlSize,
        metrics: &MetricTokens,
        scale_factor: f32,
        stylesheet: &crate::theme::stylesheet::CommonStylesheet,
    ) -> Self {
        let key = size.stylesheet_key();
        let baseline = crate::theme::stylesheet::CommonStylesheet::sdk_default().checkbox.authored_geometry(key);
        let geometry = stylesheet.checkbox.authored_geometry(key).with_fallback(&baseline);
        let indicator_size = snap_to_pixel(
            geometry
                .indicator_size
                .unwrap_or_else(|| metrics.control_height(size) * geometry.indicator_scale.unwrap_or_default()),
            scale_factor,
        );
        let glyph_inset = snap_to_pixel(geometry.glyph_inset.unwrap_or_default(), scale_factor);

        Self {
            control_radius: metrics.radius(size),
            control_padding_x: snap_to_pixel(geometry.control_padding_x.unwrap_or_default(), scale_factor),
            control_padding_y: snap_to_pixel(geometry.control_padding_y.unwrap_or_default(), scale_factor),
            indicator_size,
            height: snap_to_pixel(metrics.control_height(size), scale_factor),
            gap: snap_to_pixel(geometry.gap.unwrap_or_else(|| metrics.gap(size)), scale_factor),
            label_baseline_shift: label_baseline_shift(size),
            indicator_radius: snap_to_pixel(geometry.indicator_radius.unwrap_or_default(), scale_factor),
            glyph_size: snap_to_pixel(
                geometry.glyph_size.unwrap_or_else(|| (indicator_size - glyph_inset * 2.0).max(0.0)),
                scale_factor,
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CheckboxPalette {
    pub control_background: Option<Hsla>,
    pub control_border: Option<Hsla>,
    pub indicator_background: Hsla,
    pub indicator_border: Hsla,
    pub checkmark_color: Hsla,
    pub label_color: Hsla,
    pub label_typography: LumaTextStyle,
    pub label_font_family: SharedString,
    pub indicator_shadow: Option<Vec<BoxShadow>>,
}

pub type CheckboxLook = CheckboxPalette;

pub trait CheckboxTheme: Send + Sync {
    fn resolve(&self, checked: bool, state: InteractionState, size: ControlSize) -> CheckboxPalette;
    fn metrics(&self) -> MetricTokens;

    /// Geometry for the indicator / label row. Defaults to [`CheckboxScale::compute`].
    fn scale(&self, size: ControlSize, scale_factor: f32) -> CheckboxScale {
        CheckboxScale::compute(size, &self.metrics(), scale_factor)
    }
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
    fn resolve(&self, checked: bool, state: InteractionState, size: ControlSize) -> CheckboxPalette {
        let palette = &self.tokens.palette;
        let typography = &self.tokens.typography;
        let layer = state.layer();
        let selected = palette.state.selected;

        let indicator_background = match (checked, layer) {
            (_, InteractionLayer::Disabled) => palette.state.disabled.background,
            (true, InteractionLayer::Pressed) => palette.state.pressed.background,
            (true, InteractionLayer::Hovered) => selected.background,
            (true, InteractionLayer::Default) => selected.background,
            (false, InteractionLayer::Pressed) => palette.state.pressed.background,
            (false, InteractionLayer::Hovered) => palette.state.hover.background,
            (false, InteractionLayer::Default) => palette.form.input.background,
        };

        let label_color = if state.disabled {
            palette.state.disabled.foreground
        } else {
            palette.app.foreground
        };

        CheckboxPalette {
            control_background: None,
            control_border: None,
            indicator_background,
            indicator_border: if state.focused && !state.disabled {
                palette.focus.ring
            } else if checked && !state.disabled {
                indicator_background
            } else {
                palette.form.input.border
            },
            checkmark_color: if state.disabled {
                palette.state.disabled.foreground
            } else if checked {
                selected.foreground
            } else {
                palette.app.foreground
            },
            label_color,
            label_typography: {
                let mut label_typography = typography.text.label;
                crate::controls::textfield::apply_control_size_typography(&mut label_typography, typography, size);
                label_typography
            },
            label_font_family: typography.font.sans.family.clone().into(),
            indicator_shadow: None,
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }

    fn scale(&self, size: ControlSize, scale_factor: f32) -> CheckboxScale {
        CheckboxScale::compute_with_stylesheet(size, &self.tokens.metrics, scale_factor, &self.tokens.stylesheet)
    }
}
