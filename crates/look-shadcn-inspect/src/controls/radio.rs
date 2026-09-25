//! Inspect metadata for `radio`.

use luma::theme::{InteractionState, ThemeMode};
use luma_look_shadcn::{ResolvedColor, ResolvedMetric, ShadcnButtonStyle, ShadcnModeTokens};

pub struct RadioButtonInspectPalette {
    pub indicator_background: ResolvedColor,
    pub indicator_border: ResolvedColor,
    pub dot_color: ResolvedColor,
    pub label_color: ResolvedColor,
}

pub fn inspect_radio_button_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    selected: bool,
    state: InteractionState,
) -> RadioButtonInspectPalette {
    let colors = luma_look_shadcn::tables::resolve_radio_palette(mode, theme_mode, style, selected, state);
    RadioButtonInspectPalette {
        indicator_background: colors.indicator_background,
        indicator_border: colors.indicator_border,
        dot_color: colors.dot_color,
        label_color: colors.label_color,
    }
}

#[derive(Clone, Debug)]
pub struct RadioButtonInspectMetrics {
    pub height: ResolvedMetric,
    pub gap: ResolvedMetric,
    pub indicator_size: ResolvedMetric,
    pub dot_size: ResolvedMetric,
    pub control_radius: ResolvedMetric,
    pub border_width: ResolvedMetric,
    pub focus_ring_width: ResolvedMetric,
    pub focus_ring_offset: ResolvedMetric,
}

pub fn inspect_radio_button_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: luma::theme::ControlSize,
) -> RadioButtonInspectMetrics {
    let table = luma_look_shadcn::tables::metrics::resolve_radio_button_metrics(mode, theme_mode, size);
    table.into()
}

pub fn inspect_radio_button_elevation(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    selected: bool,
    state: InteractionState,
) -> crate::controls::button::ButtonInspectElevation {
    use luma::theme::ControlSize;
    use luma_look_shadcn::stylesheet::{embedded_stylesheet, resolve_stylesheet_shadow_token};
    use crate::controls::button::{button_style_key, inspect_layered_elevation};

    let look = luma_look_shadcn::paint::radio_button_look(mode, style, selected, state, ControlSize::Md);
    let layer = state.layer();
    let rule = embedded_stylesheet().radio.elevation_rule_for_layer(layer);
    let rule_shadow = rule.map(|rule| rule.shadow.clone()).unwrap_or_else(|| "none".to_string());
    let token = rule.and_then(|rule| resolve_stylesheet_shadow_token(&rule.shadow));
    inspect_layered_elevation(
        mode,
        theme_mode,
        state,
        rule_shadow,
        token,
        look.indicator_shadow.as_ref(),
        button_style_key(style),
    )
}

impl From<luma_look_shadcn::tables::metrics::RadioButtonMetricTable> for RadioButtonInspectMetrics {
    fn from(table: luma_look_shadcn::tables::metrics::RadioButtonMetricTable) -> Self {
        Self {
            height: table.height,
            gap: table.gap,
            indicator_size: table.indicator_size,
            dot_size: table.dot_size,
            control_radius: table.control_radius,
            border_width: table.border_width,
            focus_ring_width: table.focus_ring_width,
            focus_ring_offset: table.focus_ring_offset,
        }
    }
}
