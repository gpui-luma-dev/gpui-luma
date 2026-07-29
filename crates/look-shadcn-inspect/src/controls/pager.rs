//! Inspect metadata for `pager`.

use gpui_luma::controls::pager::PagerStyle;
use gpui_luma_look_shadcn::{ColorSource, ResolvedColor, ResolvedMetric, ShadcnLook};

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
}

pub fn inspect_pager_shell_color_palette(
    look: &ShadcnLook,
    enabled: bool,
    style: PagerStyle,
) -> PagerShellInspectPalette {
    let pager = gpui_luma_look_shadcn::paint::pager_look(look, enabled, style);

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

    let pager = gpui_luma_look_shadcn::paint::pager_look(look, true, style);
    let style_label = pager_style_label(style);

    PagerInspectMetrics {
        control_height: derived_metric(format!("{style_label} page-size trigger height"), pager.control_height),
        button_size: derived_metric(format!("{style_label} button size"), pager.button_size),
        button_min_width: derived_metric(format!("{style_label} button min width"), pager.button_min_width),
        radius: derived_metric(format!("{style_label} radius = radius.sm"), pager.radius),
        padding_x: derived_metric(format!("{style_label} shell padding x"), pager.padding_x),
        padding_y: derived_metric(format!("{style_label} shell padding y"), pager.padding_y),
        gap: derived_metric(format!("{style_label} item gap"), pager.gap),
        group_gap: derived_metric(format!("{style_label} group gap"), pager.group_gap),
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
