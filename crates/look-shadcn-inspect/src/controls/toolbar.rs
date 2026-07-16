//! Inspect metadata for `toolbar`.

use gpui_luma::theme::{ControlSize, ThemeMode};
use gpui_luma_look_shadcn::{ColorSource, LookContext, ResolvedColor, ResolvedMetric, ShadcnModeTokens};

#[derive(Clone, Debug)]
pub struct ToolbarInspectPalette {
    pub background: ResolvedColor,
    pub border: ResolvedColor,
    pub separator: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ToolbarInspectMetrics {
    pub radius: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub gap: ResolvedMetric,
    pub separator_height: ResolvedMetric,
}

pub fn inspect_toolbar_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    enabled: bool,
) -> ToolbarInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, Default::default());
    let tokens = ctx.tokens;
    let background = if enabled {
        tokens.palette.muted_background
    } else {
        tokens.palette.disabled_background
    };

    ToolbarInspectPalette {
        background: resolved_from_hsla(background, ColorSource::CssVar { token: "muted".into() }),
        border: resolved_from_hsla(tokens.palette.border_default, ColorSource::CssVar { token: "border".into() }),
        separator: resolved_from_hsla(tokens.palette.border_default, ColorSource::CssVar { token: "border".into() }),
    }
}

pub fn inspect_toolbar_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> ToolbarInspectMetrics {
    use crate::metrics::{control_size_key, derived_metric, scaffold_control_metric};

    let ctx = LookContext::new(mode, theme_mode, Default::default());
    let metrics = ctx.metrics();
    let control = metrics.for_size(size);
    let size_key = control_size_key(size);

    ToolbarInspectMetrics {
        radius: derived_metric("toolbar radius = radius.md", metrics.radius.md),
        padding_x: derived_metric("toolbar padding x = 6px", 6.0),
        padding_y: derived_metric("toolbar padding y = 4px", 4.0),
        gap: derived_metric(format!("{size_key} toolbar gap = control gap"), control.gap),
        separator_height: scaffold_control_metric(size_key, "height", control.height),
    }
}

fn resolved_from_hsla(value: gpui::Hsla, source: ColorSource) -> ResolvedColor {
    ResolvedColor { value, source }
}
