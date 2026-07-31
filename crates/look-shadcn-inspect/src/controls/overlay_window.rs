//! Inspect metadata for `overlay_window`.

use gpui_luma::controls::overlay_window::OverlayWindowMode;
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::{ColorSource, ResolvedColor, ResolvedMetric, ShadcnLook};

#[derive(Clone, Debug)]
pub struct OverlayWindowInspectPalette {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
    pub backdrop: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct OverlayWindowInspectMetrics {
    pub radius: ResolvedMetric,
    pub padding: ResolvedMetric,
    pub min_width: ResolvedMetric,
    pub max_width: ResolvedMetric,
    pub estimated_height: ResolvedMetric,
}

pub fn inspect_overlay_window_color_palette(
    look: &ShadcnLook,
    size: ControlSize,
    mode: OverlayWindowMode,
) -> OverlayWindowInspectPalette {
    let shell = gpui_luma_look_shadcn::paint::overlay_window_look(look, size, mode);

    OverlayWindowInspectPalette {
        background: resolved_from_hsla(shell.background, ColorSource::CssVar { token: "popover".into() }),
        foreground: resolved_from_hsla(shell.foreground, ColorSource::CssVar { token: "popover-foreground".into() }),
        border: resolved_from_hsla(shell.border, ColorSource::CssVar { token: "border".into() }),
        backdrop: (mode == OverlayWindowMode::Modal).then(|| {
            resolved_from_hsla(shell.backdrop_background, ColorSource::Derived { note: "modal scrim".into() })
        }),
    }
}

pub fn inspect_overlay_window_metrics(
    look: &ShadcnLook,
    size: ControlSize,
    mode: OverlayWindowMode,
) -> OverlayWindowInspectMetrics {
    use crate::metrics::derived_metric;

    let shell = gpui_luma_look_shadcn::paint::overlay_window_look(look, size, mode);
    let size_key = match size {
        ControlSize::Sm => "sm",
        ControlSize::Md => "md",
        ControlSize::Lg => "lg",
    };

    OverlayWindowInspectMetrics {
        radius: derived_metric("overlay radius = radius.lg", shell.radius),
        padding: derived_metric(format!("{size_key} overlay padding"), shell.padding),
        min_width: derived_metric(format!("{size_key} overlay min width"), shell.min_width),
        max_width: derived_metric(format!("{size_key} overlay max width"), shell.max_width),
        estimated_height: derived_metric(format!("{size_key} overlay estimated height"), shell.estimated_height),
    }
}

fn resolved_from_hsla(value: gpui::Hsla, source: ColorSource) -> ResolvedColor {
    ResolvedColor { value, source }
}
