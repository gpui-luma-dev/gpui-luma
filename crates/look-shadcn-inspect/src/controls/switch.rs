//! Inspect metadata for `switch`.

use gpui_luma::theme::{InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{
    LookContext, ColorSource, LookResolver, ResolvedColor, ResolvedMetric, ShadcnButtonStyle, ShadcnModeTokens,
    format_inspect_css_key,
};

pub struct SwitchInspectPalette {
    pub track_background: ResolvedColor,
    pub track_border: ResolvedColor,
    pub thumb_background: ResolvedColor,
    pub thumb_border: ResolvedColor,
    pub label_color: ResolvedColor,
}

pub fn inspect_switch_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    on: bool,
    state: InteractionState,
) -> SwitchInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, state);
    let catalog = ctx.catalog();
    let resolver = LookResolver::new(catalog, theme_mode, "switch_inspect");
    let colors = gpui_luma_look_shadcn::tables::resolve_switch_colors(&resolver, style, on, state.disabled)
        .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::SwitchColorTable::fallback());
    let thumb_border_color = colors.thumb_border;
    let track_border = if on && !state.disabled {
        let note = format!("= {}", format_inspect_css_key(&colors.track_background.source));
        ResolvedColor { value: colors.track_background.value, source: ColorSource::Derived { note } }
    } else {
        resolver.resolve_decl("border").unwrap_or(thumb_border_color.clone())
    };
    let thumb_border = if on && !state.disabled {
        let note = format!("= {}", format_inspect_css_key(&colors.thumb_background.source));
        ResolvedColor { value: colors.thumb_background.value, source: ColorSource::Derived { note } }
    } else {
        thumb_border_color
    };

    SwitchInspectPalette {
        track_background: colors.track_background,
        track_border,
        thumb_background: colors.thumb_background,
        thumb_border,
        label_color: colors.label_color,
    }
}

#[derive(Clone, Debug)]
pub struct SwitchInspectMetrics {
    pub track_width: ResolvedMetric,
    pub track_height: ResolvedMetric,
    pub track_padding: ResolvedMetric,
    pub thumb_size: ResolvedMetric,
    pub gap: ResolvedMetric,
    pub track_radius: ResolvedMetric,
    pub border_width: ResolvedMetric,
    pub focus_ring_width: ResolvedMetric,
    pub focus_ring_offset: ResolvedMetric,
}

pub fn inspect_switch_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    size: gpui_luma::theme::ControlSize,
) -> SwitchInspectMetrics {
    use gpui_luma_look_shadcn::catalog::SpacingField;
    use gpui_luma_look_shadcn::paint::switch_scale;
    use crate::metrics::{
        border_width_metric, control_size_key, derived_metric, focus_ring_offset_metric, focus_ring_width_metric,
        pill_radius_metric, spacing_control_metric,
    };

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let scale = switch_scale(mode, theme_mode, style, size, 1.0);
    let size_key = control_size_key(size);
    let style_key = match style {
        ShadcnButtonStyle::Primary => "primary",
        ShadcnButtonStyle::Secondary => "secondary",
        ShadcnButtonStyle::Outline => "outline",
        ShadcnButtonStyle::Ghost => "ghost",
        ShadcnButtonStyle::ContentOnly => "primary",
    };

    SwitchInspectMetrics {
        track_width: derived_metric(
            format!("{size_key} {style_key} track width (style.toml switch metrics)"),
            scale.track_width,
        ),
        track_height: derived_metric(
            format!("{size_key} {style_key} track height (style.toml switch metrics)"),
            scale.track_height,
        ),
        track_padding: derived_metric(
            format!("{size_key} {style_key} track padding = height × 2/22"),
            scale.track_padding,
        ),
        thumb_size: derived_metric(
            format!("{size_key} {style_key} thumb (style.toml switch metrics)"),
            scale.thumb_size,
        ),
        gap: spacing_control_metric(catalog, size, SpacingField::Gap, scale.gap),
        track_radius: pill_radius_metric(catalog, scale.track_radius),
        border_width: border_width_metric(metrics),
        focus_ring_width: focus_ring_width_metric(metrics),
        focus_ring_offset: focus_ring_offset_metric(metrics),
    }
}

pub fn inspect_switch_elevation(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    on: bool,
    state: InteractionState,
) -> crate::controls::button::ButtonInspectElevation {
    use gpui_luma::theme::ControlSize;
    use gpui_luma_look_shadcn::stylesheet::{embedded_stylesheet, resolve_stylesheet_shadow_token};
    use crate::controls::button::{button_style_key, inspect_layered_elevation};

    let look = gpui_luma_look_shadcn::paint::switch_look(mode, theme_mode, style, on, state, ControlSize::Md);
    let layer = state.layer();
    let rule = embedded_stylesheet().switch.elevation_rule_for_layer(layer);
    let rule_shadow = rule.map(|rule| rule.shadow.clone()).unwrap_or_else(|| "none".to_string());
    let token = rule.and_then(|rule| resolve_stylesheet_shadow_token(&rule.shadow));
    let shadows = if look.thumb_shadow.is_empty() {
        None
    } else {
        Some(&look.thumb_shadow)
    };
    inspect_layered_elevation(mode, theme_mode, state, rule_shadow, token, shadows, button_style_key(style))
}
