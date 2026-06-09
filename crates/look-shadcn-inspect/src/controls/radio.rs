//! Inspect metadata for `radio`.

use gpui_luma::theme::{InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{
    AppearanceContext, LookResolver, ResolvedColor, ResolvedMetric, ShadcnButtonStyle, ShadcnModeTokens,
};

pub struct RadioButtonInspectPalette {
    pub indicator_background: ResolvedColor,
    pub indicator_border: ResolvedColor,
    pub dot_color: ResolvedColor,
    pub label_color: ResolvedColor,
    pub focus_ring: Option<ResolvedColor>,
}

pub fn inspect_radio_button_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    selected: bool,
    state: InteractionState,
) -> RadioButtonInspectPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, state);
    let catalog = ctx.catalog();
    let layer = state.layer();
    let resolver = LookResolver::new(catalog, theme_mode, "radio_inspect");
    let colors = gpui_luma_look_shadcn::tables::resolve_radio_colors(&resolver, style, selected, layer)
        .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::RadioColorTable::fallback());
    let indicator_border = if selected && !state.disabled {
        colors.selection_ring
    } else {
        resolver.resolve_decl("border").unwrap_or(colors.selection_ring)
    };
    let focus_ring = state.focused.then(|| resolver.resolve_decl("ring")).transpose().ok().flatten();

    RadioButtonInspectPalette {
        indicator_background: colors.indicator_background,
        indicator_border,
        dot_color: colors.dot_color,
        label_color: colors.label_color,
        focus_ring,
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
    size: gpui_luma::theme::ControlSize,
) -> RadioButtonInspectMetrics {
    use gpui_luma::controls::radio_button::RadioScale;

    use gpui_luma_look_shadcn::catalog::SpacingField;
    use crate::metrics::{
        border_width_metric, control_size_key, derived_metric, focus_ring_offset_metric, focus_ring_width_metric,
        radius_metric, scaffold_control_metric, spacing_control_metric,
    };

    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let scale = RadioScale::compute(size, metrics, 1.0);
    let size_key = control_size_key(size);
    let ratio = match size {
        gpui_luma::theme::ControlSize::Sm => 0.45,
        gpui_luma::theme::ControlSize::Md => 0.50,
        gpui_luma::theme::ControlSize::Lg => 0.55,
    };

    RadioButtonInspectMetrics {
        height: scaffold_control_metric(size_key, "control_height", scale.height),
        gap: spacing_control_metric(catalog, size, SpacingField::Gap, scale.gap),
        indicator_size: derived_metric(
            format!("{size_key} indicator = control_height × {ratio}"),
            scale.indicator_size,
        ),
        dot_size: derived_metric(format!("{size_key} dot = control_height × 0.24"), scale.dot_size),
        control_radius: radius_metric(catalog, size, scale.control_radius),
        border_width: border_width_metric(metrics),
        focus_ring_width: focus_ring_width_metric(metrics),
        focus_ring_offset: focus_ring_offset_metric(metrics),
    }
}
