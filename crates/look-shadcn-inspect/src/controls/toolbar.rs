//! Inspect metadata for `toolbar`.

use luma::controls::toolbar::ToolbarVariant;
use luma::theme::{ControlSize, ThemeMode};
use luma_look_shadcn::{ResolvedColor, ResolvedMetric, ShadcnModeTokens};

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
    variant: ToolbarVariant,
) -> ToolbarInspectPalette {
    let colors = luma_look_shadcn::tables::resolve_toolbar_colors(mode, theme_mode, enabled, variant);
    ToolbarInspectPalette { background: colors.background, border: colors.border, separator: colors.separator }
}

pub fn inspect_toolbar_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> ToolbarInspectMetrics {
    let table = luma_look_shadcn::tables::metrics::resolve_toolbar_metrics(mode, theme_mode, size);
    table.into()
}

impl From<luma_look_shadcn::tables::metrics::ToolbarMetricTable> for ToolbarInspectMetrics {
    fn from(table: luma_look_shadcn::tables::metrics::ToolbarMetricTable) -> Self {
        Self {
            radius: table.radius,
            padding_x: table.padding_x,
            padding_y: table.padding_y,
            gap: table.gap,
            separator_height: table.separator_height,
        }
    }
}
