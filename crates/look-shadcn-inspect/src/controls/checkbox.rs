//! Inspect metadata for `checkbox`.

use luma::theme::{InteractionState, ThemeMode};
use luma_look_shadcn::{
    LookContext, ColorSource, LookResolver, ResolvedColor, ResolvedMetric, ShadcnButtonStyle, ShadcnModeTokens,
    format_inspect_css_key,
};

pub struct CheckboxInspectPalette {
    pub indicator_background: ResolvedColor,
    pub indicator_border: ResolvedColor,
    pub checkmark_color: ResolvedColor,
    pub label_color: ResolvedColor,
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
    let colors = luma_look_shadcn::tables::resolve_checkbox_colors(&resolver, style, checked, layer)
        .unwrap_or_else(|_| luma_look_shadcn::tables::CheckboxColorTable::fallback());
    let indicator_border = effective_checkbox_indicator_border(checked, state.disabled, &colors, &resolver);

    CheckboxInspectPalette {
        indicator_background: colors.indicator_background,
        indicator_border,
        checkmark_color: colors.checkmark_color,
        label_color: colors.label_color,
    }
}

fn effective_checkbox_indicator_border(
    checked: bool,
    disabled: bool,
    colors: &luma_look_shadcn::tables::CheckboxColorTable,
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
    size: luma::theme::ControlSize,
) -> CheckboxInspectMetrics {
    use luma::controls::checkbox::CheckboxScale;

    use luma_look_shadcn::catalog::SpacingField;
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
        luma::theme::ControlSize::Sm => (0.45, 2.0, 2.0),
        luma::theme::ControlSize::Md => (0.50, 4.0, 3.0),
        luma::theme::ControlSize::Lg => (0.55, 6.0, 4.0),
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

pub fn inspect_checkbox_elevation(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    checked: bool,
    state: InteractionState,
) -> crate::controls::button::ButtonInspectElevation {
    use luma::theme::ControlSize;
    use luma_look_shadcn::stylesheet::{embedded_stylesheet, resolve_stylesheet_shadow_token};
    use crate::controls::button::{button_style_key, inspect_layered_elevation};

    let look = luma_look_shadcn::paint::checkbox_look(mode, style, checked, state, ControlSize::Md);
    let layer = state.layer();
    let rule = embedded_stylesheet().checkbox.elevation_rule_for_layer(layer);
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
