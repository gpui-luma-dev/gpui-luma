//! Inspect metadata for `pager`.

use luma::controls::{
    button_family::ButtonFamilyRole, choice_indicator_layout::shadow_projection_extent, pager::PagerStyle,
};
use luma_look_shadcn::{ColorSource, ResolvedColor, ResolvedMetric, ResolvedTypography, ShadcnLook, TypographySource};

#[derive(Clone, Debug)]
pub struct PagerShellInspectPalette {
    pub panel_background: ResolvedColor,
    pub border: ResolvedColor,
    pub body_text: ResolvedColor,
    pub muted_text: ResolvedColor,
    pub selected_background: ResolvedColor,
    pub selected_foreground: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct PagerInspectMetrics {
    pub control_height: ResolvedMetric,
    pub button_size: ResolvedMetric,
    pub button_min_width: ResolvedMetric,
    pub radius: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub gap: ResolvedMetric,
    pub group_gap: ResolvedMetric,
    pub font_family: ResolvedTypography,
    pub reserved_shadow_extent: ResolvedMetric,
}

pub fn inspect_pager_shell_color_palette(
    look: &ShadcnLook,
    enabled: bool,
    style: PagerStyle,
) -> PagerShellInspectPalette {
    let pager = luma_look_shadcn::paint::pager_look(look, enabled, style);

    PagerShellInspectPalette {
        panel_background: resolved_shell_background(enabled, pager.panel_background),
        border: resolved_from_hsla(pager.border, ColorSource::CssVar { token: "border".into() }),
        body_text: resolved_shell_foreground(enabled, pager.body_text),
        muted_text: resolved_shell_muted(enabled, pager.muted_text),
        selected_background: resolved_from_hsla(
            pager.selected_background,
            ColorSource::CssVar { token: "accent".into() },
        ),
        selected_foreground: resolved_from_hsla(
            pager.selected_foreground,
            ColorSource::CssVar { token: "accent-foreground".into() },
        ),
    }
}

pub fn inspect_pager_metrics(look: &ShadcnLook, style: PagerStyle) -> PagerInspectMetrics {
    use crate::metrics::derived_metric;

    let pager = luma_look_shadcn::paint::pager_look(look, true, style);
    let style_label = pager_style_label(style);
    let button = look.resolve_outline_button(
        ButtonFamilyRole::Toggle { selected: false },
        luma::theme::ControlSize::Sm,
        luma::theme::InteractionState::default(),
    );
    let reserved_shadow_extent = shadow_projection_extent(button.shadow.as_deref(), 1.0, true);

    PagerInspectMetrics {
        control_height: derived_metric(format!("{style_label} page-size trigger height"), pager.control_height),
        button_size: derived_metric(format!("{style_label} button size"), pager.button_size),
        button_min_width: derived_metric(format!("{style_label} button min width"), pager.button_min_width),
        radius: derived_metric(format!("{style_label} radius = radius.sm"), pager.radius),
        padding_x: derived_metric(format!("{style_label} shell padding x"), pager.padding_x),
        padding_y: derived_metric(format!("{style_label} shell padding y"), pager.padding_y),
        gap: derived_metric(format!("{style_label} item gap"), pager.gap),
        group_gap: derived_metric(format!("{style_label} group gap"), pager.group_gap),
        font_family: pager_font_family(look),
        reserved_shadow_extent: ResolvedMetric {
            value_px: reserved_shadow_extent,
            source: luma_look_shadcn::MetricSource::Derived { note: "button outline shadow projection extent".into() },
        },
    }
}

fn pager_font_family(look: &ShadcnLook) -> ResolvedTypography {
    let family = look.mode_tokens().typography.font.sans.family.clone();
    if look.mode_tokens().catalog.get("font-sans").is_some() {
        ResolvedTypography { value: family, source: TypographySource::CssVar { token: "font-sans".into() } }
    } else {
        ResolvedTypography {
            value: family,
            source: TypographySource::Scaffold { path: "LumaTypography.font.sans.family".into() },
        }
    }
}

fn pager_style_label(style: PagerStyle) -> &'static str {
    match style {
        PagerStyle::Minimal => "minimal",
        PagerStyle::MinimalEdge => "minimal-edge",
        PagerStyle::Numeric => "numeric",
    }
}

fn resolved_shell_background(enabled: bool, value: gpui::Hsla) -> ResolvedColor {
    if enabled {
        resolved_from_hsla(value, ColorSource::CssVar { token: "card".into() })
    } else {
        resolved_from_hsla(value, ColorSource::Derived { note: "card · disabled".into() })
    }
}

fn resolved_shell_foreground(enabled: bool, value: gpui::Hsla) -> ResolvedColor {
    if enabled {
        resolved_from_hsla(value, ColorSource::CssVar { token: "foreground".into() })
    } else {
        resolved_from_hsla(value, ColorSource::Derived { note: "foreground · disabled".into() })
    }
}

fn resolved_shell_muted(enabled: bool, value: gpui::Hsla) -> ResolvedColor {
    if enabled {
        resolved_from_hsla(value, ColorSource::CssVar { token: "muted-foreground".into() })
    } else {
        resolved_from_hsla(value, ColorSource::Derived { note: "muted-foreground · disabled".into() })
    }
}

fn resolved_from_hsla(value: gpui::Hsla, source: ColorSource) -> ResolvedColor {
    ResolvedColor { value, source }
}
