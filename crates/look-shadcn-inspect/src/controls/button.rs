//! Inspect metadata for `button`.

use gpui::{BoxShadow, Hsla};
use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{
    LookContext, LookResolver, MetricSource, ResolvedColor, ResolvedMetric, ResolvedTypography, ShadcnButtonStyle,
    ShadcnModeTokens, TypographySource,
};
use gpui_luma_look_shadcn::stylesheet::{embedded_stylesheet, find_button_elevation_rule, resolve_stylesheet_shadow_token};

use gpui_luma::controls::button_family::ButtonFamilyRole;
use gpui_luma_look_shadcn::catalog::SpacingField;

pub struct ButtonInspectPalette {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
    pub focus_ring: Option<ResolvedColor>,
}

pub fn inspect_button_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    state: InteractionState,
) -> ButtonInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, state);
    let style = if matches!(role, ButtonFamilyRole::Toggle { selected: false }) {
        ShadcnButtonStyle::Outline
    } else {
        style
    };
    let layer = state.layer();
    let selected = matches!(role, ButtonFamilyRole::Toggle { selected: true });
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "button_resolver");
    let colors = gpui_luma_look_shadcn::tables::resolve_button_colors(&resolver, style, layer, theme_mode, selected)
        .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::ButtonColorPalette::fallback());

    let border = effective_border_resolved(&colors);
    let focus_ring = state.focused.then(|| resolver.resolve_decl("ring")).transpose().ok().flatten();

    ButtonInspectPalette { background: colors.background, foreground: colors.foreground, border, focus_ring }
}

#[derive(Clone, Debug)]
pub struct ButtonInspectMetrics {
    pub height: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub gap: ResolvedMetric,
    pub radius: ResolvedMetric,
    pub border_width: ResolvedMetric,
    pub focus_ring_width: ResolvedMetric,
    pub focus_ring_offset: ResolvedMetric,
}

pub fn inspect_button_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    size: ControlSize,
    state: InteractionState,
) -> ButtonInspectMetrics {
    let look = gpui_luma_look_shadcn::paint::button_look(mode, theme_mode, style, role, size, state);
    let ctx = LookContext::new(mode, theme_mode, state);
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();

    let size_key = control_size_key(size);

    ButtonInspectMetrics {
        height: scaffold_control_metric(size_key, "control_height", look.height),
        padding_x: spacing_control_metric(catalog, size, SpacingField::PaddingX, look.padding_x),
        padding_y: spacing_control_metric(catalog, size, SpacingField::PaddingY, look.padding_y),
        gap: spacing_control_metric(catalog, size, SpacingField::Gap, look.gap),
        radius: radius_metric(catalog, size, look.radius),
        border_width: ResolvedMetric {
            value_px: metrics.border_width.default,
            source: MetricSource::Scaffold { path: "MetricTokens.border_width.default".into() },
        },
        focus_ring_width: ResolvedMetric {
            value_px: metrics.focus.width,
            source: MetricSource::Scaffold { path: "MetricTokens.focus.width".into() },
        },
        focus_ring_offset: focus_ring_offset_metric(
            gpui_luma::controls::button_family::button_family_effective_border(look.border),
            metrics,
        ),
    }
}

#[derive(Clone, Debug)]
pub struct ButtonInspectTypography {
    pub font_family: ResolvedTypography,
    pub font_size: ResolvedTypography,
    pub font_weight: ResolvedTypography,
    pub line_height: ResolvedTypography,
}

#[derive(Clone, Debug)]
pub struct ButtonInspectElevationLayer {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Hsla,
    pub css: String,
}

#[derive(Clone, Debug)]
pub struct ButtonInspectElevation {
    pub applied: bool,
    pub rule_shadow: String,
    pub style_key: String,
    pub token: Option<String>,
    pub catalog_value: Option<String>,
    pub layers: Vec<ButtonInspectElevationLayer>,
    pub shadows: Option<Vec<BoxShadow>>,
}

pub fn inspect_button_elevation(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    state: InteractionState,
) -> ButtonInspectElevation {
    let effective_style = effective_button_style(style, role);
    let look = gpui_luma_look_shadcn::paint::button_look(mode, theme_mode, style, role, ControlSize::Md, state);
    let ctx = LookContext::new(mode, theme_mode, state);
    let stylesheet = embedded_stylesheet();
    let style_key = button_style_key(effective_style);
    let rule = find_button_elevation_rule(stylesheet, effective_style);
    let rule_shadow = rule.map(|rule| rule.shadow.clone()).unwrap_or_else(|| "none".to_string());
    let token = rule.and_then(|rule| resolve_stylesheet_shadow_token(&rule.shadow));
    let catalog_value = token.as_ref().and_then(|token| ctx.catalog().get(token).map(|value| value.to_string()));
    let layers: Vec<ButtonInspectElevationLayer> = look
        .shadow
        .as_ref()
        .map(|shadows| shadows.iter().enumerate().map(|(index, shadow)| elevation_layer(index, shadow)).collect())
        .unwrap_or_default();
    let applied = !state.disabled && !layers.is_empty();

    ButtonInspectElevation {
        applied,
        rule_shadow,
        style_key: style_key.to_string(),
        token,
        catalog_value,
        layers,
        shadows: look.shadow.clone(),
    }
}

pub fn format_inspect_box_shadow_layer(layer: &ButtonInspectElevationLayer) -> String {
    layer.css.clone()
}

pub fn inspect_button_typography(mode: &ShadcnModeTokens, theme_mode: ThemeMode) -> ButtonInspectTypography {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let typography = ctx.typography();
    let catalog = ctx.catalog();
    let label = &typography.text.label;

    ButtonInspectTypography {
        font_family: typography_family_field(catalog, &typography.font.sans.family),
        font_size: typography_scaffold_field("LumaTypography.text.label.size", label.size),
        font_weight: typography_scaffold_field("LumaTypography.text.label.weight", label.weight.0),
        line_height: typography_scaffold_field("LumaTypography.text.label.line_height", label.line_height),
    }
}

fn typography_family_field(catalog: &gpui_luma_look_shadcn::catalog::CssTokenMap, family: &str) -> ResolvedTypography {
    if catalog.get("font-sans").is_some() {
        ResolvedTypography { value: family.to_string(), source: TypographySource::CssVar { token: "font-sans".into() } }
    } else {
        ResolvedTypography {
            value: family.to_string(),
            source: TypographySource::Scaffold { path: "LumaTypography.font.sans.family".into() },
        }
    }
}

fn typography_scaffold_field(path: &str, value: f32) -> ResolvedTypography {
    let display = if (value - value.round()).abs() < f32::EPSILON {
        format!("{}", value.round() as i32)
    } else {
        format!("{value}")
    };
    ResolvedTypography { value: display, source: TypographySource::Scaffold { path: path.into() } }
}

fn effective_button_style(style: ShadcnButtonStyle, role: ButtonFamilyRole) -> ShadcnButtonStyle {
    if matches!(role, ButtonFamilyRole::Toggle { selected: false }) {
        ShadcnButtonStyle::Outline
    } else {
        style
    }
}

fn button_style_key(style: ShadcnButtonStyle) -> &'static str {
    match style {
        ShadcnButtonStyle::Primary => "primary",
        ShadcnButtonStyle::Secondary => "secondary",
        ShadcnButtonStyle::Outline => "outline",
        ShadcnButtonStyle::Ghost => "ghost",
    }
}

fn elevation_layer(_index: usize, shadow: &BoxShadow) -> ButtonInspectElevationLayer {
    let offset_x = shadow.offset.x.as_f32();
    let offset_y = shadow.offset.y.as_f32();
    let blur = shadow.blur_radius.as_f32();
    let spread = shadow.spread_radius.as_f32();
    let color = shadow.color;
    let css = format!(
        "{}px {}px {}px {}px hsla({}, {}, {}, {})",
        format_shadow_number(offset_x),
        format_shadow_number(offset_y),
        format_shadow_number(blur),
        format_shadow_number(spread),
        format_shadow_number(color.h * 360.0),
        format_shadow_number(color.s * 100.0),
        format_shadow_number(color.l * 100.0),
        format_shadow_alpha(color.a),
    );

    ButtonInspectElevationLayer { offset_x, offset_y, blur, spread, color, css }
}

fn format_shadow_number(value: f32) -> String {
    if (value - value.round()).abs() < f32::EPSILON {
        format!("{}", value.round() as i32)
    } else {
        format!("{value}")
    }
}

fn format_shadow_alpha(value: f32) -> String {
    let formatted = format!("{:.3}", value.clamp(0.0, 1.0));
    formatted.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn control_size_key(size: ControlSize) -> &'static str {
    match size {
        ControlSize::Sm => "sm",
        ControlSize::Md => "md",
        ControlSize::Lg => "lg",
    }
}

fn scaffold_control_metric(size_key: &str, field: &str, value_px: f32) -> ResolvedMetric {
    ResolvedMetric {
        value_px,
        source: MetricSource::Scaffold { path: format!("MetricTokens.control.{size_key}.{field}") },
    }
}

fn spacing_control_metric(
    catalog: &gpui_luma_look_shadcn::catalog::CssTokenMap,
    size: ControlSize,
    field: SpacingField,
    value_px: f32,
) -> ResolvedMetric {
    if catalog.get("spacing").is_some() {
        let size_key = control_size_key(size);
        let field_label = match field {
            SpacingField::PaddingX => "padding_x",
            SpacingField::PaddingY => "padding_y",
            SpacingField::Gap => "gap",
        };
        let multiplier = gpui_luma_look_shadcn::catalog::spacing_multiplier(size, field);
        let multiplier_label = if (multiplier - multiplier.round()).abs() < f32::EPSILON {
            format!("{}", multiplier.round() as i32)
        } else {
            format!("{multiplier}")
        };
        ResolvedMetric {
            value_px,
            source: MetricSource::Derived {
                note: format!("{size_key} {field_label} = --spacing × {multiplier_label}"),
            },
        }
    } else {
        let field_label = match field {
            SpacingField::PaddingX => "padding_x",
            SpacingField::PaddingY => "padding_y",
            SpacingField::Gap => "gap",
        };
        scaffold_control_metric(control_size_key(size), field_label, value_px)
    }
}

fn radius_metric(
    catalog: &gpui_luma_look_shadcn::catalog::CssTokenMap,
    size: ControlSize,
    value_px: f32,
) -> ResolvedMetric {
    if catalog.get("radius").is_some() {
        let (size_label, offset) = match size {
            ControlSize::Sm => ("sm", 4.0_f32),
            ControlSize::Md => ("md", 2.0_f32),
            ControlSize::Lg => ("lg", 0.0_f32),
        };
        let offset_label = if (offset - offset.round()).abs() < f32::EPSILON {
            format!("{}px", offset.round() as i32)
        } else {
            format!("{offset}px")
        };
        ResolvedMetric {
            value_px,
            source: MetricSource::Derived { note: format!("{size_label} = --radius − {offset_label}") },
        }
    } else {
        scaffold_control_metric(control_size_key(size), "radius", value_px)
    }
}

fn focus_ring_offset_metric(border: gpui::Hsla, metrics: &gpui_luma::theme::MetricTokens) -> ResolvedMetric {
    let border_width = metrics.border_width.default;
    let focus = metrics.focus.width;
    if border.a <= 0.0 {
        ResolvedMetric { value_px: 0.0, source: MetricSource::Derived { note: "inset · borderless".into() } }
    } else {
        ResolvedMetric {
            value_px: border_width + focus,
            source: MetricSource::Derived { note: "border_width.default + focus.width".into() },
        }
    }
}

fn effective_border_resolved(colors: &gpui_luma_look_shadcn::tables::ButtonColorPalette) -> ResolvedColor {
    use gpui_luma_look_shadcn::ColorSource;

    if let Some(color) = &colors.border {
        return color.clone();
    }

    ResolvedColor { value: gpui::hsla(0.0, 0.0, 0.0, 0.0), source: ColorSource::Transparent }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::retro_arcade_catalog;

    #[test]
    fn button_color_table_metadata_is_populated() {
        let metadata = gpui_luma_look_shadcn::stylesheet::resolve_button_colors_metadata(
            gpui_luma_look_shadcn::embedded_stylesheet(),
        );
        assert!(!metadata.is_empty());
        assert!(metadata[0].inputs.len() == 4);
    }

    #[test]
    fn inspect_palette_focused_includes_ring_token() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let palette = inspect_button_color_palette(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            InteractionState { focused: true, ..InteractionState::default() },
        );
        let ring = palette.focus_ring.expect("focused inspect palette should include ring");
        assert!(matches!(ring.source, gpui_luma_look_shadcn::ColorSource::CssVar { ref token } if token == "ring"));
    }

    #[test]
    fn inspect_palette_inherited_border_references_background_token() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let palette = inspect_button_color_palette(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            InteractionState::default(),
        );
        assert!(matches!(
            palette.border.source,
            gpui_luma_look_shadcn::ColorSource::CssVar { ref token } if token == "primary"
        ));
        assert_eq!(palette.border.value, palette.background.value);
    }

    #[test]
    fn inspect_metrics_match_button_look_for_primary_default_md() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let look = gpui_luma_look_shadcn::paint::button_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            InteractionState::default(),
        );
        let metrics = inspect_button_metrics(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            InteractionState::default(),
        );

        assert_eq!(metrics.height.value_px, look.height);
        assert_eq!(metrics.padding_x.value_px, look.padding_x);
        assert_eq!(metrics.padding_y.value_px, look.padding_y);
        assert_eq!(metrics.gap.value_px, look.gap);
        assert_eq!(metrics.radius.value_px, look.radius);
    }

    #[test]
    fn inspect_elevation_outline_resolves_shadow_token_and_layers() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let elevation = inspect_button_elevation(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Outline,
            ButtonFamilyRole::Text,
            InteractionState::default(),
        );

        assert!(elevation.applied);
        assert_eq!(elevation.rule_shadow, "shadow-xs");
        assert_eq!(elevation.token.as_deref(), Some("shadow-xs"));
        assert!(elevation.catalog_value.as_ref().is_some_and(|value| !value.is_empty()));
        assert!(elevation.layers.len() >= 1);
        assert!(elevation.shadows.as_ref().is_some_and(|shadows| !shadows.is_empty()));
    }

    #[test]
    fn inspect_elevation_primary_has_no_shadow() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let elevation = inspect_button_elevation(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            InteractionState::default(),
        );

        assert!(!elevation.applied);
        assert_eq!(elevation.rule_shadow, "none");
        assert!(elevation.token.is_none());
        assert!(elevation.layers.is_empty());
    }

    #[test]
    fn inspect_elevation_disabled_suppresses_applied_shadow() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let elevation = inspect_button_elevation(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Outline,
            ButtonFamilyRole::Text,
            InteractionState { disabled: true, ..InteractionState::default() },
        );

        assert!(!elevation.applied);
        assert_eq!(elevation.rule_shadow, "shadow-xs");
        assert!(!elevation.layers.is_empty());
    }

    #[test]
    fn inspect_typography_uses_font_sans_catalog_and_scaffold_label_metrics() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let typography = inspect_button_typography(&mode, ThemeMode::Light);

        assert!(matches!(
            typography.font_family.source,
            gpui_luma_look_shadcn::TypographySource::CssVar { ref token } if token == "font-sans"
        ));
        assert_eq!(typography.font_family.value, "Outfit");
        assert!(matches!(
            typography.font_size.source,
            gpui_luma_look_shadcn::TypographySource::Scaffold { ref path } if path.contains("label.size")
        ));
        assert_eq!(typography.font_size.value, "12.5");
        assert_eq!(typography.line_height.value, "18");
    }
}
