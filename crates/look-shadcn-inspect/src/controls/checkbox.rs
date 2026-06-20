//! Inspect metadata for `checkbox`.

use gpui_luma::theme::{InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{
    LookContext, ColorSource, LookResolver, ResolvedColor, ResolvedMetric, ShadcnButtonStyle, ShadcnModeTokens,
    format_inspect_css_key,
};

pub struct CheckboxInspectPalette {
    pub indicator_background: ResolvedColor,
    pub indicator_border: ResolvedColor,
    pub checkmark_color: ResolvedColor,
    pub label_color: ResolvedColor,
    pub focus_ring: Option<ResolvedColor>,
}

pub fn inspect_checkbox_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    checked: bool,
    state: InteractionState,
) -> CheckboxInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, state);
    let catalog = ctx.catalog();
    let layer = state.layer();
    let resolver = LookResolver::new(catalog, theme_mode, "checkbox_inspect");
    let colors = gpui_luma_look_shadcn::tables::resolve_checkbox_colors(&resolver, style, checked, layer)
        .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::CheckboxColorTable::fallback());
    let indicator_border = effective_checkbox_indicator_border(checked, state.disabled, &colors, &resolver);
    let focus_ring = state.focused.then(|| resolver.resolve_decl("ring")).transpose().ok().flatten();

    CheckboxInspectPalette {
        indicator_background: colors.indicator_background,
        indicator_border,
        checkmark_color: colors.checkmark_color,
        label_color: colors.label_color,
        focus_ring,
    }
}

fn effective_checkbox_indicator_border(
    checked: bool,
    disabled: bool,
    colors: &gpui_luma_look_shadcn::tables::CheckboxColorTable,
    resolver: &LookResolver,
) -> ResolvedColor {
    if checked && !disabled {
        let note = format!("= {}", format_inspect_css_key(&colors.indicator_background.source));
        ResolvedColor { value: colors.indicator_background.value, source: ColorSource::Derived { note } }
    } else {
        resolver.resolve_decl("border").unwrap_or_else(|_| ResolvedColor {
            value: colors.indicator_background.value,
            source: ColorSource::CssVar { token: "border".into() },
        })
    }
}

#[derive(Clone, Debug)]
pub struct CheckboxInspectMetrics {
    pub height: ResolvedMetric,
    pub gap: ResolvedMetric,
    pub indicator_size: ResolvedMetric,
    pub indicator_radius: ResolvedMetric,
    pub glyph_size: ResolvedMetric,
    pub control_radius: ResolvedMetric,
    pub border_width: ResolvedMetric,
    pub focus_ring_width: ResolvedMetric,
    pub focus_ring_offset: ResolvedMetric,
}

pub fn inspect_checkbox_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: gpui_luma::theme::ControlSize,
) -> CheckboxInspectMetrics {
    use gpui_luma::controls::checkbox::CheckboxScale;

    use gpui_luma_look_shadcn::catalog::SpacingField;
    use crate::metrics::{
        border_width_metric, control_size_key, derived_metric, focus_ring_offset_metric, focus_ring_width_metric,
        radius_metric, scaffold_control_metric, spacing_control_metric,
    };

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let scale = CheckboxScale::compute(size, metrics, 1.0);
    let size_key = control_size_key(size);
    let (ratio, radius_px, _icon_inset) = match size {
        gpui_luma::theme::ControlSize::Sm => (0.45, 2.0, 2.0),
        gpui_luma::theme::ControlSize::Md => (0.50, 4.0, 3.0),
        gpui_luma::theme::ControlSize::Lg => (0.55, 6.0, 4.0),
    };

    CheckboxInspectMetrics {
        height: scaffold_control_metric(size_key, "control_height", scale.height),
        gap: spacing_control_metric(catalog, size, SpacingField::Gap, scale.gap),
        indicator_size: derived_metric(
            format!("{size_key} indicator = control_height × {ratio}"),
            scale.indicator_size,
        ),
        indicator_radius: derived_metric(
            format!("{size_key} indicator_radius = {radius_px}px (scaffold)"),
            scale.indicator_radius,
        ),
        glyph_size: derived_metric(format!("{size_key} glyph = indicator − inset"), scale.glyph_size),
        control_radius: radius_metric(catalog, size, scale.control_radius),
        border_width: border_width_metric(metrics),
        focus_ring_width: focus_ring_width_metric(metrics),
        focus_ring_offset: focus_ring_offset_metric(metrics),
    }
}
