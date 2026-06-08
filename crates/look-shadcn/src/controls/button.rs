use gpui::{Hsla, hsla};

use gpui_luma::controls::button_family::{
    ButtonFamilyAppearance, ButtonFamilyPalette, ButtonFamilyRole, compose_button_family_appearance,
};
use gpui_luma::theme::adorner::{AdornerPlacement, AdornerSpec, FocusRingAdornerSpec};
use gpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, StandardBoxScale, ThemeMode};

use crate::appearance_context::AppearanceContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, MetricSource, ResolvedColor, ResolvedMetric};

use gpui_luma_look_shadcn_macros::declare_look_table;

/// Radix-style button appearance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShadcnButtonStyle {
    Primary,
    Secondary,
    Outline,
    Ghost,
}

#[derive(Clone, Debug)]
pub struct ButtonColorPalette {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: Option<ResolvedColor>,
}

impl ButtonColorPalette {
    pub fn fallback() -> Self {
        Self {
            background: ResolvedColor::transparent(),
            foreground: ResolvedColor::fallback_foreground(),
            border: None,
        }
    }
}

declare_look_table! {
    name: resolve_button_colors,
    inputs: {
        style: ShadcnButtonStyle,
        layer: InteractionLayer,
        mode: ThemeMode,
        selected: bool,
    },
    output: ButtonColorPalette { background, foreground, border },
    matrix: [
        [ShadcnButtonStyle::Primary]   | [InteractionLayer::Default]  | [_] | [_] => "primary"         | "primary-foreground"   | None,
        [ShadcnButtonStyle::Primary]   | [InteractionLayer::Hovered]  | [_] | [_] => "primary-hover"   | "primary-foreground"   | None,
        [ShadcnButtonStyle::Primary]   | [InteractionLayer::Pressed]  | [_] | [_] => "primary-pressed" | "primary-foreground"   | None,
        [ShadcnButtonStyle::Primary]   | [InteractionLayer::Disabled] | [_] | [_] => "muted"           | "muted-foreground"     | None,

        [ShadcnButtonStyle::Secondary] | [InteractionLayer::Default]  | [_] | [_] => "secondary"         | "secondary-foreground" | None,
        [ShadcnButtonStyle::Secondary] | [InteractionLayer::Hovered]  | [_] | [_] => "secondary-hover"   | "secondary-foreground" | None,
        [ShadcnButtonStyle::Secondary] | [InteractionLayer::Pressed]  | [_] | [_] => "secondary-pressed" | "secondary-foreground" | None,
        [ShadcnButtonStyle::Secondary] | [InteractionLayer::Disabled] | [_] | [_] => "muted"             | "muted-foreground"     | None,

        [ShadcnButtonStyle::Outline]   | [InteractionLayer::Default]  | [ThemeMode::Light] | [true]  => "primary"         | "primary-foreground" | "border",
        [ShadcnButtonStyle::Outline]   | [InteractionLayer::Default]  | [ThemeMode::Dark]  | [true]  => "primary"         | "primary-foreground" | "input",
        [ShadcnButtonStyle::Outline]   | [InteractionLayer::Hovered]  | [ThemeMode::Light] | [true]  => "primary-hover"   | "primary-foreground" | "border",
        [ShadcnButtonStyle::Outline]   | [InteractionLayer::Hovered]  | [ThemeMode::Dark]  | [true]  => "primary-hover"   | "primary-foreground" | "input",
        [ShadcnButtonStyle::Outline]   | [InteractionLayer::Pressed]  | [ThemeMode::Light] | [true]  => "primary-pressed" | "primary-foreground" | "border",
        [ShadcnButtonStyle::Outline]   | [InteractionLayer::Pressed]  | [ThemeMode::Dark]  | [true]  => "primary-pressed" | "primary-foreground" | "input",
        [ShadcnButtonStyle::Outline]   | [InteractionLayer::Default]  | [ThemeMode::Light] | [false] => "background"      | "foreground"         | "border",
        [ShadcnButtonStyle::Outline]   | [InteractionLayer::Default]  | [ThemeMode::Dark]  | [false] => "input/30"        | "foreground"         | "input",
        [ShadcnButtonStyle::Outline]   | [InteractionLayer::Hovered]  | [ThemeMode::Light] | [false] => "accent"          | "accent-foreground"  | "border",
        [ShadcnButtonStyle::Outline]   | [InteractionLayer::Hovered]  | [ThemeMode::Dark]  | [false] => "input/50"        | "accent-foreground"  | "input",
        [ShadcnButtonStyle::Outline]   | [InteractionLayer::Pressed]  | [ThemeMode::Light] | [false] => "accent"          | "accent-foreground"  | "border",
        [ShadcnButtonStyle::Outline]   | [InteractionLayer::Pressed]  | [ThemeMode::Dark]  | [false] => "input/50"        | "accent-foreground"  | "input",
        [ShadcnButtonStyle::Outline]   | [InteractionLayer::Disabled] | [_]                | [_]    => "transparent"     | "muted-foreground"   | "border",

        [ShadcnButtonStyle::Ghost]     | [InteractionLayer::Default]  | [_]                | [true]  => "primary"         | "primary-foreground" | None,
        [ShadcnButtonStyle::Ghost]     | [InteractionLayer::Hovered]  | [_]                | [true]  => "primary-hover"   | "primary-foreground" | None,
        [ShadcnButtonStyle::Ghost]     | [InteractionLayer::Pressed]  | [_]                | [true]  => "primary-pressed" | "primary-foreground" | None,
        [ShadcnButtonStyle::Ghost]     | [InteractionLayer::Default]  | [_]                | [false] => "transparent"     | "foreground"         | None,
        [ShadcnButtonStyle::Ghost]     | [InteractionLayer::Hovered]  | [ThemeMode::Light] | [false] => "accent"          | "accent-foreground"  | None,
        [ShadcnButtonStyle::Ghost]     | [InteractionLayer::Hovered]  | [ThemeMode::Dark]  | [false] => "accent/50"       | "accent-foreground"  | None,
        [ShadcnButtonStyle::Ghost]     | [InteractionLayer::Pressed]  | [ThemeMode::Light] | [false] => "accent"          | "accent-foreground"  | None,
        [ShadcnButtonStyle::Ghost]     | [InteractionLayer::Pressed]  | [ThemeMode::Dark]  | [false] => "accent/50"       | "accent-foreground"  | None,
        [ShadcnButtonStyle::Ghost]     | [InteractionLayer::Disabled] | [_]                | [_]    => "transparent"     | "muted-foreground"   | None,

        [_]                            | [_]                          | [_]                | [_]    => "transparent"     | "foreground"         | None,
    ]
}

pub(crate) fn button_appearance(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    size: ControlSize,
    state: InteractionState,
) -> ButtonFamilyAppearance {
    let ctx = AppearanceContext::new(mode, theme_mode, state);
    let palette = button_palette(&ctx, style, role, size);
    let scale = StandardBoxScale::compute(size, ctx.metrics(), 1.0);
    compose_button_family_appearance(&palette, role, &scale, ctx.metrics().radius.pill)
}

#[derive(Clone, Debug)]
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
    let ctx = AppearanceContext::new(mode, theme_mode, state);
    let style = if matches!(role, ButtonFamilyRole::Toggle { selected: false }) {
        ShadcnButtonStyle::Outline
    } else {
        style
    };
    let layer = state.layer();
    let selected = matches!(role, ButtonFamilyRole::Toggle { selected: true });
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "button_resolver");
    let colors = resolve_button_colors(&resolver, style, layer, theme_mode, selected)
        .unwrap_or_else(|_| ButtonColorPalette::fallback());

    let border = effective_border_resolved(style, &colors);
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
    let appearance = button_appearance(mode, theme_mode, style, role, size, state);
    let ctx = AppearanceContext::new(mode, theme_mode, state);
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();

    let size_key = control_size_key(size);

    ButtonInspectMetrics {
        height: scaffold_control_metric(size_key, "control_height", appearance.height),
        padding_x: scaffold_control_metric(size_key, "padding_x", appearance.padding_x),
        padding_y: scaffold_control_metric(size_key, "padding_y", appearance.padding_y),
        gap: scaffold_control_metric(size_key, "gap", appearance.gap),
        radius: radius_metric(catalog, size, appearance.radius),
        border_width: ResolvedMetric {
            value_px: metrics.border_width.default,
            source: MetricSource::Scaffold { path: "MetricTokens.border_width.default".into() },
        },
        focus_ring_width: ResolvedMetric {
            value_px: metrics.focus.width,
            source: MetricSource::Scaffold { path: "MetricTokens.focus.width".into() },
        },
        focus_ring_offset: focus_ring_offset_metric(style, metrics),
    }
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

fn radius_metric(catalog: &crate::catalog::CssTokenMap, size: ControlSize, value_px: f32) -> ResolvedMetric {
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

fn focus_ring_offset_metric(style: ShadcnButtonStyle, metrics: &gpui_luma::theme::MetricTokens) -> ResolvedMetric {
    let border = metrics.border_width.default;
    let focus = metrics.focus.width;
    match style {
        ShadcnButtonStyle::Ghost => ResolvedMetric {
            value_px: border,
            source: MetricSource::Derived { note: "inset · border_width.default".into() },
        },
        _ => ResolvedMetric {
            value_px: border + focus,
            source: MetricSource::Derived { note: "border_width.default + focus.width".into() },
        },
    }
}

fn effective_border_resolved(style: ShadcnButtonStyle, colors: &ButtonColorPalette) -> ResolvedColor {
    use crate::provenance::{ColorSource, format_inspect_css_key};

    if let Some(color) = &colors.border {
        return color.clone();
    }

    let background = colors.background.hsla();
    let (value, source) = match style {
        ShadcnButtonStyle::Ghost => (hsla(0.0, 0.0, 0.0, 0.0), ColorSource::Transparent),
        ShadcnButtonStyle::Primary | ShadcnButtonStyle::Secondary | ShadcnButtonStyle::Outline => (
            background,
            ColorSource::Derived { note: format!("= {}", format_inspect_css_key(&colors.background.source)) },
        ),
    };

    ResolvedColor { value, source }
}

pub(crate) fn button_palette(
    ctx: &AppearanceContext,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    _size: ControlSize,
) -> ButtonFamilyPalette {
    let state = ctx.state;
    let style = if matches!(role, ButtonFamilyRole::Toggle { selected: false }) {
        ShadcnButtonStyle::Outline
    } else {
        style
    };

    let layer = state.layer();
    let theme_mode = ctx.theme_mode;
    let palette = ctx.palette();
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let selected = matches!(role, ButtonFamilyRole::Toggle { selected: true });

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "button_resolver");
    let colors = resolve_button_colors(&resolver, style, layer, theme_mode, selected)
        .unwrap_or_else(|_| ButtonColorPalette::fallback());

    let background = colors.background.hsla();
    let foreground = colors.foreground.hsla();
    let border = resolve_button_border(style, &colors, background);

    let adorner = if state.focused {
        let (placement, distance) = match style {
            ShadcnButtonStyle::Ghost => (AdornerPlacement::Inset, metrics.border_width.default),
            _ => (AdornerPlacement::Oversize, metrics.border_width.default + metrics.focus.width),
        };

        Some(AdornerSpec::FocusRing(FocusRingAdornerSpec {
            color: palette.focus_ring,
            placement,
            distance,
            width: metrics.focus.width,
        }))
    } else {
        None
    };

    ButtonFamilyPalette {
        background,
        foreground,
        border,
        adorner,
        typography: typography.text.label,
        font_family: typography.font.sans.family.clone().into(),
    }
}

fn resolve_button_border(style: ShadcnButtonStyle, colors: &ButtonColorPalette, background: Hsla) -> Hsla {
    match &colors.border {
        Some(color) => color.hsla(),
        None => match style {
            ShadcnButtonStyle::Ghost => hsla(0.0, 0.0, 0.0, 0.0),
            ShadcnButtonStyle::Primary | ShadcnButtonStyle::Secondary => background,
            ShadcnButtonStyle::Outline => background,
        },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use gpui_luma::controls::button_family::ButtonFamilyRole;
    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use gpui_luma::theme::ThemeMode;

    fn retro_arcade_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "hsl(330.9554 64.0816% 51.9608%)".into()),
            ("primary-foreground".into(), "hsl(0 0% 100%)".into()),
            ("secondary".into(), "hsl(175.4622 58.6207% 39.8039%)".into()),
            ("secondary-foreground".into(), "hsl(0 0% 100%)".into()),
            ("background".into(), "hsl(43.8462 86.6667% 94.1176%)".into()),
            ("foreground".into(), "hsl(192.2034 80.8219% 14.3137%)".into()),
            ("muted".into(), "hsl(180 6.9307% 60.3922%)".into()),
            ("muted-foreground".into(), "hsl(192.2034 80.8219% 14.3137%)".into()),
            ("accent".into(), "hsl(17.5691 80.4444% 44.1176%)".into()),
            ("accent-foreground".into(), "hsl(0 0% 100%)".into()),
            ("destructive".into(), "hsl(1.0405 71.1934% 52.3529%)".into()),
            ("destructive-foreground".into(), "hsl(0 0% 100%)".into()),
            ("border".into(), "hsl(186.3158 8.2969% 55.0980%)".into()),
            ("input".into(), "hsl(186.3158 8.2969% 55.0980%)".into()),
            ("ring".into(), "hsl(330.9554 64.0816% 51.9608%)".into()),
            ("card".into(), "hsl(45.6000 42.3729% 88.4314%)".into()),
            ("radius".into(), "0.25rem".into()),
        ]))
    }

    #[test]
    fn button_color_table_metadata_is_populated() {
        let metadata = resolve_button_colors_metadata();
        assert!(!metadata.is_empty());
        assert!(metadata[0].inputs.len() == 4);
    }

    #[test]
    fn primary_hover_background_differs_from_default() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let default = button_appearance(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            InteractionState::default(),
        );
        let hovered = button_appearance(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            InteractionState { hovered: true, ..InteractionState::default() },
        );

        assert_ne!(default.background, hovered.background);
    }

    #[test]
    fn ghost_light_hover_pairs_accent_fill_with_accent_foreground() {
        let catalog = retro_arcade_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let hovered = button_appearance(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Ghost,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            InteractionState { hovered: true, ..InteractionState::default() },
        );

        assert_eq!(hovered.background, catalog.color("accent").expect("accent"));
        assert_eq!(hovered.foreground, catalog.color("accent-foreground").expect("accent-foreground"));
    }

    #[test]
    fn ghost_dark_hover_pairs_accent_half_fill_with_accent_foreground() {
        let catalog = retro_arcade_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Dark).expect("catalog");
        let accent = catalog.color("accent").expect("accent");
        let hovered = button_appearance(
            &mode,
            ThemeMode::Dark,
            ShadcnButtonStyle::Ghost,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            InteractionState { hovered: true, ..InteractionState::default() },
        );

        assert!((hovered.background.a - 0.50).abs() < f32::EPSILON);
        assert_eq!(hovered.background.h, accent.h);
        assert_eq!(hovered.foreground, catalog.color("accent-foreground").expect("accent-foreground"));
    }

    #[test]
    fn outline_light_hover_uses_accent_fill_and_accent_foreground() {
        let catalog = retro_arcade_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let hovered = button_appearance(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Outline,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            InteractionState { hovered: true, ..InteractionState::default() },
        );

        assert_eq!(hovered.background, catalog.color("accent").expect("accent"));
        assert_eq!(hovered.foreground, catalog.color("accent-foreground").expect("accent-foreground"));
    }

    #[test]
    fn outline_dark_hover_uses_input_alpha_with_accent_foreground() {
        let catalog = retro_arcade_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Dark).expect("catalog");
        let input = catalog.color("input").expect("input");
        let hovered = button_appearance(
            &mode,
            ThemeMode::Dark,
            ShadcnButtonStyle::Outline,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            InteractionState { hovered: true, ..InteractionState::default() },
        );

        assert!((hovered.background.a - 0.50).abs() < f32::EPSILON);
        assert_eq!(hovered.background.h, input.h);
        assert_eq!(hovered.foreground, catalog.color("accent-foreground").expect("accent-foreground"));
        assert_eq!(hovered.border, input);
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
        assert!(matches!(ring.source, crate::provenance::ColorSource::CssVar { ref token } if token == "ring"));
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
            crate::provenance::ColorSource::Derived { ref note } if note == "= --primary"
        ));
    }

    #[test]
    fn inspect_metrics_match_button_appearance_for_primary_default_md() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let appearance = button_appearance(
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

        assert_eq!(metrics.height.value_px, appearance.height);
        assert_eq!(metrics.padding_x.value_px, appearance.padding_x);
        assert_eq!(metrics.padding_y.value_px, appearance.padding_y);
        assert_eq!(metrics.gap.value_px, appearance.gap);
        assert_eq!(metrics.radius.value_px, appearance.radius);
    }

    #[test]
    fn retro_arcade_metrics_use_radius_catalog_and_scaffold_padding() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let metrics = inspect_button_metrics(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            InteractionState::default(),
        );

        assert!(matches!(metrics.radius.source, crate::provenance::MetricSource::Derived { .. }));
        assert!(matches!(
            metrics.padding_x.source,
            crate::provenance::MetricSource::Scaffold { ref path } if path.contains("padding_x")
        ));
        assert_eq!(metrics.radius.value_px, 2.0);
    }

    #[test]
    fn focused_ghost_and_primary_focus_ring_offsets_differ() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let focused = InteractionState { focused: true, ..InteractionState::default() };
        let primary = inspect_button_metrics(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            focused,
        );
        let ghost = inspect_button_metrics(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Ghost,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            focused,
        );

        assert_ne!(primary.focus_ring_offset.value_px, ghost.focus_ring_offset.value_px);
        assert!(matches!(
            ghost.focus_ring_offset.source,
            crate::provenance::MetricSource::Derived { ref note } if note.contains("inset")
        ));
        assert!(matches!(
            primary.focus_ring_offset.source,
            crate::provenance::MetricSource::Derived { ref note } if note.contains('+')
        ));
    }

    #[test]
    fn retro_arcade_primary_hover_is_subtle() {
        let catalog = retro_arcade_catalog();
        let light = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("light");
        let dark = ShadcnModeTokens::from_catalog(catalog, ThemeMode::Dark).expect("dark");

        for (mode, theme_mode) in [(&light, ThemeMode::Light), (&dark, ThemeMode::Dark)] {
            let default = button_appearance(
                mode,
                theme_mode,
                ShadcnButtonStyle::Primary,
                ButtonFamilyRole::Text,
                ControlSize::Md,
                InteractionState::default(),
            );
            let hovered = button_appearance(
                mode,
                theme_mode,
                ShadcnButtonStyle::Primary,
                ButtonFamilyRole::Text,
                ControlSize::Md,
                InteractionState { hovered: true, ..InteractionState::default() },
            );
            assert!((default.background.l - hovered.background.l).abs() < 0.08);
        }
    }
}
